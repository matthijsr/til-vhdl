use std::{collections::BTreeMap, convert::TryInto, error, sync::Arc};

use indexmap::IndexMap;
use tydi_intern::Id;

use crate::ir::{GetSelf, InternSelf, Ir};
use tydi_common::{
    error::{Error, Result},
    name::{Name, PathName},
    numbers::{BitCount, NonNegative},
    util::log2_ceil,
};

use super::LogicalType;

///
///
/// [Reference](https://abs-tudelft.github.io/tydi/specification/logical.html#union)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Union {
    fields: BTreeMap<PathName, Id<LogicalType>>,
    field_order: Vec<PathName>,
}

impl Union {
    /// Returns a new Union logical stream type. Returns an error when either
    /// the name or logical stream type conversion fails, or when there are
    /// duplicate names.
    pub fn try_new(
        db: &dyn Ir,
        parent_id: Option<PathName>,
        union: impl IntoIterator<
            Item = (
                impl TryInto<Name, Error = impl Into<Box<dyn error::Error>>>,
                Id<LogicalType>,
            ),
        >,
    ) -> Result<Self> {
        let base_id = match parent_id {
            Some(id) => id,
            None => PathName::new_empty(),
        };
        let mut fields = BTreeMap::new();
        let mut field_order = vec![];
        for (name, typ) in union
            .into_iter()
            .map(|(name, typ)| match (name.try_into(), typ) {
                (Ok(name), _) => Ok((name, typ)),
                (Err(name), _) => Err(Error::from(name.into())),
            })
            .collect::<Result<Vec<_>>>()?
        {
            let path_name = base_id.with_child(name);
            field_order.push(path_name.clone());
            fields
                .insert(path_name, typ)
                .map(|_| -> Result<()> { Err(Error::UnexpectedDuplicate) })
                .transpose()?;
        }
        Ok(Union {
            fields,
            field_order,
        })
    }

    /// Create a new Union explicitly from a set of ordered fields with PathNames.
    pub(crate) fn new(db: &dyn Ir, fields: IndexMap<PathName, Id<LogicalType>>) -> Self {
        let mut map = BTreeMap::new();
        let mut field_order = vec![];
        for (name, id) in fields {
            field_order.push(name.clone());
            map.insert(name, id);
        }
        Union {
            fields: map,
            field_order,
        }
    }

    /// Returns the unordered fields of the Union.
    pub fn fields(&self, db: &dyn Ir) -> Arc<BTreeMap<PathName, LogicalType>> {
        Arc::new(
            self.fields
                .iter()
                .map(|(name, id)| (name.clone(), id.get(db)))
                .collect(),
        )
    }

    /// Returns the fields in the order they were declared
    pub fn ordered_fields(&self, db: &dyn Ir) -> Arc<IndexMap<PathName, LogicalType>> {
        let mut map = IndexMap::new();
        for name in &self.field_order {
            map.insert(name.clone(), self.get_field(db, name).unwrap());
        }
        Arc::new(map)
    }

    /// Returns the unordered fields of the Union with the logical types as IDs.
    pub fn field_ids(&self) -> &BTreeMap<PathName, Id<LogicalType>> {
        &self.fields
    }

    /// Returns the field IDs in the order they were declared
    pub fn ordered_field_ids(&self) -> Arc<IndexMap<PathName, Id<LogicalType>>> {
        let mut map = IndexMap::new();
        for name in &self.field_order {
            map.insert(name.clone(), self.get_field_id(name).unwrap());
        }
        Arc::new(map)
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

    /// Returns the tag width of this union.
    pub fn tag(&self) -> Option<BitCount> {
        if self.fields.len() > 1 {
            Some(
                BitCount::new(log2_ceil(
                    BitCount::new(self.fields.len() as NonNegative).unwrap(),
                ))
                .unwrap(),
            )
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::Database;

    #[test]
    fn test_new() {
        let mut db = Database::default();
        let bits = db.intern_type(LogicalType::try_new_bits(8).unwrap());
        let union = Union::try_new(&db, None, vec![("a", bits)]).unwrap();
        assert_eq!(
            union
                .get_field_id(&PathName::try_new(vec!["a"]).unwrap())
                .unwrap(),
            bits
        );
        assert_eq!(union.tag(), None);
    }
}
