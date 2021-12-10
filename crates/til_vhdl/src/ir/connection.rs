use tydi_intern::Id;

use super::Port;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Connection {
    source: Id<Port>,
    sink: Id<Port>,
}
