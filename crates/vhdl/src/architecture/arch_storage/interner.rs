use tydi_common::error;
use tydi_common::error::TryResult;
use tydi_intern::Id;

use crate::architecture::arch_storage::Arch;
use crate::object::object_type::ObjectType;
use crate::{
    declaration::{ArchitectureDeclaration, ObjectDeclaration},
    object::Object,
};

#[salsa::query_group(InternerStorage)]
pub trait Interner {
    #[salsa::interned]
    fn intern_architecture_declaration(
        &self,
        arch_decl: ArchitectureDeclaration,
    ) -> Id<ArchitectureDeclaration>;

    #[salsa::interned]
    fn intern_object_declaration(&self, obj_decl: ObjectDeclaration) -> Id<ObjectDeclaration>;

    #[salsa::interned]
    fn intern_object(&self, obj: Object) -> Id<Object>;

    #[salsa::interned]
    fn intern_object_type(&self, object_type: ObjectType) -> Id<ObjectType>;
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
    fn try_intern(self, db: &dyn Arch) -> error::Result<Id<T>>;
}

pub trait TryInternAs<T> {
    fn try_intern_as(self, db: &dyn Arch) -> error::Result<Id<T>>;
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
    fn try_intern(self, db: &dyn Arch) -> error::Result<Id<T>> {
        Ok(self.try_result()?.intern(db))
    }
}
