use tydi_common::name::PathName;

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
