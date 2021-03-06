use std::sync::Arc;

use tydi_intern::Id;

use super::{
    project::{interface::Interface, namespace::Namespace},
    traits::InternSelf,
    Implementation, Ir, LogicalType, Stream, Streamlet,
};

// This will almost certainly lead to bad design, so comment it out for now unless I can think of a valid use.
//
// impl<T> InternSelf {
//     fn intern(self, db: &dyn super::Ir) -> Id<Self> {
//         self.clone().intern(db)
//     }
// }

impl InternSelf for Implementation {
    fn intern(self, db: &dyn Ir) -> Id<Self> {
        db.intern_implementation(self)
    }
}

impl InternSelf for LogicalType {
    fn intern(self, db: &dyn Ir) -> Id<Self> {
        db.intern_type(self)
    }
}

impl InternSelf for Arc<Streamlet> {
    fn intern(self, db: &dyn Ir) -> Id<Self> {
        db.intern_streamlet(self)
    }
}

impl InternSelf for Stream {
    fn intern(self, db: &dyn Ir) -> Id<Self> {
        db.intern_stream(self)
    }
}

impl InternSelf for Namespace {
    fn intern(self, db: &dyn Ir) -> Id<Self> {
        db.intern_namespace(self)
    }
}

impl InternSelf for Arc<Interface> {
    fn intern(self, db: &dyn Ir) -> Id<Self> {
        db.intern_interface(self)
    }
}
