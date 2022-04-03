use core::fmt;

use std::sync::Arc;
use tydi_intern::Id;

use crate::ir::{
    traits::{GetSelf, InternSelf, MoveDb},
    Ir,
};
use tydi_common::{
    error::{Result, TryResult},
    map::InsertionOrderedMap,
    name::{Name, PathName},
};

use super::LogicalType;

/// The Group stream type acts as a product type (composition).
///
/// [Reference](https://abs-tudelft.github.io/tydi/specification/logical.html#group)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Group {
    fields: InsertionOrderedMap<PathName, Id<LogicalType>>,
}

impl Group {
    /// Returns a new Group logical stream type. Returns an error when either
    /// the name or logical stream type conversion fails, or when there are
    /// duplicate names.
    pub fn try_new(
        parent_id: Option<PathName>,
        group: impl IntoIterator<Item = (impl TryResult<Name>, Id<LogicalType>)>,
    ) -> Result<Self> {
        let base_id = match parent_id {
            Some(id) => id,
            None => PathName::new_empty(),
        };
        let mut fields = InsertionOrderedMap::new();
        for (name, typ) in group
            .into_iter()
            .map(|(name, typ)| Ok((name.try_result()?, typ)))
            .collect::<Result<Vec<_>>>()?
        {
            let path_name = base_id.with_child(name);
            fields.try_insert(path_name, typ)?;
        }
        Ok(Group { fields })
    }

    /// Create a new Group explicitly from a set of ordered fields with PathNames.
    pub(crate) fn new(fields: InsertionOrderedMap<PathName, Id<LogicalType>>) -> Self {
        Group { fields }
    }

    /// Returns the fields of the Group.
    pub fn fields(&self, db: &dyn Ir) -> Arc<InsertionOrderedMap<PathName, LogicalType>> {
        let mut result = InsertionOrderedMap::new();
        for (name, id) in self.field_ids() {
            result.insert_or_replace(name.clone(), id.get(db));
        }
        Arc::new(result)
    }

    /// Returns the fields of the Group with the logical types as IDs.
    pub fn field_ids(&self) -> &InsertionOrderedMap<PathName, Id<LogicalType>> {
        &self.fields
    }

    /// Gets the LogicalType of a field, if the field exists.
    pub fn get_field(&self, db: &dyn Ir, name: &PathName) -> Option<LogicalType> {
        match self.fields.get(name) {
            Some(x) => Some(x.get(db)),
            None => None,
        }
    }

    /// Gets the ID of the LogicalType of a field, if the field exists.
    pub fn get_field_id(&self, name: &PathName) -> Option<Id<LogicalType>> {
        match self.fields.get(name) {
            Some(x) => Some(*x),
            None => None,
        }
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

impl MoveDb<Id<LogicalType>> for Group {
    fn move_db(
        &self,
        original_db: &dyn Ir,
        target_db: &dyn Ir,
        prefix: &Option<Name>,
    ) -> Result<Id<LogicalType>> {
        let mut fields = InsertionOrderedMap::new();
        for (name, id) in self.field_ids() {
            fields.insert_or_replace(name.clone(), id.move_db(original_db, target_db, prefix)?);
        }
        Ok(LogicalType::from(Group { fields }).intern(target_db))
    }
}

impl fmt::Display for Group {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = self
            .field_ids()
            .into_iter()
            .map(|(name, id)| format!("{}: {}", name, id))
            .collect::<Vec<String>>()
            .join(", ");
        write!(f, "({})", fields)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{db::Database, interner::Interner};

    #[test]
    fn test_new() {
        let db = Database::default();
        let bits = db.intern_type(LogicalType::try_new_bits(8).unwrap());
        let group = Group::try_new(None, vec![("a", bits)]).unwrap();
        assert_eq!(
            group
                .get_field_id(&PathName::try_new(vec!["a"]).unwrap())
                .unwrap(),
            bits
        );
    }
}
