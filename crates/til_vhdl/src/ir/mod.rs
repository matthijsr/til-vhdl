use crate::common::logical;
use tydi_intern::Id;

pub use connection::Connection;
pub mod connection;
pub use implementation::Implementation;
pub mod implementation;
pub use physical_properties::PhysicalProperties;
pub mod physical_properties;
pub use port::Port;
pub mod port;
pub use streamlet::Streamlet;
pub mod streamlet;
pub use db::Database;
pub mod db;

/// List of all the nodes
pub type LogicalType = logical::logicaltype::LogicalType;
pub type Stream = logical::logicaltype::Stream;
pub type Name = tydi_common::name::Name;
pub type Field = logical::logicaltype::Field;
pub type Identifier = Vec<Name>;

#[salsa::query_group(IrStorage)]
pub trait Ir {
    #[salsa::interned]
    fn intern_connection(&self, connection: Connection) -> Id<Connection>;
    #[salsa::interned]
    fn intern_field(&self, field: Field) -> Id<Field>;
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
