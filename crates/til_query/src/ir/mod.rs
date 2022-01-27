use std::sync::Arc;

use crate::common::logical::logicaltype::{stream::Stream, LogicalType};

use self::{
    implementation::Implementation, interface::Interface, interner::Interner, project::Project,
    streamlet::Streamlet, traits::GetSelf,
};

pub mod annotation_keys;
pub mod connection;
pub mod db;
pub mod get_self;
pub mod implementation;
pub mod interface;
pub mod intern_self;
pub mod interner;
pub mod physical_properties;
pub mod project;
pub mod streamlet;
pub mod traits;

#[salsa::query_group(IrStorage)]
pub trait Ir: Interner {
    #[salsa::input]
    fn annotation(&self, intern_id: salsa::InternId, key: String) -> String;

    #[salsa::input]
    fn project(&self) -> Arc<Project>;

    fn all_streamlets(&self) -> Arc<Vec<Streamlet>>;
}

fn all_streamlets(db: &dyn Ir) -> Arc<Vec<Streamlet>> {
    let project = db.project();

    Arc::new(
        project
            .namespaces()
            .iter()
            .map(|(_, id)| id.get(db))
            .map(|namespace| {
                namespace
                    .streamlets(db)
                    .into_iter()
                    .map(|(_, streamlet)| streamlet)
                    .collect::<Vec<Streamlet>>()
            })
            .flatten()
            .collect(),
    )
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
