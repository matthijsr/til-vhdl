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
use tydi_vhdl::architecture::arch_storage::Arch;
pub mod db;

pub mod get_self;
pub mod intern_self;

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

pub trait GetSelf<T> {
    fn get(&self, db: &dyn Ir) -> T;
}

pub trait InternSelf<T> {
    fn intern(self, db: &dyn Ir) -> Id<T>;
}

pub trait IntoVhdl<T> {
    fn into_vhdl(&self, ir_db: &dyn Ir, vhdl_db: &dyn Arch) -> T;
}

#[cfg(test)]
mod tests {
    use super::*;
    use tydi_common::error::Result;

    // Want to make sure interning works as I expect it to (identical objects get same ID)
    #[test]
    fn verify_intern_id() -> Result<()> {
        let _db = Database::default();
        let db = &_db;
        let id1 = db.intern_type(LogicalType::try_new_bits(8)?);
        let id2 = db.intern_type(LogicalType::try_new_bits(8)?);
        assert_eq!(id1, id2);
        Ok(())
    }
}
