use tydi_common::{
    error::{Result, TryOptional, TryResult},
    map::InsertionOrderedMap,
    name::{Name, PathName},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScopeStore<T>
where
    T: Clone,
{
    name: Option<Name>,
    value: T,
    synched: bool,
    child_scopes: InsertionOrderedMap<Name, ScopeStore<T>>,
}

impl<T: Clone> ScopeStore<T> {
    pub fn new(name: Option<Name>, value: T, synched: bool) -> Self {
        Self {
            name,
            value,
            synched,
            child_scopes: InsertionOrderedMap::new(),
        }
    }

    pub fn try_new(
        name: impl TryOptional<Name>,
        value: impl TryResult<T>,
        synced: bool,
    ) -> Result<Self> {
        Ok(Self {
            name: name.try_optional()?,
            value: value.try_result()?,
            synched: synced,
            child_scopes: InsertionOrderedMap::new(),
        })
    }

    /// Get a reference to the scope store's name.
    #[must_use]
    pub fn name(&self) -> Option<&Name> {
        self.name.as_ref()
    }

    /// Get a reference to the scope store's value.
    #[must_use]
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Indicates whether this scope is synchronized with its parent scope
    #[must_use]
    pub fn synched(&self) -> bool {
        self.synched
    }

    /// Get a reference to the scope store's child scopes.
    #[must_use]
    pub fn child_scopes(&self) -> &InsertionOrderedMap<Name, ScopeStore<T>> {
        &self.child_scopes
    }

    fn try_get_with_parents<'a>(
        &self,
        mut path: impl Iterator<Item = &'a Name>,
        mut parents: Vec<T>,
    ) -> Result<ScopeResult<T>> {
        if let Some(name) = path.next() {
            parents.push(self.value().clone());
            let child = self.child_scopes().try_get(name)?;
            child.try_get_with_parents(path, parents)
        } else {
            Ok(ScopeResult::new(self.value().clone()).with_synched_parents(parents))
        }
    }

    pub fn try_get(&self, path: &PathName) -> Result<ScopeResult<T>> {
        self.try_get_with_parents(path.as_ref().iter(), vec![])
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScopeResult<T: Clone> {
    selection: T,
    synched_parents: Vec<T>,
}

impl<T: Clone> ScopeResult<T> {
    pub fn new(selection: T) -> Self {
        Self {
            selection,
            synched_parents: vec![],
        }
    }

    pub fn set_synched_parents(&mut self, synched_parents: Vec<T>) {
        self.synched_parents = synched_parents;
    }

    pub fn with_synched_parents(mut self, synched_parents: Vec<T>) -> Self {
        self.set_synched_parents(synched_parents);
        self
    }

    /// Get a reference to the scope result's selection.
    #[must_use]
    pub fn selection(&self) -> &T {
        &self.selection
    }

    /// Get a reference to the scope result's synched parents.
    #[must_use]
    pub fn synched_parents(&self) -> &[T] {
        self.synched_parents.as_ref()
    }
}
