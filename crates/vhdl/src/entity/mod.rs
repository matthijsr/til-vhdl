use tydi_common::map::InsertionOrderedMap;

use crate::{
    common::vhdl_name::VhdlName,
    port::{GenericParameter, Port},
};

mod impls;

/// An Entity.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Entity {
    /// Component identifier.
    identifier: VhdlName,
    /// The parameters of the entity..
    parameters: InsertionOrderedMap<VhdlName, GenericParameter>,
    /// The ports of the entity.
    ports: InsertionOrderedMap<VhdlName, Port>,
    /// Documentation.
    doc: Option<String>,
}
