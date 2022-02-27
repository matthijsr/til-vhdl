use indexmap::IndexMap;
use tydi_common::{name::PathName};

use crate::{
    common::physical::{fields::Fields},
    ir::Ir,
};

#[derive(Debug, Clone, PartialEq)]
pub struct LogicalStream<F, P> {
    signals: Fields<F>,
    streams: IndexMap<PathName, P>,
}

impl<F, P> LogicalStream<F, P> {
    pub fn new(signals: Fields<F>, streams: IndexMap<PathName, P>) -> Self {
        LogicalStream { signals, streams }
    }

    pub fn signals(&self) -> impl Iterator<Item = (&PathName, &F)> {
        self.signals.iter()
    }

    pub fn streams(&self) -> impl Iterator<Item = (&PathName, &P)> {
        self.streams.iter()
    }
}

pub trait SynthesizeLogicalStream<F, P> {
    fn synthesize(&self, db: &dyn Ir) -> LogicalStream<F, P>;
}
