#[salsa::database(super::ArchStorage)]
#[derive(Default)]
pub struct Database {
    storage: salsa::Storage<Database>,
}

impl salsa::Database for Database {}
