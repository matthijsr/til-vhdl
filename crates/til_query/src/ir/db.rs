use super::{interner::InternerStorage, IrStorage};

#[salsa::database(IrStorage, InternerStorage)]
#[derive(Default)]
pub struct Database {
    storage: salsa::Storage<Database>,
}

impl salsa::Database for Database {}
