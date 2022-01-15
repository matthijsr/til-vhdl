use std::sync::Arc;

use tydi_common::error::{Error, Result, TryResult};
use tydi_intern::Id;

use crate::{
    assignment::AssignmentKind,
    declaration::{ArchitectureDeclaration, ObjectDeclaration, ObjectMode, ObjectState},
    object::ObjectType,
    package::Package,
    statement::Statement,
};

use super::Architecture;

pub mod db;
pub mod get_self;
pub mod intern_self;

#[salsa::query_group(ArchStorage)]
pub trait Arch {
    #[salsa::input]
    fn default_package(&self) -> Arc<Package>;

    #[salsa::input]
    fn architecture(&self) -> Arc<Architecture>;

    #[salsa::interned]
    fn intern_architecture_declaration(
        &self,
        arch_decl: ArchitectureDeclaration,
    ) -> Id<ArchitectureDeclaration>;

    #[salsa::interned]
    fn intern_object_declaration(&self, obj_decl: ObjectDeclaration) -> Id<ObjectDeclaration>;

    fn get_object(&self, id: Id<ObjectDeclaration>) -> Result<ObjectDeclaration>;

    // #[salsa::interned]
    // fn intern_statement(&self, stat: Statement) -> Id<Statement>;
}

fn get_object(db: &dyn Arch, id: Id<ObjectDeclaration>) -> Result<ObjectDeclaration> {
    let mut obj = db.lookup_intern_object_declaration(id);
    for stat in db.architecture().statements() {
        match stat {
            Statement::Assignment(ass) => {
                if ass.object() == id {
                    if ass.assignment().to_field().is_empty() {
                        match ass.assignment().kind() {
                            AssignmentKind::Object(oa) => {
                                obj.set_state(oa.selected_object(db)?.state())?
                            }
                            AssignmentKind::Direct(_) => obj.set_state(ObjectState::Assigned)?,
                        }
                    } else {
                        todo!()
                        // Need to be able to keep track of individual fields, or collections of fields... which is very hard (consider that an array consists of multiple fields, but can also be assigned in slices)
                        // Should use a similar function to this one to track such things, rather than give them all individual IDs...
                    }
                }
            }
            Statement::PortMapping(pm) => {
                // TODO: Currently port mappings assume that ports are assigned completely, rather than assigning individual fields...
                if pm
                    .mappings()
                    .values()
                    .any(|x| x.object() == id && x.assignment().to_field().is_empty())
                {
                    obj.set_state(ObjectState::Assigned)?
                }
            }
        }
    }
    Ok(obj)
}

pub trait GetSelf<T> {
    fn get(&self, db: &dyn Arch) -> T;
}

pub trait InternSelf: Sized {
    fn intern(self, db: &dyn Arch) -> Id<Self>;
}

pub trait InternAs<T> {
    fn intern_as(self, db: &dyn Arch) -> Id<T>;
}

pub trait TryIntern<T> {
    fn try_intern(self, db: &dyn Arch) -> Result<Id<T>>;
}

pub trait TryInternAs<T> {
    fn try_intern_as(self, db: &dyn Arch) -> Result<Id<T>>;
}

impl<T, U> InternAs<T> for U
where
    U: Into<T>,
    T: InternSelf,
{
    fn intern_as(self, db: &dyn Arch) -> Id<T> {
        self.into().intern(db)
    }
}

impl<T, U> TryIntern<T> for U
where
    U: TryResult<T>,
    T: InternSelf,
{
    fn try_intern(self, db: &dyn Arch) -> Result<Id<T>> {
        Ok(self.try_result()?.intern(db))
    }
}
