use tydi_common::{
    insertion_ordered_map::InsertionOrderedMap,
    name::{Name, PathName},
};

use super::TypeReference;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScopeStream<F: Clone + PartialEq> {
    pub name: PathName,
    pub children: InsertionOrderedMap<Name, TypeReference<F>>,
}
