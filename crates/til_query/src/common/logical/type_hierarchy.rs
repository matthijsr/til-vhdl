use tydi_common::{
    error::Result, map::InsertionOrderedMap, name::PathName, numbers::Positive,
};
use tydi_intern::Id;

use crate::ir::{traits::GetSelf, Ir};

use super::logicaltype::{stream::Stream, LogicalType};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// A simplified representation of the hierarchy of different logical types.
///
/// This is used to reconstruct type references during Synthesis.
pub enum TypeHierarchy {
    Null,
    Bits(Positive),
    Group(InsertionOrderedMap<PathName, TypeHierarchy>),
    Union(InsertionOrderedMap<PathName, TypeHierarchy>),
    Stream(Box<TypeHierarchy>),
}

impl TypeHierarchy {
    pub fn from_stream(db: &dyn Ir, stream_id: Id<Stream>) -> Result<Self> {
        let logical_type = stream_id.get(db).data(db);
        Ok(match &logical_type {
            // If the data type is a Stream, automatically flatten it in the hierarchy.
            LogicalType::Stream(stream_id) => Self::from_stream(db, *stream_id)?,
            _ => Self::Stream(Box::new(Self::from_logical_type(db, logical_type)?)),
        })
    }

    pub fn from_logical_type(db: &dyn Ir, logical_type: LogicalType) -> Result<Self> {
        fn fields_into_hierarchy(
            db: &dyn Ir,
            fields: &InsertionOrderedMap<PathName, Id<LogicalType>>,
        ) -> Result<InsertionOrderedMap<PathName, TypeHierarchy>> {
            let mut result = InsertionOrderedMap::new();
            for (name, typ) in fields.iter() {
                result.try_insert(name.clone(), TypeHierarchy::from_logical_type_id(db, *typ)?)?;
            }
            Ok(result)
        }

        Ok(match logical_type {
            LogicalType::Null => Self::Null,
            LogicalType::Bits(n) => Self::Bits(n),
            LogicalType::Group(group) => Self::Group(fields_into_hierarchy(db, group.field_ids())?),
            LogicalType::Union(union) => Self::Union(fields_into_hierarchy(db, union.field_ids())?),
            LogicalType::Stream(stream_id) => Self::from_stream(db, stream_id)?,
        })
    }

    pub fn from_logical_type_id(db: &dyn Ir, logical_type: Id<LogicalType>) -> Result<Self> {
        Self::from_logical_type(db, logical_type.get(db))
    }
}
