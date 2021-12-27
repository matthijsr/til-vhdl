use tydi_intern::Id;

use super::{Connection, Field, InternSelf, Implementation, LogicalType, Port, Stream, Streamlet};

impl InternSelf<Connection> for Connection {
    fn intern(self, db: &dyn super::Ir) -> Id<Connection> {
        db.intern_connection(self)
    }
}

impl InternSelf<Field> for Field {
    fn intern(self, db: &dyn super::Ir) -> Id<Field> {
        db.intern_field(self)
    }
}

impl InternSelf<Implementation> for Implementation {
    fn intern(self, db: &dyn super::Ir) -> Id<Implementation> {
        db.intern_implementation(self)
    }
}

impl InternSelf<LogicalType> for LogicalType {
    fn intern(self, db: &dyn super::Ir) -> Id<LogicalType> {
        db.intern_type(self)
    }
}

impl InternSelf<Port> for Port {
    fn intern(self, db: &dyn super::Ir) -> Id<Port> {
        db.intern_port(self)
    }
}

impl InternSelf<Streamlet> for Streamlet {
    fn intern(self, db: &dyn super::Ir) -> Id<Streamlet> {
        db.intern_streamlet(self)
    }
}

impl InternSelf<Stream> for Stream {
    fn intern(self, db: &dyn super::Ir) -> Id<Stream> {
        db.intern_stream(self)
    }
}
