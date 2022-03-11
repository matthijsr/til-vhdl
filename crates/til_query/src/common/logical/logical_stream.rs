use indexmap::IndexMap;
use tydi_common::{error::Result, name::PathName};

use crate::{common::physical::fields::Fields, ir::Ir};

#[derive(Debug, Clone, PartialEq)]
pub struct LogicalStream<F: Clone + PartialEq, P: Clone + PartialEq> {
    /// User-defined fields
    fields: Fields<F>,
    /// Streams adhering to the Tydi specification
    streams: IndexMap<PathName, P>,
}

impl<F: Clone + PartialEq, P: Clone + PartialEq> LogicalStream<F, P> {
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

    /// Create a new LogicalStreams with just the fields mapped to a new type
    pub fn map_fields<M, R: Clone + PartialEq>(&self, f: M) -> LogicalStream<R, P>
    where
        M: FnMut(F) -> R,
    {
        LogicalStream::new(self.fields.clone().map(f), self.streams.clone())
    }

    /// Create a new LogicalStreams with just the streams mapped to a new type
    pub fn map_streams<M, R: Clone + PartialEq>(&self, mut f: M) -> LogicalStream<F, R>
    where
        M: FnMut(P) -> R,
    {
        LogicalStream::new(
            self.fields.clone(),
            self.streams
                .clone()
                .into_iter()
                .map(|(n, p)| (n, f(p)))
                .collect(),
        )
    }

    pub fn map<MF, MP, RF: Clone + PartialEq, RP: Clone + PartialEq>(
        self,
        mf: MF,
        mut mp: MP,
    ) -> LogicalStream<RF, RP>
    where
        MF: FnMut(F) -> RF,
        MP: FnMut(P) -> RP,
    {
        let fields = self.fields.map(mf);
        let streams = self.streams.into_iter().map(|(n, p)| (n, mp(p))).collect();
        LogicalStream::new(fields, streams)
    }
}

pub trait SynthesizeLogicalStream<F: Clone + PartialEq, P: Clone + PartialEq> {
    fn synthesize(&self, db: &dyn Ir) -> Result<LogicalStream<F, P>>;
}
