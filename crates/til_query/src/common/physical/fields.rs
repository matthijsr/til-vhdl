use indexmap::IndexMap;

use tydi_common::{
    error::{Error, Result},
    name::PathName,
};

// TODO: Can probably replace this with InsertionOrderedMap

#[derive(Debug, Clone, PartialEq)]
pub struct Fields<T: Clone + PartialEq>(IndexMap<PathName, T>);

impl<T: Clone + PartialEq> Fields<T> {
    pub fn new(iter: impl IntoIterator<Item = (PathName, T)>) -> Result<Self> {
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

    pub fn new_empty() -> Self {
        Fields(IndexMap::new())
    }

    pub fn indexed(&self) -> &IndexMap<PathName, T> {
        &self.0
    }

    pub fn insert(&mut self, path_name: PathName, bit_count: T) -> Result<()> {
        self.0
            .insert(path_name, bit_count)
            .map(|_| -> Result<()> { Err(Error::UnexpectedDuplicate) })
            .transpose()?;
        Ok(())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&PathName, &T)> {
        self.0.iter()
    }

    pub fn keys(&self) -> impl Iterator<Item = &PathName> {
        self.0.keys()
    }

    pub fn values(&self) -> impl Iterator<Item = &T> {
        self.0.values()
    }

    /// Use a function to convert the Fields from `T` to `R`
    pub fn map<F, R: Clone + PartialEq>(self, mut f: F) -> Fields<R>
    where
        F: FnMut(T) -> R,
    {
        Fields(self.0.into_iter().map(|(n, x)| (n, f(x))).collect())
    }
}

impl<'a, T: Clone + PartialEq> IntoIterator for &'a Fields<T> {
    type Item = (&'a PathName, &'a T);
    type IntoIter = indexmap::map::Iter<'a, PathName, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
