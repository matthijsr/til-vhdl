use crate::error::{Error, Result};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// A Set which keeps track of insertion order, but also implements Eq and Hash
///
/// This Set exists so that it may be used in structs stored in `salsa` databases.
/// In virtually all other cases, you should prefer `IndexSet`.
///
/// This set does not support Remove, and will return an Error when attempting to
/// insert a key which already exists.
pub struct InsertionOrderedSet<V: Clone + PartialEq + Ord> {
    items: InsertionOrderedMap<V, ()>,
}

impl<V: Clone + PartialEq + Ord + ToString> InsertionOrderedSet<V> {
    pub fn new() -> Self {
        InsertionOrderedSet {
            items: InsertionOrderedMap::new(),
        }
    }

    pub fn try_new(from: impl IntoIterator<Item = V>) -> Result<Self> {
        let mut result = Self::new();
        for v in from.into_iter() {
            result.try_insert(v)?;
        }
        Ok(result)
    }

    /// Tries to insert a value into the set.
    ///
    /// If the given value already exists in the map, this function
    /// will return an `Error::UnexpectedDuplicate`.
    pub fn try_insert(&mut self, value: V) -> Result<()> {
        self.items.try_insert(value, ())
    }

    /// Adds a value to the set.
    ///
    /// If the set did not have an equal element present, `true` is returned.
    ///
    /// If the set did have an equal element present, `false` is returned, and
    /// the entry is not updated.
    pub fn insert(&mut self, value: V) -> bool {
        match self.items.insert_or_replace(value, ()) {
            Some(_) => false,
            None => true,
        }
    }

    pub fn contains(&self, value: &V) -> bool {
        self.items.contains(value)
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = &V> {
        self.items.iter().map(|(v, _)| v)
    }

    pub fn into_iter(self) -> impl ExactSizeIterator<Item = V> {
        self.items.into_iter().map(|(v, _)| v)
    }
}

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

    /// Tries to append the keys and values from another `InsertionOrderedMap`
    /// to this one.
    pub fn try_append(&mut self, other: Self) -> Result<()> {
        for (k, v) in other.into_iter() {
            self.try_insert(k, v)?;
        }
        Ok(())
    }

    /// Tries to append the keys and values from another `InsertionOrderedMap`
    /// to this one, and replaces duplicate keys with the values from `other`.
    ///
    /// Returns a vector with all values that were replaced.
    pub fn append_or_replace(&mut self, other: Self) -> Vec<V> {
        let mut replaced_vals = vec![];
        for (k, v) in other.into_iter() {
            match self.insert_or_replace(k, v) {
                Some(replaced) => replaced_vals.push(replaced),
                None => (),
            }
        }
        replaced_vals
    }

    /// Tries to insert a key and value into the map.
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

    /// Tries to replace the value associated with a key in the map.
    ///
    /// If an item with the given key does not exist in the map, this function
    /// will return an `Error::InvalidArgument`.
    pub fn try_replace(&mut self, key: &K, value: V) -> Result<()> {
        let item = self.items.get_mut(key);
        match item {
            Some(mut_val) => Ok(*mut_val = value),
            None => Err(Error::InvalidArgument(format!(
                "No value associated with {}, nothing to replace",
                key.to_string()
            ))),
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

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.items.get_mut(key)
    }

    pub fn get_or_insert(&mut self, key: &K, or_insert: V) -> &mut V {
        if self.contains(key) {
            self.get_mut(key).unwrap()
        } else {
            self.try_insert(key.clone(), or_insert).unwrap();
            self.get_mut(key).unwrap()
        }
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

    pub fn try_get_mut(&mut self, key: &K) -> Result<&mut V> {
        if let Some(v) = self.get_mut(key) {
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

    pub fn iter(&self) -> impl ExactSizeIterator<Item = (&K, &V)> {
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

    /// Use a function to convert the map's values from `V` to `R`, using `&K`
    /// in its function
    pub fn map_convert_with_key<F, R: Clone + PartialEq>(
        self,
        mut f: F,
    ) -> InsertionOrderedMap<K, R>
    where
        F: FnMut(&K, V) -> R,
    {
        let len = self.len();
        let keys = self.keys;
        let items = self
            .items
            .into_iter()
            .map(|(n, x)| {
                let r = f(&n, x);
                (n, r)
            })
            .collect();
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

    /// Use a function to try to convert the map's values from `V` to `R`, using
    /// `&K` in its function
    pub fn try_map_convert_with_key<F, R: Clone + PartialEq>(
        self,
        mut f: F,
    ) -> Result<InsertionOrderedMap<K, R>>
    where
        F: FnMut(&K, V) -> Result<R>,
    {
        let len = self.len();
        let keys = self.keys;
        let mut items = BTreeMap::new();
        for (n, x) in self.items.into_iter() {
            let r = f(&n, x)?;
            items.insert(n, r);
        }
        Ok(InsertionOrderedMap::<K, R> { len, keys, items })
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.iter().map(|(k, _)| k)
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.iter().map(|(_, v)| v)
    }

    ///Returns an unordered iterator of mutable values
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.items.values_mut()
    }
}

pub struct InsertionOrderedMapIter<K: Ord + Clone, V: Clone> {
    len: usize,
    index: usize,
    insertion_ordered_map: InsertionOrderedMap<K, V>,
}

pub struct InsertionOrderedMapRefIter<'a, K: Ord + Clone, V: Clone> {
    len: usize,
    index: usize,
    insertion_ordered_map: &'a InsertionOrderedMap<K, V>,
}

impl<K: Ord + Clone, V: Clone> Iterator for InsertionOrderedMapIter<K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        match self.insertion_ordered_map.keys.get(&self.index) {
            Some(key) => {
                self.index += 1;
                self.insertion_ordered_map.items.remove_entry(key)
            }
            None => None,
        }
    }
}

impl<'a, K: Ord + Clone, V: Clone> Iterator for InsertionOrderedMapRefIter<'a, K, V> {
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

impl<K: Ord + Clone, V: Clone> ExactSizeIterator for InsertionOrderedMapIter<K, V> {
    fn len(&self) -> usize {
        self.len
    }
}

impl<'a, K: Ord + Clone, V: Clone> ExactSizeIterator for InsertionOrderedMapRefIter<'a, K, V> {
    fn len(&self) -> usize {
        self.len
    }
}

impl<K: Ord + Clone, V: Clone> IntoIterator for InsertionOrderedMap<K, V> {
    type Item = (K, V);

    type IntoIter = InsertionOrderedMapIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        InsertionOrderedMapIter {
            len: self.len,
            index: 0,
            insertion_ordered_map: self,
        }
    }
}

impl<'a, K: Ord + Clone, V: Clone> IntoIterator for &'a InsertionOrderedMap<K, V> {
    type Item = (&'a K, &'a V);

    type IntoIter = InsertionOrderedMapRefIter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        InsertionOrderedMapRefIter {
            len: self.len,
            index: 0,
            insertion_ordered_map: self,
        }
    }
}
