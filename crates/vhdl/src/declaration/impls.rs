use crate::usings::{ListUsings, Usings};
use tydi_common::error::Result;

use super::ObjectDeclaration;

impl ListUsings for ObjectDeclaration {
    fn list_usings(&self) -> Result<Usings> {
        match self.default() {
            Some(ak) => ak.list_usings(),
            None => Ok(Usings::new_empty()),
        }
    }
}
