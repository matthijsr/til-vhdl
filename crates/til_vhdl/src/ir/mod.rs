use crate::common::{logical, name};
use tydi_intern::Id;

pub(crate) use connection::Connection;
pub mod connection;
pub(crate) use implementation::Implementation;
pub mod implementation;
pub(crate) use physical_properties::PhysicalProperties;
pub mod physical_properties;
pub(crate) use port::Port;
pub mod port;
pub(crate) use streamlet::Streamlet;
pub mod streamlet;

/// List of all the nodes
pub(crate) type LogicalType = logical::logicaltype::LogicalType;
pub(crate) type Stream = logical::logicaltype::Stream;
pub(crate) type Name = name::Name;
pub(crate) type Field = logical::logicaltype::Field;
pub(crate) type Identifier = Vec<Name>;

#[salsa::query_group(IrStorage)]
pub(crate) trait Ir {
    #[salsa::interned]
    fn intern_connection(&self, connection: Connection) -> Id<Connection>;
    #[salsa::interned]
    fn intern_field(&self, field: Field) -> Id<Field>;
    #[salsa::interned]
    fn intern_identifier(&self, identifier: Identifier) -> Id<Identifier>;
    #[salsa::interned]
    fn intern_implementation(&self, implementation: Implementation) -> Id<Implementation>;
    #[salsa::interned]
    fn intern_type(&self, logical_type: LogicalType) -> Id<LogicalType>;
    #[salsa::interned]
    fn intern_port(&self, logical_type: Port) -> Id<Port>;
    #[salsa::interned]
    fn intern_stream(&self, stream: Stream) -> Id<Stream>;
    #[salsa::interned]
    fn intern_streamlet(&self, streamlet: Streamlet) -> Id<Streamlet>;
}
