use std::{convert::TryInto, error, sync::Arc};

use crate::{
    common::physical::{fields::Fields, stream::PhysicalStream},
    ir::{GetSelf, InternSelf, IntoVhdl, Ir},
};
use indexmap::IndexMap;
use tydi_common::{
    error::{Error, Result},
    name::{Name, PathName},
    numbers::{BitCount, NonNegative, Positive},
    traits::Reverse,
    util::log2_ceil,
};

pub use field::*;
pub mod field;

pub use group::*;
pub mod group;

pub use stream::*;
pub mod stream;

use tydi_intern::Id;
pub use union::*;
pub mod union;

pub trait IsNull {
    fn is_null(&self, db: &dyn Ir) -> bool;
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SplitStreams {
    signals: LogicalType,
    streams: IndexMap<PathName, Stream>,
}

impl SplitStreams {
    pub fn streams(&self) -> impl Iterator<Item = (&PathName, &Stream)> {
        self.streams.iter()
    }
    pub fn signals(&self) -> &LogicalType {
        &self.signals
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LogicalStream {
    signals: Fields,
    streams: IndexMap<PathName, PhysicalStream>,
}

impl LogicalStream {
    #[allow(dead_code)]
    pub fn signals(&self) -> impl Iterator<Item = (&PathName, &BitCount)> {
        self.signals.iter()
    }

    pub fn streams(&self) -> impl Iterator<Item = (&PathName, &PhysicalStream)> {
        self.streams.iter()
    }
}

/// Types of logical streams.
///
/// This structure is at the heart of the logical stream specification. It is
/// used both to specify the type of a logical stream and internally for the
/// process of lowering the recursive structure down to physical streams and
/// signals.
///
/// The logical stream type is defined recursively by means of a number of
/// stream types. Two classes of stream types are defined: stream-manipulating
/// types, and element-manipulating types.
///
/// # Examples
///
/// ```rust
/// ```
///
/// [Reference](https://abs-tudelft.github.io/tydi/specification/logical.html#logical-stream-type)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LogicalType {
    /// The Null stream type indicates the transferrence of one-valued data: it
    /// is only valid value is ∅ (null).
    ///
    /// [Reference](https://abs-tudelft.github.io/tydi/specification/logical.html#null)
    Null,
    /// The Bits stream type, defined as `Bits(b)`, indicates the transferrence
    /// of `2^b`-valued data carried by means of a group of `b` bits, where`b`
    /// is a positive integer.
    ///
    /// [Reference](https://abs-tudelft.github.io/tydi/specification/logical.html#bits)
    Bits(Positive),
    /// The Group stream type acts as a product type (composition).
    ///
    /// [Reference](https://abs-tudelft.github.io/tydi/specification/logical.html#group)
    Group(Group),
    /// The Union stream type acts as a sum type (exclusive disjunction).
    ///
    /// [Reference](https://abs-tudelft.github.io/tydi/specification/logical.html#union)
    Union(Union),
    /// The Stream type is used to define a new physical stream.
    ///
    /// [Reference](https://abs-tudelft.github.io/tydi/specification/logical.html#stream)
    Stream(Id<Stream>),
}

impl LogicalType {
    /// Returns a new Bits stream type with the provided bit count as number of
    /// bits. Returns an error when the bit count is zero.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tydi::{Error, logical::LogicalType, Positive};
    ///
    /// let bits = LogicalType::try_new_bits(4);
    /// let zero = LogicalType::try_new_bits(0);
    ///
    /// assert_eq!(bits, Ok(LogicalType::Bits(Positive::new(4).unwrap())));
    /// assert_eq!(zero, Err(Error::InvalidArgument("bit count cannot be zero".to_string())));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn try_new_bits(bit_count: NonNegative) -> Result<Self> {
        Ok(LogicalType::Bits(Positive::new(bit_count).ok_or_else(
            || Error::InvalidArgument("bit count cannot be zero".to_string()),
        )?))
    }

    /// Returns a new Group stream type from the provided iterator of names and
    /// stream types. Returns an error when the values cannot be converted into
    /// valid names, or valid logical stream types as required by [`Group`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tydi::{Error, logical::{Group, LogicalType}};
    ///
    /// let group = LogicalType::try_new_group(
    ///     vec![
    ///         ("a", 4), // TryFrom<NonNegative> for LogicalType::Bits.
    ///         ("b", 12),
    ///     ]
    /// )?;
    ///
    /// assert!(match group {
    ///     LogicalType::Group(_) => true,
    ///     _ => false,
    /// });
    ///
    /// assert_eq!(
    ///     LogicalType::try_new_group(vec![("1badname", 4)]),
    ///     Err(Error::InvalidArgument("name cannot start with a digit".to_string()))
    /// );
    /// assert_eq!(
    ///     LogicalType::try_new_group(vec![("good_name", 0)]),
    ///     Err(Error::InvalidArgument("bit count cannot be zero".to_string()))
    /// );
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// [`Group`]: ./struct.Group.html
    pub fn try_new_group(
        db: &dyn Ir,
        parent_id: Option<PathName>,
        group: impl IntoIterator<
            Item = (
                impl TryInto<Name, Error = impl Into<Box<dyn error::Error>>>,
                Id<LogicalType>,
            ),
        >,
    ) -> Result<Self> {
        Group::try_new(db, parent_id, group).map(Into::into)
    }

    pub fn try_new_union(
        db: &dyn Ir,
        parent_id: Option<PathName>,
        union: impl IntoIterator<
            Item = (
                impl TryInto<Name, Error = impl Into<Box<dyn error::Error>>>,
                Id<LogicalType>,
            ),
        >,
    ) -> Result<Self> {
        Union::try_new(db, parent_id, union).map(Into::into)
    }

    /// Returns true if this logical stream consists of only element-
    /// manipulating stream types. This recursively checks all inner stream
    /// types.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tydi::logical::LogicalType;
    ///
    /// assert!(LogicalType::Null.is_element_only());
    /// assert!(LogicalType::try_new_bits(3)?.is_element_only());
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn is_element_only(&self, db: &dyn Ir) -> bool {
        match self {
            LogicalType::Null | LogicalType::Bits(_) => true,
            LogicalType::Group(Group(fields)) | LogicalType::Union(Union(fields)) => {
                fields.into_iter().all(|field_id| {
                    db.lookup_intern_field(field_id.clone())
                        .typ(db)
                        .is_element_only(db)
                })
            }
            LogicalType::Stream(stream) => stream.get(db).data(db).is_element_only(db),
        }
    }

    /// Flattens a logical stream type consisting of Null, Bits, Group and
    /// Union stream types into a [`Fields`].
    ///
    /// [Reference](https://abs-tudelft.github.io/tydi/specification/logical.html#field-conversion-function)
    ///
    /// [`Fields`]: ./struct.Fields.html
    pub(crate) fn fields(&self, db: &dyn Ir) -> Fields {
        let mut fields = Fields::new_empty();
        match self {
            LogicalType::Null | LogicalType::Stream(_) => fields,
            LogicalType::Bits(b) => {
                fields.insert(PathName::new_empty(), *b).unwrap();
                fields
            }
            LogicalType::Group(group) => {
                for field in group.fields(db).iter() {
                    field
                        .typ(db)
                        .fields(db)
                        .iter()
                        .for_each(|(path_name, bit_count)| {
                            fields
                                .insert(path_name.with_parents(field.name().clone()), *bit_count)
                                .unwrap();
                        })
                }
                fields
            }
            LogicalType::Union(union) => {
                if let Some(tag) = union.tag() {
                    fields
                        .insert(PathName::try_new(vec!["tag"]).unwrap(), tag)
                        .unwrap();
                }
                let b = union.fields(db).iter().fold(0, |acc, field| {
                    acc.max(
                        field
                            .typ(db)
                            .fields(db)
                            .values()
                            .fold(0, |acc, count| acc.max(count.get())),
                    )
                });
                if b > 0 {
                    fields
                        .insert(
                            PathName::try_new(vec!["union"]).unwrap(),
                            BitCount::new(b).unwrap(),
                        )
                        .unwrap();
                }
                fields
            }
        }
    }

    /// Splits a logical stream type into simplified stream types.
    ///
    /// [Reference](https://abs-tudelft.github.io/tydi/specification/logical.html#split-function)
    pub(crate) fn split_streams(&self, db: &dyn Ir) -> SplitStreams {
        fn split_fields(
            db: &dyn Ir,
            fields: Arc<Vec<LogicalField>>,
        ) -> (Vec<LogicalField>, IndexMap<PathName, Stream>) {
            let signals = fields
                .iter()
                .map(|field| {
                    LogicalField::new(
                        field.name().clone(),
                        field.typ(db).split_streams(db).signals().intern(db),
                    )
                })
                .collect();
            let streams = fields
                .iter()
                .map(|field| {
                    field
                        .typ(db)
                        .split_streams(db)
                        .streams()
                        .map(|(path_name, stream)| {
                            (path_name.with_parents(field.name().clone()), stream.clone())
                        })
                        .collect::<Vec<_>>()
                })
                .flatten()
                .collect();
            (signals, streams)
        }

        match self {
            LogicalType::Null | LogicalType::Bits(_) => SplitStreams {
                signals: self.clone(),
                streams: IndexMap::new(),
            },
            LogicalType::Group(group) => {
                let (fields, streams) = split_fields(db, group.fields(db));
                SplitStreams {
                    signals: Group::new(db, fields).into(),
                    streams,
                }
            }
            LogicalType::Union(union) => {
                let (fields, streams) = split_fields(db, union.fields(db));
                SplitStreams {
                    signals: Union::new(db, fields).into(),
                    streams,
                }
            }
            LogicalType::Stream(stream_id) => stream_id.get(db).split_streams(db),
        }
    }
}

impl IsNull for LogicalType {
    /// Returns true if and only if this logical stream does not result in any
    /// signals.
    ///
    /// [Reference](https://abs-tudelft.github.io/tydi/specification/logical.html#null-detection-function)
    fn is_null(&self, db: &dyn Ir) -> bool {
        match self {
            LogicalType::Null => true,
            LogicalType::Group(Group(fields)) => fields
                .into_iter()
                .all(|field_id| db.lookup_intern_field(field_id.clone()).typ(db).is_null(db)),
            LogicalType::Union(Union(fields)) => {
                fields.len() == 1
                    && fields.into_iter().all(|field_id| {
                        db.lookup_intern_field(field_id.clone()).typ(db).is_null(db)
                    })
            }
            LogicalType::Stream(stream) => stream.is_null(db),
            LogicalType::Bits(_) => false,
        }
    }
}
