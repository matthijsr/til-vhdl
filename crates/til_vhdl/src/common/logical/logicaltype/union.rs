use std::{convert::TryInto, error, sync::Arc};

use indexmap::IndexMap;
use tydi_intern::Id;

use crate::{
    common::{
        error::{Error, Result},
        integers::{BitCount, NonNegative},
        name::Name,
        util::log2_ceil,
    },
    ir::{Identifier, Ir},
};

use super::{Field, LogicalType};

///
///
/// [Reference](https://abs-tudelft.github.io/tydi/specification/logical.html#union)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Union(pub(super) Vec<Id<Field>>);

impl Union {
    /// Returns a new Union logical stream type. Returns an error when either
    /// the name or logical stream type conversion fails, or when there are
    /// duplicate names.
    pub fn try_new(
        db: &dyn Ir,
        parent_id: Option<Identifier>,
        union: impl IntoIterator<
            Item = (
                impl TryInto<Name, Error = impl Into<Box<dyn error::Error>>>,
                Id<LogicalType>,
            ),
        >,
    ) -> Result<Self> {
        let mut map = IndexMap::new();
        for (name, typ) in union
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
            None => vec![],
        };
        let fields = map
            .into_iter()
            .map(|(name, typ)| db.intern_field(Field::new(db, &base_id, name, typ)))
            .collect();
        Ok(Union(fields))
    }

    /// Returns the tag width of this union.
    pub fn tag(&self) -> Option<BitCount> {
        if self.0.len() > 1 {
            Some(
                BitCount::new(log2_ceil(
                    BitCount::new(self.0.len() as NonNegative).unwrap(),
                ))
                .unwrap(),
            )
        } else {
            None
        }
    }

    /// Returns an iterator over the fields of the Union.
    pub fn iter(&self, db: &dyn Ir) -> Arc<Vec<Field>> {
        Arc::new(
            self.0
                .iter()
                .map(|x| db.lookup_intern_field(x.clone()))
                .collect(),
        )
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
        let fields = union.iter(&db);
        let field = fields.last().unwrap();
        assert_eq!(field.name().last().unwrap(), "a");
        assert_eq!(field.typ(&db), db.lookup_intern_type(bits));
        assert_eq!(union.tag(), None);
    }
}
