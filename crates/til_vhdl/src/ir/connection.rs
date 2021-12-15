use tydi_intern::Id;

use super::Port;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Connection {
    source: Id<Port>,
    sink: Id<Port>,
}
