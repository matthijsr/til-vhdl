use indexmap::IndexMap;
use tydi_common::name::PathName;
use tydi_intern::Id;

use crate::ir::Ir;

use super::logicaltype::{stream::Stream, LogicalType};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SplitStreams {
    signals: Id<LogicalType>,
    streams: IndexMap<PathName, Id<Stream>>,
}

impl SplitStreams {
    pub fn new(signals: Id<LogicalType>, streams: IndexMap<PathName, Id<Stream>>) -> Self {
        SplitStreams { signals, streams }
    }

    pub fn streams(&self) -> impl Iterator<Item = (&PathName, &Id<Stream>)> {
        self.streams.iter()
    }
    pub fn signals(&self) -> Id<LogicalType> {
        self.signals
    }
}

pub(crate) trait SplitsStreams {
    /// Splits a logical stream type into simplified stream types.
    ///
    /// [Reference](https://abs-tudelft.github.io/tydi/specification/logical.html#split-function)
    fn split_streams(&self, db: &dyn Ir) -> SplitStreams;
}
