use tydi_common::{
    error::{Error, Result},
    map::InsertionOrderedMap,
    name::{Name, PathName},
    numbers::Positive,
};
use tydi_intern::Id;

use crate::ir::{traits::GetSelf, Ir};

use self::{scope_stream::ScopeStream, stream_reference::StreamReference};

use super::{logicaltype::LogicalType, split_streams::SplitStreams, type_hierarchy::TypeHierarchy};

pub mod scope_stream;
pub mod stream_reference;
pub mod transfer_mode;
pub mod transfer_scope;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ElementManipulatingReference {
    /// Null does not have any associated physical signals.
    Null,
    /// Bits(N) refers to N bits of a `data` signal
    Bits(Positive),
    /// Group contains a number of distinct fields, which themselves refer to
    /// either element-manipulating nodes or Streams.
    Group(InsertionOrderedMap<Name, TypeReference>),
    /// Union contains a number of overlapping fields, which themselves refer to
    /// either element-manipulating nodes or Streams, the active field is selected
    /// through the `tag` signal.
    ///
    /// Any element-manipulating nodes draw from a single, overlapped `union` signal.
    Union(InsertionOrderedMap<Name, TypeReference>),
}

impl ElementManipulatingReference {
    pub fn from_logical_type_id(db: &dyn Ir, logical_type: Id<LogicalType>) -> Result<Self> {
        fn logical_type_fields_into_ref_fields(
            db: &dyn Ir,
            fields: &InsertionOrderedMap<PathName, Id<LogicalType>>,
        ) -> Result<InsertionOrderedMap<Name, TypeReference>> {
            let mut result_fields = InsertionOrderedMap::new();
            for (field_name, field_id) in fields {
                result_fields.try_insert(
                    field_name.last().unwrap().clone(),
                    TypeReference::ElementManipulating(
                        ElementManipulatingReference::from_logical_type_id(db, *field_id)?,
                    ),
                )?;
            }
            Ok(result_fields)
        }

        match logical_type.get(db) {
            LogicalType::Null => Ok(ElementManipulatingReference::Null),
            LogicalType::Bits(n) => Ok(ElementManipulatingReference::Bits(n)),
            LogicalType::Group(group) => Ok(Self::Group(logical_type_fields_into_ref_fields(
                db,
                group.field_ids(),
            )?)),
            LogicalType::Union(union) => Ok(Self::Union(logical_type_fields_into_ref_fields(
                db,
                union.field_ids(),
            )?)),
            LogicalType::Stream(_) => Err(Error::InvalidArgument(
                "The user signal should only carry element-manipulating types".to_string(),
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeReference {
    /// Element-manipulating types should exclusively manipulate the `data` signal.
    ///
    /// However, they can still refer to Streams indirectly, thereby putting constraints
    /// on those Streams.
    ElementManipulating(ElementManipulatingReference),
    /// A Stream type refers to a physical stream and its relevant signals.
    Stream(StreamReference),
    /// Scope Streams refer to Streams which have been flattened into their
    /// children, they are scopes which exists in name only. Scope Streams are
    /// treated as synchronous by default, as their children should be
    /// independently marked as desynchronized.
    ScopeStream(ScopeStream),
}

impl TypeReference {
    pub fn collect_reference_from_split_streams(
        db: &dyn Ir,
        split_streams: &SplitStreams,
        hierarchy: &TypeHierarchy,
        path_name: &PathName,
    ) -> Result<TypeReference> {
        let streams = split_streams.streams_map();
        Ok(match hierarchy {
            TypeHierarchy::Null => {
                TypeReference::ElementManipulating(ElementManipulatingReference::Null)
            }
            TypeHierarchy::Bits(n) => {
                TypeReference::ElementManipulating(ElementManipulatingReference::Bits(*n))
            }
            TypeHierarchy::Group(fields) => {
                let mut group_fields = InsertionOrderedMap::new();
                for (field_name, field_hierarchy) in fields {
                    let field_reference = Self::collect_reference_from_split_streams(
                        db,
                        split_streams,
                        field_hierarchy,
                        field_name,
                    )?;
                    group_fields.try_insert(field_name.last().unwrap().clone(), field_reference)?;
                }

                TypeReference::ElementManipulating(ElementManipulatingReference::Group(
                    group_fields,
                ))
            }
            TypeHierarchy::Union(fields) => {
                let mut union_fields = InsertionOrderedMap::new();
                for (field_name, field_hierarchy) in fields {
                    let field_reference = Self::collect_reference_from_split_streams(
                        db,
                        split_streams,
                        field_hierarchy,
                        field_name,
                    )?;
                    union_fields.try_insert(field_name.last().unwrap().clone(), field_reference)?;
                }

                TypeReference::ElementManipulating(ElementManipulatingReference::Union(
                    union_fields,
                ))
            }
            TypeHierarchy::Stream(stream_data_hierarchy) => {
                if let Some(stream_id) = streams.get(path_name) {
                    Self::Stream(StreamReference::from_stream_id(db, *stream_id, path_name)?)
                } else {
                    TypeReference::ScopeStream(ScopeStream {
                        name: path_name.clone(),
                        child: Box::new(Self::collect_reference_from_split_streams(
                            db,
                            split_streams,
                            stream_data_hierarchy.as_ref(),
                            path_name,
                        )?),
                    })
                }
            }
        })
    }
}
