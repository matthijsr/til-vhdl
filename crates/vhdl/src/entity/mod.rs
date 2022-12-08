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
    parameters: Vec<GenericParameter>,
    /// The ports of the entity.
    ports: Vec<Port>,
    /// Documentation.
    doc: Option<String>,
}
