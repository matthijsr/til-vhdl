use std::sync::Arc;

use tydi_intern::Id;

use super::{
    project::{interface::Interface, namespace::Namespace},
    traits::GetSelf,
    Implementation, Ir, LogicalType, Stream, Streamlet,
};

impl GetSelf<Implementation> for Id<Implementation> {
    fn get(&self, db: &dyn Ir) -> Implementation {
        db.lookup_intern_implementation(*self)
    }
}

impl GetSelf<LogicalType> for Id<LogicalType> {
    fn get(&self, db: &dyn Ir) -> LogicalType {
        db.lookup_intern_type(*self)
    }
}

impl GetSelf<Streamlet> for Id<Streamlet> {
    fn get(&self, db: &dyn Ir) -> Streamlet {
        db.lookup_intern_streamlet(*self)
    }
}

impl GetSelf<Stream> for Id<Stream> {
    fn get(&self, db: &dyn Ir) -> Stream {
        db.lookup_intern_stream(*self)
    }
}

impl GetSelf<Namespace> for Id<Namespace> {
    fn get(&self, db: &dyn Ir) -> Namespace {
        db.lookup_intern_namespace(*self)
    }
}

impl GetSelf<Arc<Interface>> for Id<Arc<Interface>> {
    fn get(&self, db: &dyn Ir) -> Arc<Interface> {
        db.lookup_intern_interface(*self)
    }
}
