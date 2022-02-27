use indexmap::IndexMap;
use tydi_common::name::PathName;

use crate::{common::physical::fields::Fields, ir::Ir};

#[derive(Debug, Clone, PartialEq)]
pub struct LogicalStream<F, P> {
    /// User-defined fields
    fields: Fields<F>,
    /// Streams adhering to the Tydi specification
    streams: IndexMap<PathName, P>,
}

impl<F, P> LogicalStream<F, P> {
    pub fn new(fields: Fields<F>, streams: IndexMap<PathName, P>) -> Self {
        LogicalStream { fields, streams }
    }

    pub fn fields(&self) -> &Fields<F> {
        &self.fields
    }

    pub fn fields_iter(&self) -> impl Iterator<Item = (&PathName, &F)> {
        self.fields.iter()
    }

    pub fn streams(&self) -> &IndexMap<PathName, P> {
        &self.streams
    }

    pub fn streams_iter(&self) -> impl Iterator<Item = (&PathName, &P)> {
        self.streams.iter()
    }
}

pub trait SynthesizeLogicalStream<F, P> {
    fn synthesize(&self, db: &dyn Ir) -> LogicalStream<F, P>;
}
