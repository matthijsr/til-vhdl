use tydi_intern::Id;

use super::Interface;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Connection {
    source: Id<Interface>,
    sink: Id<Interface>,
}
