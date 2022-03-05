use std::sync::Arc;

use tydi_common::error::{Result, TryResult};
use tydi_intern::Id;

use crate::{
    common::vhdl_name::VhdlName,
    component::Component,
    package::Package,
};

use self::interner::Interner;

use super::Architecture;

pub mod db;
pub mod get_self;
pub mod intern_self;
pub mod interner;

#[salsa::query_group(ArchStorage)]
pub trait Arch: Interner {
    #[salsa::input]
    fn default_package(&self) -> Arc<Package>;

    #[salsa::input]
    fn subject_component_name(&self) -> Arc<VhdlName>;

    #[salsa::input]
    fn architecture(&self) -> Architecture;

    fn subject_component(&self) -> Result<Arc<Component>>;
}

fn subject_component(db: &dyn Arch) -> Result<Arc<Component>> {
    let package = db.default_package();
    package.get_subject_component(db)
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
