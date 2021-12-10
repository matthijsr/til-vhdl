use std::{convert::TryInto, error};

use indexmap::IndexMap;

use crate::common::{
    error::{Error, Result},
    integers::{BitCount, NonNegative},
    name::Name,
    util::log2_ceil,
};

use super::LogicalType;

///
///
/// [Reference](https://abs-tudelft.github.io/tydi/specification/logical.html#union)
#[derive(Debug, Clone, PartialEq)]
pub struct Union(pub(super) IndexMap<Name, LogicalType>);

impl Union {
    /// Returns a new Union logical stream type. Returns an error when either
    /// the name or logical stream type conversion fails, or when there are
    /// duplicate names.
    pub fn try_new(
        union: impl IntoIterator<
            Item = (
                impl TryInto<Name, Error = impl Into<Box<dyn error::Error>>>,
                impl TryInto<LogicalType, Error = impl Into<Box<dyn error::Error>>>,
            ),
        >,
    ) -> Result<Self> {
        let mut map = IndexMap::new();
        for (name, stream) in union
            .into_iter()
            .map(
                |(name, stream)| match (name.try_into(), stream.try_into()) {
                    (Ok(name), Ok(stream)) => Ok((name, stream)),
                    (Err(name), _) => Err(Error::from(name.into())),
                    (_, Err(stream)) => Err(Error::from(stream.into())),
                },
            )
            .collect::<Result<Vec<_>>>()?
        {
            map.insert(name, stream)
                .map(|_| -> Result<()> { Err(Error::UnexpectedDuplicate) })
                .transpose()?;
        }
        Ok(Union(map))
    }

    /// Returns the tag name and width of this union.
    /// [Reference](https://abs-tudelft.github.io/tydi/specification/logical.html)
    pub fn tag(&self) -> Option<(String, BitCount)> {
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

    /// Returns an iterator over the fields of the Union.
    pub fn iter(&self) -> impl Iterator<Item = (&Name, &LogicalType)> {
        self.0.iter()
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
