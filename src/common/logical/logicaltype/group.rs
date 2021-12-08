use std::{convert::TryInto, error};

use indexmap::IndexMap;

use crate::common::{
    error::{Error, Result},
    name::Name,
};

use super::LogicalType;

/// The Group stream type acts as a product type (composition).
///
/// [Reference](https://abs-tudelft.github.io/tydi/specification/logical.html#group)
#[derive(Debug, Clone, PartialEq)]
pub struct Group(pub(super) IndexMap<Name, LogicalType>);

impl Group {
    /// Returns a new Group logical stream type. Returns an error when either
    /// the name or logical stream type conversion fails, or when there are
    /// duplicate names.
    pub fn try_new(
        group: impl IntoIterator<
            Item = (
                impl TryInto<Name, Error = impl Into<Box<dyn error::Error>>>,
                impl TryInto<LogicalType, Error = impl Into<Box<dyn error::Error>>>,
            ),
        >,
    ) -> Result<Self> {
        let mut map = IndexMap::new();
        for (name, stream) in group
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
        Ok(Group(map))
    }

    /// Returns an iterator over the fields of the Group.
    pub fn iter(&self) -> impl Iterator<Item = (&Name, &LogicalType)> {
        self.0.iter()
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
