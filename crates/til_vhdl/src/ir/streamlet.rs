use tydi_intern::Id;

use super::{Implementation, Port};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Streamlet {
    implementation: Id<Implementation>,
    ports: Vec<Id<Port>>,
}
