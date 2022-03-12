use tydi_common::{
    insertion_ordered_map::InsertionOrderedMap,
    name::{Name, PathName},
    numbers::{NonNegative, Positive},
};

use crate::common::physical::complexity::Complexity;

use super::logicaltype::stream::Direction;

pub mod transfer_mode;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BitsReference<F: Clone + PartialEq> {
    pub bits: Positive,
    pub field: F,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnionReference<F: Clone + PartialEq> {
    pub tag: F,
    pub union: InsertionOrderedMap<Name, TypeReference<F>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TransferScope {
    /// This Stream exists on the root of an Interface, or is Desynchronized
    /// from its parent.
    ///
    /// Transfers on this stream define the transfer scope of synchronized child
    /// streams.
    Root,
    /// This Stream is synchronized with its parent Stream.
    ///
    /// _Exactly_ one transfer must occur on this stream per transfer on its
    /// parent Stream.
    ///
    /// In effect:
    /// 1. Once a transfer has occurred on its parent Stream, a transfer _must_
    /// occur on this Stream prior to the next transfer on the parent Stream.
    /// 2. Vice versa, if a transfer has occurred
    /// on this Stream, a transfer _must_ occur on its parent.
    ///
    /// Note that a Stream being synchronized does not prevent it from being the
    /// parent to further child Streams. As a result, it also defines its own
    /// transfer scope.
    Sync(PathName),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StreamElementReference<F: Clone + PartialEq> {
    pub element: ElementManipulatingReference<F>,
    pub last: Option<F>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// The signals which indicate whether data is being transfered on a given
/// element lane.
pub struct ElementActivityReference<F: Clone + PartialEq> {
    pub strb: Option<F>,
    pub stai: Option<F>,
    pub endi: Option<F>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StreamReference<F: Clone + PartialEq> {
    pub physical_stream: PathName,
    pub direction: Direction,
    pub complexity: Complexity,
    pub dimensionality: NonNegative,
    pub transfer_scope: TransferScope,
    pub activity: ElementActivityReference<F>,
    pub elements: Vec<StreamElementReference<F>>,
    pub user: ElementManipulatingReference<F>,
}

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
