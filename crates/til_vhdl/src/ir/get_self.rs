use tydi_intern::Id;

use super::{Connection, Field, GetSelf, Implementation, LogicalType, Port, Stream, Streamlet};

impl GetSelf<Connection> for Id<Connection> {
    fn get(&self, db: &dyn super::Ir) -> Connection {
        db.lookup_intern_connection(*self)
    }
}

impl GetSelf<Field> for Id<Field> {
    fn get(&self, db: &dyn super::Ir) -> Field {
        db.lookup_intern_field(*self)
    }
}

impl GetSelf<Implementation> for Id<Implementation> {
    fn get(&self, db: &dyn super::Ir) -> Implementation {
        db.lookup_intern_implementation(*self)
    }
}

impl GetSelf<LogicalType> for Id<LogicalType> {
    fn get(&self, db: &dyn super::Ir) -> LogicalType {
        db.lookup_intern_type(*self)
    }
}

impl GetSelf<Port> for Id<Port> {
    fn get(&self, db: &dyn super::Ir) -> Port {
        db.lookup_intern_port(*self)
    }
}

impl GetSelf<Streamlet> for Id<Streamlet> {
    fn get(&self, db: &dyn super::Ir) -> Streamlet {
        db.lookup_intern_streamlet(*self)
    }
}

impl GetSelf<Stream> for Id<Stream> {
    fn get(&self, db: &dyn super::Ir) -> Stream {
        db.lookup_intern_stream(*self)
    }
}
