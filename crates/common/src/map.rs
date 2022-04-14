use crate::error::{Error, Result};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// A Map which keeps track of insertion order, but also implements Eq and Hash
///
/// This Map exists so that it may be used in structs stored in `salsa` databases.
/// In virtually all other cases, you should prefer `IndexMap`.
///
/// This map does not support Remove, and will return an Error when attempting to
/// insert an item with a key which already exists.
pub struct InsertionOrderedMap<K: Ord + Clone, V: Clone> {
    len: usize,
    keys: BTreeMap<usize, K>,
    items: BTreeMap<K, V>,
}

impl<K: Ord + Clone + ToString, V: Clone> InsertionOrderedMap<K, V> {
    pub fn new() -> Self {
        InsertionOrderedMap {
            len: 0,
            keys: BTreeMap::new(),
            items: BTreeMap::new(),
        }
    }

    pub fn try_new(from: impl IntoIterator<Item = (K, V)>) -> Result<Self> {
        let mut result = Self::new();
        for (k, v) in from.into_iter() {
            result.try_insert(k, v)?;
        }
        Ok(result)
    }

    /// Tries to insert a key and value in to the map.
    ///
    /// If an item with the given key already exists in the map, this function
    /// will return an `Error::UnexpectedDuplicate`.
    pub fn try_insert(&mut self, key: K, value: V) -> Result<()> {
        match self.items.insert(key.clone(), value) {
            Some(_) => Err(Error::UnexpectedDuplicate(key.to_string())),
            None => {
                self.keys.insert(self.len, key);
                self.len += 1;
                Ok(())
            }
        }
    }

    /// Tries to insert a key and value into the map.
    ///
    /// If an item with the given key already exists in the map, it will replace
    /// its value, but retain the original insertion order for that key.
    pub fn insert_or_replace(&mut self, key: K, value: V) -> Option<V> {
        match self.items.insert(key.clone(), value) {
            Some(value) => Some(value),
            None => {
                self.keys.insert(self.len, key);
                self.len += 1;
                None
            }
        }
    }

    pub fn contains(&self, key: &K) -> bool {
        self.items.contains_key(key)
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.items.get(key)
    }

    pub fn try_get(&self, key: &K) -> Result<&V> {
        if let Some(v) = self.get(key) {
            Ok(v)
        } else {
            Err(Error::InvalidArgument(format!(
                "Key {} does not exist in this map.",
                key.to_string()
            )))
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        (&self).into_iter()
    }

    /// Get the index of a given key.
    ///
    /// Returns None if no such key exists.
    pub fn key_index(&self, key: &K) -> Option<usize> {
        self.keys
            .iter()
            .find_map(|(idx, k)| if k == key { Some(*idx) } else { None })
    }

    /// Use a function to convert the map's values from `V` to `R`
    pub fn map_convert<F, R: Clone + PartialEq>(self, mut f: F) -> InsertionOrderedMap<K, R>
    where
        F: FnMut(V) -> R,
    {
        let len = self.len();
        let keys = self.keys;
        let items = self.items.into_iter().map(|(n, x)| (n, f(x))).collect();
        InsertionOrderedMap::<K, R> { len, keys, items }
    }

    /// Use a function to try to convert the map's values from `V` to `R`
    pub fn try_map_convert<F, R: Clone + PartialEq>(
        self,
        mut f: F,
    ) -> Result<InsertionOrderedMap<K, R>>
    where
        F: FnMut(V) -> Result<R>,
    {
        let len = self.len();
        let keys = self.keys;
        let mut items = BTreeMap::new();
        for (n, x) in self.items.into_iter() {
            items.insert(n, f(x)?);
        }
        Ok(InsertionOrderedMap::<K, R> { len, keys, items })
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.iter().map(|(k, _)| k)
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.iter().map(|(_, v)| v)
    }
}

pub struct InsertionOrderedMapIter<'a, K: Ord + Clone, V: Clone> {
    len: usize,
    index: usize,
    insertion_ordered_map: &'a InsertionOrderedMap<K, V>,
}

impl<'a, K: Ord + Clone, V: Clone> Iterator for InsertionOrderedMapIter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        match self.insertion_ordered_map.keys.get(&self.index) {
            Some(key) => {
                self.index += 1;
                self.insertion_ordered_map.items.get_key_value(key)
            }
            None => None,
        }
    }
}

impl<'a, K: Ord + Clone, V: Clone> ExactSizeIterator for InsertionOrderedMapIter<'a, K, V> {
    fn len(&self) -> usize {
        self.len
    }
}

impl<'a, K: Ord + Clone, V: Clone> IntoIterator for &'a InsertionOrderedMap<K, V> {
    type Item = (&'a K, &'a V);

    type IntoIter = InsertionOrderedMapIter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        InsertionOrderedMapIter {
            len: self.len,
            index: 0,
            insertion_ordered_map: self,
        }
    }
}
