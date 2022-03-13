use tydi_common::{
    insertion_ordered_map::InsertionOrderedMap,
    name::{Name, PathName},
};
use tydi_intern::Id;

use crate::ir::Ir;

use self::{
    bits_reference::BitsReference, scope_stream::ScopeStream, stream_reference::StreamReference,
    union_reference::UnionReference,
};

use super::logicaltype::{stream::Stream, LogicalType};

pub mod bits_reference;
pub mod elements;
pub mod scope_stream;
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
    /// A Stream type refers to a physical stream and its relevant signals.
    Stream(StreamReference<F>),
    /// Scope Streams refer to Streams which have been flattened into their
    /// children, they are scopes which exists in name only. Scope Streams are
    /// treated as synchronous by default, as their children should be
    /// independently marked as desynchronized.
    ScopeStream(ScopeStream<F>),
}

impl<F: Clone + PartialEq> TypeReference<F> {
    /// Attempt to find the root Stream.
    /// Create a `ScopeStream` if it does not exist.
    ///
    /// Returns None if there are no more Streams to iterate over.
    pub fn collect_root<'a>(
        db: &dyn Ir,
        root_name: &PathName,
        mut streams: impl Iterator<Item = (&'a PathName, &'a Id<Stream>)>,
    ) -> Option<Self> {
        if let Some((stream_name, stream_id)) = streams.next() {
            if stream_name == root_name {
                // Create a Stream
            } else {
                // Create a ScopeStream, add all Streams which start with its
                // name as children.

                // If the child names are more than one Name longer than the
                // root, recursively create further scopes based on these new
                // roots.
            }
            todo!()
        } else {
            None
        }
    }

    pub fn collect_references<'a>(
        db: &dyn Ir,
        parent: &PathName,
        logical_type: Id<LogicalType>,
        mut streams: impl Iterator<Item = (&'a PathName, &'a Id<Stream>)>,
    ) -> Self {
        todo!()
    }
}
