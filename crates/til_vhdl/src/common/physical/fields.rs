use indexmap::IndexMap;

use crate::common::{
    error::{Error, Result},
    integers::BitCount,
    name::PathName,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Fields(IndexMap<PathName, BitCount>);

impl Fields {
    pub(crate) fn new(iter: impl IntoIterator<Item = (PathName, BitCount)>) -> Result<Self> {
        let fields = iter.into_iter();
        let (lower, upper) = fields.size_hint();
        let mut map = IndexMap::with_capacity(upper.unwrap_or(lower));

        for (path_name, bit_count) in fields {
            map.insert(path_name, bit_count)
                .map(|_| -> Result<()> { Err(Error::UnexpectedDuplicate) })
                .transpose()?;
        }

        Ok(Fields(map))
    }

    pub(crate) fn new_empty() -> Self {
        Fields(IndexMap::new())
    }

    pub(crate) fn insert(&mut self, path_name: PathName, bit_count: BitCount) -> Result<()> {
        self.0
            .insert(path_name, bit_count)
            .map(|_| -> Result<()> { Err(Error::UnexpectedDuplicate) })
            .transpose()?;
        Ok(())
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&PathName, &BitCount)> {
        self.0.iter()
    }

    pub(crate) fn keys(&self) -> impl Iterator<Item = &PathName> {
        self.0.keys()
    }

    pub(crate) fn values(&self) -> impl Iterator<Item = &BitCount> {
        self.0.values()
    }
}

impl<'a> IntoIterator for &'a Fields {
    type Item = (&'a PathName, &'a BitCount);
    type IntoIter = indexmap::map::Iter<'a, PathName, BitCount>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
