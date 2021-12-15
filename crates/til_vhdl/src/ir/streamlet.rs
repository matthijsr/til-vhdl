use tydi_intern::Id;

use super::{Implementation, Port};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Streamlet {
    implementation: Id<Implementation>,
    ports: Vec<Id<Port>>,
}
