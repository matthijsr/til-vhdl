use tydi_common::{
    error::Result,
    insertion_ordered_map::InsertionOrderedMap,
    name::{Name, PathName},
    numbers::BitCount,
    util::log2_ceil,
};
use tydi_intern::Id;

use crate::{
    common::logical::{
        logicaltype::stream::Synchronicity, type_reference::transfer_scope::TransferScope,
    },
    ir::{traits::GetSelf, Ir},
};

use self::{
    bits_reference::BitsReference, scope_stream::ScopeStream, stream_reference::StreamReference,
    union_reference::UnionReference,
};

use super::{
    logicaltype::{stream::Stream, LogicalType},
    type_hierarchy::TypeHierarchy,
};

pub mod bits_reference;
pub mod elements;
pub mod scope_stream;
pub mod stream_reference;
pub mod transfer_mode;
pub mod transfer_scope;
pub mod union_reference;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ElementManipulatingReference<F: Clone + PartialEq> {
    /// Null does not have any associated physical signals.
    Null,
    /// Bits(N) refers to N bits of a `data` signal
    Bits(BitsReference<F>),
    /// Group contains a number of distinct fields, which themselves refer to
    /// either element-manipulating nodes or Streams.
    Group(InsertionOrderedMap<Name, TypeReference<F>>),
    /// Union contains a number of overlapping fields, which themselves refer to
    /// either element-manipulating nodes or Streams, the active field is selected
    /// through the `tag` signal.
    ///
    /// Any element-manipulating nodes draw from a single, overlapped `union` signal.
    Union(UnionReference<F>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeReference<F: Clone + PartialEq> {
    /// Element-manipulating types should exclusively manipulate the `data` signal.
    ///
    /// However, they can still refer to Streams indirectly, thereby putting constraints
    /// on those Streams.
    ElementManipulating(ElementManipulatingReference<F>),
    /// A Stream type refers to a physical stream and its relevant signals.
    Stream(StreamReference<F>),
    /// Scope Streams refer to Streams which have been flattened into their
    /// children, they are scopes which exists in name only. Scope Streams are
    /// treated as synchronous by default, as their children should be
    /// independently marked as desynchronized.
    ScopeStream(ScopeStream<F>),
}

impl TypeReference<BitCount> {
    pub fn element_size(&self) -> u32 {
        match self {
            TypeReference::ElementManipulating(element) => match element {
                ElementManipulatingReference::Null => 0,
                ElementManipulatingReference::Bits(bits) => bits.bits.into(),
                ElementManipulatingReference::Group(fields) => {
                    let mut result = 0;
                    for (_, field) in fields {
                        result += field.element_size();
                    }
                    result
                }
                ElementManipulatingReference::Union(union_ref) => {
                    let mut result = 0;
                    if let Some(union) = union_ref.union {
                        result += u32::from(union);
                    }
                    if let Some(tag) = union_ref.tag {
                        result += u32::from(tag);
                    }
                    result
                }
            },
            TypeReference::Stream(_) => 0,
            TypeReference::ScopeStream(_) => 0,
        }
    }
}

pub trait TypeReferenceCollect<F: Clone + PartialEq> {
    fn collect_reference_from_type(
        self,
        db: &dyn Ir,
        hierarchy: &TypeHierarchy,
        path_name: &PathName,
    ) -> Result<TypeReference<F>>;
}

impl TypeReferenceCollect<BitCount> for &InsertionOrderedMap<PathName, Id<Stream>> {
    fn collect_reference_from_type(
        self,
        db: &dyn Ir,
        hierarchy: &TypeHierarchy,
        path_name: &PathName,
    ) -> Result<TypeReference<BitCount>> {
        Ok(match hierarchy {
            TypeHierarchy::Null => {
                TypeReference::ElementManipulating(ElementManipulatingReference::Null)
            }
            TypeHierarchy::Bits(n) => TypeReference::ElementManipulating(
                ElementManipulatingReference::Bits(BitsReference {
                    bits: *n,
                    field: *n,
                }),
            ),
            TypeHierarchy::Group(fields) => {
                let mut result_fields = InsertionOrderedMap::new();
                for (field_name, field_hierarchy) in fields {
                    let field_reference =
                        self.collect_reference_from_type(db, field_hierarchy, field_name)?;
                    result_fields
                        .try_insert(field_name.last().unwrap().clone(), field_reference)?;
                }

                TypeReference::ElementManipulating(ElementManipulatingReference::Group(
                    result_fields,
                ))
            }
            TypeHierarchy::Union(fields) => {
                let mut union_fields = InsertionOrderedMap::new();
                let mut union = None;
                let tag = if fields.len() > 1 {
                    Some(
                        BitCount::new(log2_ceil(BitCount::new(fields.len() as u32).unwrap()))
                            .unwrap(),
                    )
                } else {
                    None
                };
                for (field_name, field_hierarchy) in fields {
                    let field_reference =
                        self.collect_reference_from_type(db, field_hierarchy, field_name)?;
                    let field_size = BitCount::new(field_reference.element_size());
                    if let Some(union_size) = union {
                        if let Some(field_size) = field_size {
                            if field_size > union_size {
                                union = Some(field_size);
                            }
                        }
                    } else {
                        union = field_size;
                    }
                    union_fields.try_insert(field_name.last().unwrap().clone(), field_reference)?;
                }

                TypeReference::ElementManipulating(ElementManipulatingReference::Union(
                    UnionReference {
                        union_fields,
                        union,
                        tag,
                    },
                ))
            }
            TypeHierarchy::Stream(stream_data_hierarchy) => {
                if let Some(stream_id) = self.get(path_name) {
                    let stream = stream_id.get(db);
                    let physical_stream = path_name.clone();
                    let valid = BitCount::new(1).unwrap();
                    let ready = BitCount::new(1).unwrap();
                    let direction = stream.direction();
                    let complexity = stream.complexity();
                    let dimensionality = stream.dimensionality();
                    let transfer_scope = if path_name.is_empty() {
                        TransferScope::Root
                    } else {
                        match stream.synchronicity() {
                            Synchronicity::Sync | Synchronicity::Flatten => {
                                TransferScope::Sync(path_name.root())
                            }
                            Synchronicity::Desync | Synchronicity::FlatDesync => {
                                TransferScope::Root
                            }
                        }
                    };

                    // pub activity: ElementActivityReference<F>,
                    // pub elements: Vec<StreamElementReference<F>>,
                    // pub user: ElementManipulatingReference<F>,
                    todo!()
                } else {
                    TypeReference::ScopeStream(ScopeStream {
                        name: path_name.clone(),
                        child: Box::new(self.collect_reference_from_type(
                            db,
                            stream_data_hierarchy.as_ref(),
                            path_name,
                        )?),
                    })
                }
            }
        })
    }
}
