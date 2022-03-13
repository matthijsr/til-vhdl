use tydi_common::{insertion_ordered_map::InsertionOrderedMap, name::Name};

use self::{
    bits_reference::BitsReference, stream_reference::StreamReference,
    union_reference::UnionReference,
};

pub mod bits_reference;
pub mod elements;
pub mod stream_reference;
pub mod transfer_mode;
pub mod transfer_scope;
pub mod union_reference;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ElementManipulatingReference<F: Clone + PartialEq> {
    /// Null does not have any associated physical signals.
    Null,
    /// Bits(N) refers to N bits of a `data` signal
    Bits(BitsReference<F>),
    /// Group contains a number of distinct fields, which themselves refer to
    /// either element-manipulating nodes or Streams.
    Group(InsertionOrderedMap<Name, TypeReference<F>>),
    /// Union contains a number of overlapping fields, which themselves refer to
    /// either element-manipulating nodes or Streams, the active field is selected
    /// through the `tag` signal.
    ///
    /// Any element-manipulating nodes draw from a single, overlapped `union` signal.
    Union(UnionReference<F>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeReference<F: Clone + PartialEq> {
    /// Element-manipulating types should exclusively manipulate the `data` signal.
    ///
    /// However, they can still refer to Streams indirectly, thereby putting constraints
    /// on those Streams.
    ElementManipulating(ElementManipulatingReference<F>),
    /// A Stream type refers to a
    Stream(StreamReference<F>),
}
