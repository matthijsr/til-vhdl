use tydi_common::{error::Result, map::InsertionOrderedMap, name::PathName};

use crate::ir::Ir;

use super::type_reference::TypeReference;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LogicalStream<F: Clone + PartialEq, P: Clone + PartialEq> {
    /// User-defined fields
    fields: InsertionOrderedMap<PathName, F>,
    /// Streams adhering to the Tydi specification
    streams: InsertionOrderedMap<PathName, P>,
}

impl<F: Clone + PartialEq, P: Clone + PartialEq> LogicalStream<F, P> {
    pub fn new(
        fields: InsertionOrderedMap<PathName, F>,
        streams: InsertionOrderedMap<PathName, P>,
    ) -> Self {
        LogicalStream { fields, streams }
    }

    pub fn fields(&self) -> &InsertionOrderedMap<PathName, F> {
        &self.fields
    }

    pub fn fields_iter(&self) -> impl Iterator<Item = (&PathName, &F)> {
        self.fields.iter()
    }

    pub fn streams(&self) -> &InsertionOrderedMap<PathName, P> {
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
        LogicalStream::new(self.fields.clone().map_convert(f), self.streams.clone())
    }

    /// Create a new LogicalStreams with just the streams mapped to a new type
    pub fn map_streams<M, R: Clone + PartialEq>(&self, f: M) -> LogicalStream<F, R>
    where
        M: FnMut(P) -> R,
    {
        LogicalStream::new(self.fields.clone(), self.streams.clone().map_convert(f))
    }

    pub fn map<MF, MP, RF: Clone + PartialEq, RP: Clone + PartialEq>(
        self,
        mf: MF,
        mp: MP,
    ) -> LogicalStream<RF, RP>
    where
        MF: FnMut(F) -> RF,
        MP: FnMut(P) -> RP,
    {
        let fields = self.fields.map_convert(mf);
        let streams = self.streams.map_convert(mp);
        LogicalStream::new(fields, streams)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypedStream<F: Clone + PartialEq, P: Clone + PartialEq> {
    logical_stream: LogicalStream<F, P>,
    type_reference: TypeReference,
}

impl<F: Clone + PartialEq, P: Clone + PartialEq> TypedStream<F, P> {
    pub fn new(logical_stream: LogicalStream<F, P>, type_reference: TypeReference) -> Self {
        TypedStream {
            logical_stream,
            type_reference,
        }
    }

    pub fn logical_stream(&self) -> &LogicalStream<F, P> {
        &self.logical_stream
    }

    pub fn type_reference(&self) -> &TypeReference {
        &self.type_reference
    }
}

pub trait SynthesizeLogicalStream<F: Clone + PartialEq, P: Clone + PartialEq> {
    fn synthesize(&self, db: &dyn Ir) -> Result<TypedStream<F, P>>;
}
