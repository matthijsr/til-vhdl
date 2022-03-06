use object_type::ObjectType;
use tydi_common::error::{Error, Result};
use tydi_intern::Id;

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
