use std::sync::Arc;

use tydi_common::{
    error::{Error, Result},
    traits::Identify,
};
use tydi_intern::Id;

use crate::{
    common::vhdl_name::VhdlName,
    component::Component,
    declaration::{ObjectDeclaration, ObjectKind},
    package::Package,
    port::Mode,
    statement::relation::Relation,
};

use self::{
    interner::{GetSelf, Interner},
    object_queries::ObjectQueries,
};

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

    fn can_assign(&self, to: ObjectKey, assignment: Assignment) -> Result<()>;

    fn can_map<'a>(&self, target: Id<ObjectDeclaration>, assignment: AssignmentKind) -> Result<()>;
}

fn subject_component(db: &dyn Arch) -> Result<Arc<Component>> {
    let package = db.default_package();
    package.get_subject_component(db)
}

fn can_assign(db: &dyn Arch, to: ObjectKey, assignment: Assignment) -> Result<()> {
    let to_key = to.with_nested(assignment.to_field().clone());
    let to = db.get_object(to_key.clone())?;
    to.assignable.to_or_err()?;
    let to_typ = db.lookup_intern_object_type(to.typ);
    match assignment.kind() {
        AssignmentKind::Relation(relation) => relation.can_assign(db, &to_typ),
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
                            FieldSelection::Range(RangeConstraint::Index(to_array.high())),
                        );
                        match array {
                            ArrayAssignment::Direct(direct) => {
                                if to_array.width() == direct.len().try_into().unwrap() {
                                    for value in direct {
                                        db.can_assign(
                                            to_array_elem_key.clone(),
                                            Assignment::from(value.clone()),
                                        )?;
                                    }
                                    Ok(())
                                } else {
                                    Err(Error::InvalidArgument(format!("Attempted full array assignment. Number of fields do not match. Array has {} fields, assignment has {} fields", to_array.width(), direct.len())))
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
                                    )?;
                                    ranges_assigned.push(range);
                                }
                                let total_assigned: u32 =
                                    ranges_assigned.iter().map(|x| x.width_u32()).sum();
                                if total_assigned == to_array.width() {
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
                                        )
                                    } else {
                                        Err(Error::InvalidArgument("Sliced array assignment does not assign all values directly, but does not contain an 'others' field.".to_string()))
                                    }
                                }
                            }
                            ArrayAssignment::Others(others) => db.can_assign(
                                to_array_elem_key,
                                Assignment::from(others.as_ref().clone()),
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

fn can_map(
    db: &dyn Arch,
    target: Id<ObjectDeclaration>,
    assignment_kind: AssignmentKind,
) -> Result<()> {
    let target_obj = target.get(db);
    fn is_explicit_out(object_kind: &ObjectKind) -> Result<bool> {
        match object_kind {
            ObjectKind::Signal
            | ObjectKind::Variable
            | ObjectKind::Constant
            | ObjectKind::ComponentPort(Mode::In) => Ok(false),
            ObjectKind::ComponentPort(Mode::Out) => Ok(true),
            ObjectKind::EntityPort(_) => Err(Error::BackEndError(
                "Entity ports should not be included in mappings".to_string(),
            )),
            ObjectKind::Alias(_, alias_kind) => is_explicit_out(alias_kind),
        }
    }
    fn is_explicit_in(object_kind: &ObjectKind) -> Result<bool> {
        match object_kind {
            ObjectKind::Variable | ObjectKind::Constant | ObjectKind::ComponentPort(Mode::In) => {
                Ok(true)
            }
            ObjectKind::Signal | ObjectKind::ComponentPort(Mode::Out) => Ok(false),
            ObjectKind::EntityPort(_) => Err(Error::BackEndError(
                "Entity ports should not be included in mappings".to_string(),
            )),
            ObjectKind::Alias(_, alias_kind) => is_explicit_in(alias_kind),
        }
    }
    match assignment_kind {
        AssignmentKind::Relation(relation) => {
            if is_explicit_out(target_obj.kind())? {
                if let Relation::Object(o) = relation {
                    let obj = db.get_object(o.as_object_key(db))?;
                    obj.assignable.to_or_err()?;
                    obj.typ(db).can_assign_type(&target_obj.typ(db)?)
                } else {
                    Err(Error::InvalidTarget(format!(
                            "Cannot map an output object ({}) to a non-object relation",
                            target_obj.identifier()
                        )))
                }
            } else {
                Ok(())
            }
        },
        AssignmentKind::Direct(_) => Err(Error::BackEndError(
            "Backend does not currently support direct assignment (full arrays or records) in mappings".to_string(),
        )),
    }
}
