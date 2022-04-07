use core::fmt;

use textwrap::indent;
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

impl fmt::Display for ElementManipulatingReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ElementManipulatingReference::Null => write!(f, "Null"),
            ElementManipulatingReference::Bits(b) => write!(f, "Bits({})", b),
            ElementManipulatingReference::Group(g) => write!(
                f,
                "Group (\n{})",
                indent(
                    &g.iter()
                        .map(|(n, t)| format!("{}: {}\n", n, t))
                        .collect::<Vec<String>>()
                        .join(""),
                    "  "
                )
            ),
            ElementManipulatingReference::Union(u) => write!(
                f,
                "Union (\n{})",
                indent(
                    &u.iter()
                        .map(|(n, t)| format!("{}: {}\n", n, t))
                        .collect::<Vec<String>>()
                        .join(""),
                    "  "
                )
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FieldSelect {
    Group(Name),
    Union(Name),
}

impl fmt::Display for FieldSelect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FieldSelect::Group(n) => write!(f, "Group Field ({})", n),
            FieldSelect::Union(n) => write!(f, "Union Field ({})", n),
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

impl fmt::Display for TypeReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeReference::ElementManipulating(el) => write!(f, "{}", el),
            TypeReference::Stream(stream) => write!(f, "{}", stream),
            TypeReference::ScopeStream(scope) => write!(f, "{}", scope),
        }
    }
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
                    Self::Stream(StreamReference::from_stream_id(
                        db,
                        *stream_id,
                        path_name,
                        split_streams,
                        stream_data_hierarchy.as_ref(),
                    )?)
                } else {
                    TypeReference::ScopeStream(ScopeStream::new(
                        path_name.clone(),
                        Box::new(Self::collect_reference_from_split_streams(
                            db,
                            split_streams,
                            stream_data_hierarchy.as_ref(),
                            path_name,
                        )?),
                    ))
                }
            }
        })
    }

    /// Attempts to get a physical stream at the given location.
    pub fn get(
        &self,
        path: impl IntoIterator<Item = Option<FieldSelect>>,
    ) -> Result<StreamReference> {
        let mut result = self;
        let mut curr_path = vec![];
        for sub in path {
            if let Some(select) = sub {
                match result {
                    TypeReference::ElementManipulating(el) => match el {
                        ElementManipulatingReference::Null => return Err(Error::InvalidArgument(format!("Path refers to a field, but found a Null type at this location: {}", curr_path.join(" -> ")))),
                        ElementManipulatingReference::Bits(_) => return Err(Error::InvalidArgument(format!("Path refers to a field, but found a Bits type at this location: {}", curr_path.join(" -> ")))),
                        ElementManipulatingReference::Group(group) => {
                            if let FieldSelect::Group(field) = &select {
                                match group.get(field) {
                                    Some(typ) => result = typ,
                                    None => return Err(Error::InvalidArgument(format!("Field {} does not exist in Group at location {}", field, curr_path.join(" -> ")))),
                                }
                            } else {
                                return Err(Error::InvalidArgument(format!("There is a Group at this location ({}), but selection is a {}", curr_path.join(" -> "), select)))
                            }
                        },
                        ElementManipulatingReference::Union(union) => {
                            if let FieldSelect::Union(field) = &select {
                                match union.get(field) {
                                    Some(typ) => result = typ,
                                    None => return Err(Error::InvalidArgument(format!("Field {} does not exist in Union at location {}", field, curr_path.join(" -> ")))),
                                }
                            } else {
                                return Err(Error::InvalidArgument(format!("There is a Union at this location ({}), but selection is a {}", curr_path.join(" -> "), select)))
                            }
                        },
                    },
                    TypeReference::Stream(_) |
                    TypeReference::ScopeStream(_) => return Err(Error::InvalidArgument(format!("Path refers to a field, but found a stream-manipulating type at this location: {}", curr_path.join(" -> ")))),
                }
                curr_path.push(select.to_string());
            } else {
                match result {
                    TypeReference::ElementManipulating(_) => return Err(Error::InvalidArgument(format!("Path refers to a scope/stream, but found an element-manipulating type at this location: {}", curr_path.join(" -> ")))),
                    TypeReference::Stream(stream) => {
                        result = stream.data();
                        curr_path.push(format!("Scope ({})", stream.physical_stream()));
                    },
                    TypeReference::ScopeStream(scope) => {
                        result = scope.child();
                        curr_path.push(format!("Scope ({})", scope.name()));
                    },
                }
            }
        }

        if let TypeReference::Stream(stream) = result {
            Ok(stream.clone())
        } else {
            Err(Error::InvalidArgument(format!(
                "There is no physical stream for path {}, found a {}",
                curr_path.join(" -> "),
                result
            )))
        }
    }
}
