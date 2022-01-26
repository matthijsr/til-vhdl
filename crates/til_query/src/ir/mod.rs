use crate::common::logical::logicaltype::{stream::Stream, LogicalType};
use tydi_common::{error::{Result, TryResult}, name::Name};
use tydi_intern::Id;

use self::{
    implementation::Implementation,
    interface::Interface,
    project::{namespace::Namespace, Project},
    streamlet::Streamlet,
};

pub mod annotation_keys;
pub mod connection;
pub mod db;
pub mod get_self;
pub mod implementation;
pub mod interface;
pub mod intern_self;
pub mod physical_properties;
pub mod project;
pub mod streamlet;

#[salsa::query_group(IrStorage)]
pub trait Ir {
    #[salsa::input]
    fn annotation(&self, intern_id: salsa::InternId, key: String) -> String;

    #[salsa::input]
    fn project(&self) -> Project;

    #[salsa::interned]
    fn intern_namespace(&self, namespace: Namespace) -> Id<Namespace>;
    #[salsa::interned]
    fn intern_implementation(&self, implementation: Implementation) -> Id<Implementation>;
    #[salsa::interned]
    fn intern_type(&self, logical_type: LogicalType) -> Id<LogicalType>;
    #[salsa::interned]
    fn intern_port(&self, logical_type: Interface) -> Id<Interface>;
    #[salsa::interned]
    fn intern_stream(&self, stream: Stream) -> Id<Stream>;
    #[salsa::interned]
    fn intern_streamlet(&self, streamlet: Streamlet) -> Id<Streamlet>;
}

pub trait GetSelf<T> {
    fn get(&self, db: &dyn Ir) -> T;
}

pub trait InternSelf: Sized {
    fn intern(self, db: &dyn Ir) -> Id<Self>;
}

pub trait TryIntern<T> {
    fn try_intern(self, db: &dyn Ir) -> Result<Id<T>>;
}

pub trait MoveDb<T>: Sized {
    /// Move (parts) of an object from one database to another.
    /// The prefix parameter can be used to prefix names with the name of the original project, to avoid conflicts.
    fn move_db(&self, original_db: &dyn Ir, target_db: &dyn Ir, prefix: &Option<Name>) -> Result<T>;
}

impl<T> MoveDb<Id<T>> for Id<T>
where
    Id<T>: GetSelf<T>,
    T: MoveDb<Id<T>>,
{
    fn move_db(&self, original_db: &dyn Ir, target_db: &dyn Ir, prefix: &Option<Name>) -> Result<Id<T>> {
        self.get(original_db).move_db(original_db, target_db, prefix)
    }
}

impl<T, U> TryIntern<T> for U
where
    U: TryResult<T>,
    T: InternSelf,
{
    fn try_intern(self, db: &dyn Ir) -> Result<Id<T>> {
        Ok(self.try_result()?.intern(db))
    }
}

#[cfg(test)]
mod tests {
    use crate::ir::db::Database;

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
