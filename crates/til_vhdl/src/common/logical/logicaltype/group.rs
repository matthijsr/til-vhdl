use std::{convert::TryInto, error, sync::Arc};

use indexmap::IndexMap;
use tydi_intern::Id;

use crate::ir::{InternSelf, Ir};
use tydi_common::{
    error::{Error, Result},
    name::{Name, PathName},
};

use super::{LogicalField, LogicalType};

/// The Group stream type acts as a product type (composition).
///
/// [Reference](https://abs-tudelft.github.io/tydi/specification/logical.html#group)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Group(pub(super) Vec<Id<LogicalField>>);

impl Group {
    /// Returns a new Group logical stream type. Returns an error when either
    /// the name or logical stream type conversion fails, or when there are
    /// duplicate names.
    pub fn try_new(
        db: &dyn Ir,
        parent_id: Option<PathName>,
        group: impl IntoIterator<
            Item = (
                impl TryInto<Name, Error = impl Into<Box<dyn error::Error>>>,
                Id<LogicalType>,
            ),
        >,
    ) -> Result<Self> {
        let mut map = IndexMap::new();
        for (name, typ) in group
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
        let base_id = match parent_id {
            Some(id) => id,
            None => PathName::new_empty(),
        };
        let fields = map
            .into_iter()
            .map(|(name, typ)| LogicalField::new(base_id.with_child(name), typ).intern(db))
            .collect();
        Ok(Group(fields))
    }

    pub(crate) fn new(db: &dyn Ir, fields: Vec<LogicalField>) -> Self {
        let fields = fields.iter().map(|x| x.intern(db)).collect();
        Group(fields)
    }

    /// Returns the fields of the Group.
    pub fn fields(&self, db: &dyn Ir) -> Arc<Vec<LogicalField>> {
        Arc::new(
            self.0
                .iter()
                .map(|x| db.lookup_intern_field(x.clone()))
                .collect(),
        )
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::Database;

    #[test]
    fn test_new() {
        let mut db = Database::default();
        let bits = db.intern_type(LogicalType::try_new_bits(8).unwrap());
        let group = Group::try_new(&db, None, vec![("a", bits)]).unwrap();
        let fields = group.fields(&db);
        let field = fields.last().unwrap();
        assert_eq!(field.name().last().unwrap(), "a");
        assert_eq!(field.typ(&db), db.lookup_intern_type(bits));
    }
}
