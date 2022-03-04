use super::{interner::InternerStorage, ArchStorage};

#[salsa::database(ArchStorage, InternerStorage)]
#[derive(Default)]
pub struct Database {
    storage: salsa::Storage<Database>,
}

impl salsa::Database for Database {}
