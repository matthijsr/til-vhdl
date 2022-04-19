use crate::{
    architecture::arch_storage::Arch,
    usings::{ListUsings, ListUsingsDb, Usings},
};
use tydi_common::error::Result;
use tydi_intern::Id;

use super::ObjectDeclaration;

impl ListUsings for ObjectDeclaration {
    fn list_usings(&self) -> Result<Usings> {
        match self.default() {
            Some(ak) => ak.list_usings(),
            None => Ok(Usings::new_empty()),
        }
    }
}

impl ListUsingsDb for Id<ObjectDeclaration> {
    fn list_usings_db(&self, db: &dyn Arch) -> Result<Usings> {
        db.lookup_intern_object_declaration(*self).list_usings()
    }
}
