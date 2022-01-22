#[salsa::database(super::IrStorage)]
#[derive(Default)]
pub struct Database {
    storage: salsa::Storage<Database>,
}

impl salsa::Database for Database {}
