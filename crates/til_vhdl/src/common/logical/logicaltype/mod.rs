use std::{convert::TryInto, error};

use crate::{
    common::physical::{fields::Fields, stream::PhysicalStream},
    ir::{GetSelf, Identifier, IntoVhdl, Ir},
};
use indexmap::IndexMap;
use tydi_common::{
    error::{Error, Result},
    name::{Name, PathName},
    numbers::{BitCount, NonNegative, Positive},
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
    streams: IndexMap<PathName, LogicalType>,
}

impl SplitStreams {
    pub fn streams(&self) -> impl Iterator<Item = (&PathName, &LogicalType)> {
        self.streams.iter()
    }
    pub fn signal(&self) -> &LogicalType {
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
        parent_id: Option<Identifier>,
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
        parent_id: Option<Identifier>,
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

    // // Splits a logical stream type into simplified stream types.
    // //
    // // [Reference](https://abs-tudelft.github.io/tydi/specification/logical.html#split-function)
    // pub(crate) fn split_streams(&self, db: &dyn Ir) -> SplitStreams {
    //     match self {
    //         LogicalType::Stream(stream_in) => {
    //             let stream_in = stream_in.get(db);
    //             let mut streams = IndexMap::new();

    //             let split = stream_in.data(db).split_streams(db);
    //             let (element, rest) = (split.signals, split.streams);
    //             if !element.is_null(db) || !stream_in.user(db).is_null(db) || stream_in.keep() {
    //                 streams.insert(
    //                     PathName::new_empty(),
    //                     // todo: add method
    //                     Stream::new(
    //                         element,
    //                         stream_in.throughput(),
    //                         stream_in.dimensionality,
    //                         stream_in.synchronicity,
    //                         stream_in.complexity.clone(),
    //                         stream_in.direction,
    //                         stream_in.user.clone().map(|stream| *stream),
    //                         stream_in.keep,
    //                     )
    //                     .into(),
    //                 );
    //             }

    //             streams.extend(rest.into_iter().map(|(name, stream)| match stream {
    //                 LogicalType::Stream(mut stream) => {
    //                     if stream_in.direction == Direction::Reverse {
    //                         stream.reverse();
    //                     }
    //                     if stream_in.synchronicity == Synchronicity::Flatten
    //                         || stream_in.synchronicity == Synchronicity::FlatDesync
    //                     {
    //                         stream.set_synchronicity(Synchronicity::FlatDesync);
    //                     }
    //                     if stream.synchronicity != Synchronicity::Flatten
    //                         && stream_in.synchronicity != Synchronicity::FlatDesync
    //                     {
    //                         stream.set_dimensionality(
    //                             stream.dimensionality + stream_in.dimensionality,
    //                         );
    //                     };
    //                     stream.set_throughput(stream.throughput * stream_in.throughput);
    //                     (name, stream.into())
    //                 }
    //                 _ => unreachable!(),
    //             }));

    //             SplitStreams {
    //                 signals: LogicalType::Null,
    //                 streams,
    //             }
    //         }
    //         LogicalType::Null | LogicalType::Bits(_) => SplitStreams {
    //             signals: self.clone(),
    //             streams: IndexMap::new(),
    //         },
    //         LogicalType::Group(Group(fields)) | LogicalType::Union(Union(fields)) => {
    //             let signals = fields
    //                 .into_iter()
    //                 .map(|(name, stream)| (name.clone(), stream.split_streams().signals))
    //                 .collect();

    //             SplitStreams {
    //                 signals: match self {
    //                     LogicalType::Group(_) => LogicalType::Group(Group(signals)),
    //                     LogicalType::Union(_) => LogicalType::Union(Union(signals)),
    //                     _ => unreachable!(),
    //                 },
    //                 streams: fields
    //                     .into_iter()
    //                     .map(|(name, stream)| {
    //                         stream.split_streams().streams.into_iter().map(
    //                             move |(mut path_name, stream_)| {
    //                                 path_name.push(name.clone());
    //                                 (path_name, stream_)
    //                             },
    //                         )
    //                     })
    //                     .flatten()
    //                     .collect(),
    //             }
    //         }
    //     }
    // }
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
