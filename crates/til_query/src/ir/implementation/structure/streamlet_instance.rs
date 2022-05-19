use tydi_common::{name::{Name, NameSelf}, traits::Identify, map::InsertionOrderedMap};
use tydi_intern::Id;

use crate::ir::{streamlet::Streamlet, physical_properties::Domain};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ListOrDefault {
    /// Definition has a list of Domains, and is assigned either a Default
    /// Domain (None), or a named Domain (Some).
    List(InsertionOrderedMap<Domain, Option<Domain>>),
    /// Definition has a Default Domain, and is assigned either a Default
    /// Domain (None), or a named Domain (Some).
    Default(Option<Domain>),
}


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StreamletInstance {
    name: Name,
    definition: Id<Streamlet>,
    domain_assignments: ListOrDefault,
}

impl Identify for StreamletInstance {
    fn identifier(&self) -> String {
        self.name.to_string()
    }
}

impl NameSelf for StreamletInstance {
    fn name(&self) -> &Name {
        &self.name
    }
}