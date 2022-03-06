use object_type::ObjectType;
use tydi_common::error::{Error, Result, TryResult};
use tydi_intern::Id;

use crate::architecture::arch_storage::{interner::TryIntern, Arch};

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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub struct Object {
    pub typ: Id<ObjectType>,
    pub assignable: Assignable,
}

impl Object {
    pub fn new(db: &dyn Arch, typ: Id<ObjectType>, assignable: Assignable) -> Id<Object> {
        db.intern_object(Object { typ, assignable })
    }

    pub fn try_new(
        db: &dyn Arch,
        typ: impl TryIntern<ObjectType>,
        assignable: impl TryResult<Assignable>,
    ) -> Result<Id<Object>> {
        Ok(Object::new(
            db,
            typ.try_intern(db)?,
            assignable.try_result()?,
        ))
    }
}
