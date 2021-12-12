use std::{convert::TryInto, error};

use indexmap::IndexMap;
use tydi_intern::Id;

use crate::{
    common::{
        error::{Error, Result},
        name::Name,
    },
    ir::{Identifier, Ir},
};

use super::{Field, LogicalType};

/// The Group stream type acts as a product type (composition).
///
/// [Reference](https://abs-tudelft.github.io/tydi/specification/logical.html#group)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Group(pub(super) Vec<Id<Field>>);

impl Group {
    /// Returns a new Group logical stream type. Returns an error when either
    /// the name or logical stream type conversion fails, or when there are
    /// duplicate names.
    pub(crate) fn try_new(
        db: &dyn Ir,
        parent_id: Id<Identifier>,
        group: impl IntoIterator<
            Item = (
                impl TryInto<Name, Error = impl Into<Box<dyn error::Error>>>,
                Id<LogicalType>,
            ),
        >,
    ) -> Result<Self> {
        let mut map = IndexMap::new();
        for (name, typ) in group
            .into_iter()
            .map(|(name, typ)| match (name.try_into(), typ) {
                (Ok(name), _) => Ok((name, typ)),
                (Err(name), _) => Err(Error::from(name.into())),
            })
            .collect::<Result<Vec<_>>>()?
        {
            map.insert(name, typ)
                .map(|_| -> Result<()> { Err(Error::UnexpectedDuplicate) })
                .transpose()?;
        }
        let base_id = db.lookup_intern_identifier(parent_id);
        let fields = map
            .into_iter()
            .map(|(name, typ)| db.intern_field(Field::new(db, &base_id, name, typ)))
            .collect();
        Ok(Group(fields))
    }
}

impl From<Group> for LogicalType {
    /// Wraps this group in a [`LogicalType`].
    ///
    /// [`LogicalType`]: ./enum.LogicalType.html
    fn from(group: Group) -> Self {
        LogicalType::Group(group)
    }
}
