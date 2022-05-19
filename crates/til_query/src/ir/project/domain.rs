use tydi_common::{map::InsertionOrderedMap, name::Name};

pub type Domain = Name;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ListOrDefault {
    List(InsertionOrderedMap<Domain, Option<Domain>>),
    Default(Option<Domain>),
}
