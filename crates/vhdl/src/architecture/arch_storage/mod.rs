use std::sync::Arc;

use tydi_common::error::Result;

use crate::{common::vhdl_name::VhdlName, component::Component, package::Package};

use super::Architecture;

use self::{interner::Interner, object_queries::ObjectQueries};

pub mod db;
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

    #[salsa::input]
    fn architecture(&self) -> Architecture;

    fn subject_component(&self) -> Result<Arc<Component>>;
}

fn subject_component(db: &dyn Arch) -> Result<Arc<Component>> {
    let package = db.default_package();
    package.get_subject_component(db)
}
