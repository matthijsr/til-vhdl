use std::{convert::TryInto, error};

use indexmap::IndexMap;
use tydi_intern::Id;

use crate::{
    common::{
        error::{Error, Result},
        integers::{BitCount, NonNegative},
        name::Name,
        util::log2_ceil,
    },
    ir::{Identifier, Ir},
};

use super::{Field, LogicalType};

///
///
/// [Reference](https://abs-tudelft.github.io/tydi/specification/logical.html#union)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Union(pub(super) Vec<Id<Field>>);

impl Union {
    /// Returns a new Union logical stream type. Returns an error when either
    /// the name or logical stream type conversion fails, or when there are
    /// duplicate names.
    pub(crate) fn try_new(
        db: &dyn Ir,
        parent_id: Id<Identifier>,
        union: impl IntoIterator<
            Item = (
                impl TryInto<Name, Error = impl Into<Box<dyn error::Error>>>,
                Id<LogicalType>,
            ),
        >,
    ) -> Result<Self> {
        let mut map = IndexMap::new();
        for (name, typ) in union
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
        Ok(Union(fields))
    }

    /// Returns the tag name and width of this union.
    /// [Reference](https://abs-tudelft.github.io/tydi/specification/logical.html)
    pub(crate) fn tag(&self) -> Option<(String, BitCount)> {
        if self.0.len() > 1 {
            Some((
                "tag".to_string(),
                BitCount::new(log2_ceil(
                    BitCount::new(self.0.len() as NonNegative).unwrap(),
                ))
                .unwrap(),
            ))
        } else {
            None
        }
    }
}

impl From<Union> for LogicalType {
    /// Wraps this union in a [`LogicalType`].
    ///
    /// [`LogicalType`]: ./enum.LogicalType.html
    fn from(union: Union) -> Self {
        LogicalType::Union(union)
    }
}
