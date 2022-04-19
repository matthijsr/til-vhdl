use object_type::ObjectType;
use tydi_common::error::{Error, Result, TryResult};
use tydi_intern::Id;

use crate::{
    architecture::arch_storage::{
        interner::TryIntern, object_queries::object_key::ObjectKey, Arch,
    },
    declaration::ObjectKind,
    port::Mode,
};

pub mod array;
pub mod object_from;
pub mod object_type;
pub mod record;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub struct Assignable {
    /// Can be assigned to
    pub to: bool,
    /// Can be assigned from
    pub from: bool,
}

impl Assignable {
    /// If `to` is false, returns an error
    pub fn to_or_err(&self) -> Result<()> {
        if self.to {
            Ok(())
        } else {
            Err(Error::InvalidTarget(
                "The selected object cannot be assigned to".to_string(),
            ))
        }
    }

    /// If `from` is false, returns an error
    pub fn from_or_err(&self) -> Result<()> {
        if self.from {
            Ok(())
        } else {
            Err(Error::InvalidTarget(
                "The selected object cannot be assigned from".to_string(),
            ))
        }
    }
}

impl From<&ObjectKind> for Assignable {
    fn from(kind: &ObjectKind) -> Self {
        match kind {
            ObjectKind::Signal => Assignable {
                to: true,
                from: true,
            },
            ObjectKind::Variable => Assignable {
                to: true,
                from: true,
            },
            ObjectKind::Constant => Assignable {
                to: false,
                from: true,
            },
            ObjectKind::EntityPort(mode) => match mode {
                Mode::In => Assignable {
                    to: false,
                    from: true,
                },
                Mode::Out => Assignable {
                    to: true,
                    from: false,
                },
            },
            ObjectKind::ComponentPort(mode) => match mode {
                Mode::In => Assignable {
                    to: true,
                    from: false,
                },
                Mode::Out => Assignable {
                    to: true, // TODO: This should be false, but since portmapping uses Assign and a portmapping puts
                    // the port on the left side, always, this breaks that interaction.
                    // Though really, this is just another example of why PortMapping needs a rework.
                    from: true,
                },
            },
            ObjectKind::Alias(_, kind) => Assignable::from(kind.as_ref()),
        }
    }
}

impl From<ObjectKind> for Assignable {
    fn from(kind: ObjectKind) -> Self {
        Assignable::from(&kind)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub struct Object {
    pub typ: Id<ObjectType>,
    pub assignable: Assignable,
}

impl Object {
    pub fn new(db: &dyn Arch, typ: Id<ObjectType>, assignable: Assignable) -> ObjectKey {
        db.intern_object(Object { typ, assignable }).into()
    }

    pub fn try_new(
        db: &dyn Arch,
        typ: impl TryIntern<ObjectType>,
        assignable: impl TryResult<Assignable>,
    ) -> Result<ObjectKey> {
        Ok(Object::new(
            db,
            typ.try_intern(db)?,
            assignable.try_result()?,
        ))
    }

    pub fn typ(&self, db: &dyn Arch) -> ObjectType {
        db.lookup_intern_object_type(self.typ)
    }

    pub fn typ_id(&self) -> Id<ObjectType> {
        self.typ
    }
}
