use indexmap::IndexMap;
use tydi_common::{name::PathName, numbers::BitCount};

use crate::{
    common::physical::{fields::Fields, stream::PhysicalStream},
    ir::Ir,
};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LogicalStream {
    signals: Fields,
    streams: IndexMap<PathName, PhysicalStream>,
}

impl LogicalStream {
    pub fn new(signals: Fields, streams: IndexMap<PathName, PhysicalStream>) -> Self {
        LogicalStream { signals, streams }
    }

    #[allow(dead_code)]
    pub fn signals(&self) -> impl Iterator<Item = (&PathName, &BitCount)> {
        self.signals.iter()
    }

    #[allow(dead_code)]
    pub fn streams(&self) -> impl Iterator<Item = (&PathName, &PhysicalStream)> {
        self.streams.iter()
    }
}

pub(crate) trait SynthesizeLogicalStream {
    fn synthesize(&self, db: &dyn Ir) -> LogicalStream;
}
