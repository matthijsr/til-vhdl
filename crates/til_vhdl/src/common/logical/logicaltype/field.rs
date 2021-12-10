use tydi_intern::Id;

use crate::ir::{Identifier, Ir, Name};

use super::LogicalType;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Field {
    /// The relative name of the field
    name: Id<Identifier>,
    typ: Id<LogicalType>,
}

impl Field {
    pub fn new(db: &dyn Ir, base_id: &Vec<Name>, name: Name, typ: Id<LogicalType>) -> Field {
        if base_id.is_empty() {
            Field {
                name: db.intern_identifier(vec![name]),
                typ: typ,
            }
        } else {
            let id = base_id.clone();
            id.push(name);
            Field {
                name: db.intern_identifier(id),
                typ: typ,
            }
        }
    }
}
