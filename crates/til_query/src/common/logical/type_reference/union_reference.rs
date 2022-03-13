use tydi_common::{insertion_ordered_map::InsertionOrderedMap, name::Name};

use super::TypeReference;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnionReference<F: Clone + PartialEq> {
    pub union: InsertionOrderedMap<Name, TypeReference<F>>,
    pub tag: F,
}
