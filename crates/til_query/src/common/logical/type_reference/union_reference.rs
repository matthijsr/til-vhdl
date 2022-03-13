use tydi_common::{insertion_ordered_map::InsertionOrderedMap, name::Name};

use super::TypeReference;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnionReference<F: Clone + PartialEq> {
    pub union_fields: InsertionOrderedMap<Name, TypeReference<F>>,
    pub union: Option<F>,
    pub tag: Option<F>,
}
