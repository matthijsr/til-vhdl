use tydi_common::name::PathName;
use tydi_intern::Id;

use crate::ir::{Ir, Name};

use super::LogicalType;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LogicalField {
    /// The relative name of the field
    name: PathName,
    typ: Id<LogicalType>,
}

impl LogicalField {
    pub fn new(name: PathName, typ: Id<LogicalType>) -> LogicalField {
        LogicalField {
            name: name,
            typ: typ,
        }
    }

    pub fn name(&self) -> &PathName {
        &self.name
    }

    pub fn typ(&self, db: &dyn Ir) -> LogicalType {
        db.lookup_intern_type(self.typ)
    }
}
