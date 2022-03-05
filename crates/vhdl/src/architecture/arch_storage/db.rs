use super::{interner::InternerStorage, object_queries::ObjectStorage, ArchStorage};

#[salsa::database(ArchStorage, InternerStorage, ObjectStorage)]
#[derive(Default)]
pub struct Database {
    storage: salsa::Storage<Database>,
}

impl salsa::Database for Database {}
