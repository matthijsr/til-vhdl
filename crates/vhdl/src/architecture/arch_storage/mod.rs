use std::sync::Arc;

use tydi_common::error::{Error, Result};
use tydi_intern::Id;

use crate::{
    common::vhdl_name::VhdlName, component::Component, declaration::ObjectDeclaration,
    object::Object, package::Package,
};

use self::{interner::Interner, object_queries::ObjectQueries};

use std::convert::TryInto;

use crate::{
    assignment::{
        array_assignment::ArrayAssignment, Assignment, AssignmentKind, DirectAssignment,
        FieldSelection, RangeConstraint,
    },
    object::object_type::ObjectType,
};

use self::object_queries::object_key::ObjectKey;

pub mod db;
pub mod get_name;
pub mod get_self;
pub mod intern_self;
pub mod interner;
pub mod object_queries;

#[salsa::query_group(ArchStorage)]
pub trait Arch: Interner + ObjectQueries {
    #[salsa::input]
    fn default_package(&self) -> Arc<Package>;

    #[salsa::input]
    fn subject_component_name(&self) -> Arc<VhdlName>;

    fn subject_component(&self) -> Result<Arc<Component>>;

    fn can_assign(
        &self,
        to: ObjectKey,
        assignment: Assignment,
        state: AssignmentState,
    ) -> Result<()>;

    /// Get an object based on its key
    fn get_object(&self, key: ObjectKey) -> Result<Object>;

    fn get_object_type(&self, key: ObjectKey) -> Result<Arc<ObjectType>>;

    fn get_object_declaration_type(&self, key: Id<ObjectDeclaration>) -> Result<Arc<ObjectType>>;
}

fn get_object(db: &dyn Arch, key: ObjectKey) -> Result<Object> {
    let obj = db.lookup_intern_object(key.obj());
    let typ = db
        .lookup_intern_object_type(obj.typ)
        .get_nested(db, key.selection())?;
    Ok(Object {
        typ: db.intern_object_type(typ),
        assignable: obj.assignable,
    })
}

fn get_object_type(db: &dyn Arch, key: ObjectKey) -> Result<Arc<ObjectType>> {
    Ok(Arc::new(
        db.lookup_intern_object_type(db.get_object(key)?.typ_id()),
    ))
}

fn get_object_declaration_type(
    db: &dyn Arch,
    key: Id<ObjectDeclaration>,
) -> Result<Arc<ObjectType>> {
    db.get_object_type(
        db.lookup_intern_object_declaration(key)
            .object_key()
            .clone(),
    )
}

fn subject_component(db: &dyn Arch) -> Result<Arc<Component>> {
    let package = db.default_package();
    package.get_subject_component(db)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssignmentState {
    /// Default behavior, trying to assign to an object from something else
    Default,
    /// Inverted behavior, required for mapping to out ports/signals
    /// of components and procedures
    OutMapping,
    /// Default behavior, but omits the "to" check. Required for constants and
    /// generic parameters.
    Initialization,
}

fn can_assign(
    db: &dyn Arch,
    to: ObjectKey,
    assignment: Assignment,
    state: AssignmentState,
) -> Result<()> {
    let to_key = to.with_nested(assignment.to_field().clone());
    let to = db.get_object(to_key.clone())?;
    match state {
        AssignmentState::Default => to.assignable.to_or_err()?,
        AssignmentState::OutMapping => to.assignable.from_or_err()?,
        AssignmentState::Initialization => (),
    };
    let to_typ = db.lookup_intern_object_type(to.typ);
    match assignment.kind() {
        AssignmentKind::Relation(relation) => match state {
            AssignmentState::Default => relation.can_assign(db, &to_typ),
            AssignmentState::OutMapping => relation.can_be_assigned(db, &to_typ),
            AssignmentState::Initialization => relation.can_assign(db, &to_typ),
        },
        AssignmentKind::Direct(direct) => {
            match direct {
                DirectAssignment::FullRecord(record) => {
                    if let ObjectType::Record(to_record) = &to_typ {
                        if to_record.fields().len() == record.len() {
                            for ra in record {
                                let to_field_key = to_key
                                    .clone()
                                    .with_selection(FieldSelection::name(ra.field().clone()));
                                db.can_assign(
                                    to_field_key,
                                    Assignment::from(ra.assignment().clone()),
                                    state,
                                )?;
                            }
                            Ok(())
                        } else {
                            Err(Error::InvalidArgument(format!("Attempted full record assignment. Number of fields do not match. Record has {} fields, assignment has {} fields", to_record.fields().len(), record.len())))
                        }
                    } else {
                        Err(Error::InvalidTarget(format!(
                            "Cannot perform full Record assignment to {}",
                            to_typ
                        )))
                    }
                }
                DirectAssignment::FullArray(array) => {
                    if let ObjectType::Array(to_array) = &to_typ {
                        // As each element is the same and we only really care about the type, using a single ObjectKey to represent all queries
                        // will be more efficient. (As this means Salsa is more likely to reuse previous results.)
                        let to_array_elem_key = to_key.clone().with_selection(
                            FieldSelection::Range(RangeConstraint::Index(to_array.high().clone())),
                        );
                        match array {
                            ArrayAssignment::Direct(direct) => {
                                match to_array.width()? {
                                    Some(w) if w != direct.len().try_into().unwrap() => Err(Error::InvalidArgument(format!("Attempted full array assignment. Number of fields do not match. Array has {} fields, assignment has {} fields", w, direct.len()))),
                                    _ => {
                                        for value in direct {
                                            db.can_assign(
                                                to_array_elem_key.clone(),
                                                Assignment::from(value.clone()),
                                                state,
                                            )?;
                                        }
                                        Ok(())
                                    }
                                }
    
                            }
                            ArrayAssignment::Sliced { direct, others } => {
                                let mut ranges_assigned: Vec<&RangeConstraint> = vec![];
                                for ra in direct {
                                    let range = ra.constraint();
                                    if !range.is_between(to_array.high(), to_array.low())? {
                                        return Err(Error::InvalidArgument(format!(
                                            "{} is not between {} and {}",
                                            range,
                                            to_array.high(),
                                            to_array.low()
                                        )));
                                    }
                                    if ranges_assigned.iter().any(|x| x.overlaps(range)) {
                                        return Err(Error::InvalidArgument(format!("Sliced array assignment: {} overlaps with a range which was already assigned.", range)));
                                    }
                                    db.can_assign(
                                        to_array_elem_key.clone(),
                                        Assignment::from(ra.assignment().clone()),
                                        state,
                                    )?;
                                    ranges_assigned.push(range);
                                }
                                let total_assigned: u32 = ranges_assigned
                                    .iter()
                                    .map(|x| x.width_u32().unwrap_or(0)) // TODO: This unwrap probably isn't entirely correct
                                    .sum();
                                    if let Some(w) = to_array.width()? {
                                        if total_assigned == w {
                                    if let Some(_) = others {
                                        return Err(Error::InvalidArgument("Sliced array assignment contains an 'others' field, but already assigns all fields directly.".to_string()));
                                    } else {
                                        Ok(())
                                    }
                                } else {
                                    if let Some(value) = others {
                                        db.can_assign(
                                            to_array_elem_key,
                                            Assignment::from(value.as_ref().clone()),
                                            state,
                                        )
                                    } else {
                                        Err(Error::InvalidArgument("Sliced array assignment does not assign all values directly, but does not contain an 'others' field.".to_string()))
                                    }
                                }
                                    } else {
                                        // TODO: There's probably a check I can do here
                                        Ok(())
                                    }
                                
                            }
                            ArrayAssignment::Others(others) => db.can_assign(
                                to_array_elem_key,
                                Assignment::from(others.as_ref().clone()),
                                state,
                            ),
                        }
                    } else {
                        Err(Error::InvalidTarget(format!(
                            "Cannot perform full Array assignment to {}",
                            to_typ
                        )))
                    }
                }
            }
        }
    }
}
