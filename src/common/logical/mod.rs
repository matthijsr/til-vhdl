use std::{
    convert::{TryFrom, TryInto},
    error,
};

use crate::common::{util::log2_ceil, BitCount};

use super::{physical::Fields, Error, Name, NonNegative, PathName, Positive, Result};

pub(crate) use group::*;
pub(crate) mod group;
use indexmap::IndexMap;
pub(crate) use union::*;
pub(crate) mod union;
pub(crate) use stream::*;
pub(crate) mod stream;
pub(crate) use signals::*;
pub(crate) mod signals;

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
#[derive(Debug, Clone, PartialEq)]
pub enum LogicalType {
    /// The Null stream type indicates the transferrence of one-valued data: its
    /// only valid value is ∅ (null).
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
    Stream(Stream),
}

impl TryFrom<NonNegative> for LogicalType {
    type Error = Error;

    /// Returns a new Bits stream type with the provided bit count as number of
    /// bits. Returns an error when the bit count is zero.
    fn try_from(bit_count: NonNegative) -> Result<Self> {
        LogicalType::try_new_bits(bit_count)
    }
}

impl From<Positive> for LogicalType {
    fn from(bit_count: Positive) -> Self {
        LogicalType::Bits(bit_count)
    }
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
        group: impl IntoIterator<
            Item = (
                impl TryInto<Name, Error = impl Into<Box<dyn error::Error>>>,
                impl TryInto<LogicalType, Error = impl Into<Box<dyn error::Error>>>,
            ),
        >,
    ) -> Result<Self> {
        Group::try_new(group).map(Into::into)
    }

    pub fn try_new_union(
        union: impl IntoIterator<
            Item = (
                impl TryInto<Name, Error = impl Into<Box<dyn error::Error>>>,
                impl TryInto<LogicalType, Error = impl Into<Box<dyn error::Error>>>,
            ),
        >,
    ) -> Result<Self> {
        Union::try_new(union).map(Into::into)
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
    pub fn is_element_only(&self) -> bool {
        match self {
            LogicalType::Null | LogicalType::Bits(_) => true,
            LogicalType::Group(Group(fields)) | LogicalType::Union(Union(fields)) => {
                fields.values().all(|stream| stream.is_element_only())
            }
            LogicalType::Stream(stream) => stream.data.is_element_only(),
        }
    }

    /// Returns true if and only if this logical stream does not result in any
    /// signals.
    ///
    /// [Reference](https://abs-tudelft.github.io/tydi/specification/logical.html#null-detection-function)
    pub fn is_null(&self) -> bool {
        match self {
            LogicalType::Null => true,
            LogicalType::Group(Group(fields)) => fields.values().all(|stream| stream.is_null()),
            LogicalType::Union(Union(fields)) => {
                fields.len() == 1 && fields.values().all(|stream| stream.is_null())
            }
            LogicalType::Stream(stream) => stream.is_null(),
            LogicalType::Bits(_) => false,
        }
    }

    /// Flattens a logical stream type consisting of Null, Bits, Group and
    /// Union stream types into a [`Fields`].
    ///
    /// [Reference](https://abs-tudelft.github.io/tydi/specification/logical.html#field-conversion-function)
    ///
    /// [`Fields`]: ./struct.Fields.html
    pub(crate) fn fields(&self) -> Fields {
        let mut fields = Fields::new_empty();
        match self {
            LogicalType::Null | LogicalType::Stream(_) => fields,
            LogicalType::Bits(b) => {
                fields.insert(PathName::new_empty(), *b).unwrap();
                fields
            }
            LogicalType::Group(Group(inner)) => {
                inner.iter().for_each(|(name, stream)| {
                    stream.fields().iter().for_each(|(path_name, bit_count)| {
                        fields
                            .insert(path_name.with_parent(name.clone()), *bit_count)
                            .unwrap();
                    })
                });
                fields
            }
            LogicalType::Union(Union(inner)) => {
                if inner.len() > 1 {
                    fields
                        .insert(
                            PathName::try_new(vec!["tag"]).unwrap(),
                            BitCount::new(log2_ceil(
                                BitCount::new(inner.len() as NonNegative).unwrap(),
                            ))
                            .unwrap(),
                        )
                        .unwrap();
                }
                let b = inner.iter().fold(0, |acc, (_, stream)| {
                    acc.max(
                        stream
                            .fields()
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

    pub(crate) fn synthesize(&self) -> LogicalStream {
        let split = self.split_streams();
        let (signals, rest) = (split.signals.fields(), split.streams);
        LogicalStream {
            signals,
            streams: rest
                .into_iter()
                .map(|(path_name, stream)| match stream {
                    LogicalType::Stream(stream) => (
                        path_name,
                        PhysicalStream::new(
                            stream.data.fields(),
                            Positive::new(stream.throughput.get().ceil() as NonNegative).unwrap(),
                            stream.dimensionality,
                            stream.complexity,
                            stream
                                .user
                                .map(|stream| stream.fields())
                                .unwrap_or_else(Fields::new_empty),
                        ),
                    ),
                    _ => unreachable!(),
                })
                .collect(),
        }
    }

    pub fn compatible(&self, other: &LogicalType) -> bool {
        self == other
            || match other {
                LogicalType::Stream(other) => match self {
                    LogicalType::Stream(stream) => {
                        stream.data.compatible(&other.data) && stream.complexity < other.complexity
                    }
                    _ => false,
                },
                _ => false,
            }
            || match self {
                LogicalType::Group(Group(source)) | LogicalType::Union(Union(source)) => {
                    match other {
                        LogicalType::Group(Group(sink)) | LogicalType::Union(Union(sink)) => {
                            source.len() == sink.len()
                                && source.iter().zip(sink.iter()).all(
                                    |((name, stream), (name_, stream_))| {
                                        name == name_ && stream.compatible(stream_)
                                    },
                                )
                        }
                        _ => false,
                    }
                }
                _ => false,
            }
    }

    pub fn split(&self) -> std::vec::IntoIter<LogicalSplitItem> {
        let split_streams = self.split_streams();
        let (signals, streams) = (split_streams.signals, split_streams.streams);
        let mut map = Vec::with_capacity(streams.len() + 1);

        if !signals.is_null() {
            map.push(LogicalSplitItem::Signals(Signals(signals)));
        }

        map.extend(streams.into_iter().map(|(path_name, logical_type)| {
            LogicalSplitItem::Stream(ElementStream {
                path_name,
                logical_type,
            })
        }));
        map.into_iter()
    }

    pub fn physical(&self) -> std::vec::IntoIter<PhysicalSplitItem> {
        self.split()
            .map(|item| match item {
                LogicalSplitItem::Signals(signals) => PhysicalSplitItem::Signals(signals),
                LogicalSplitItem::Stream(stream) => PhysicalSplitItem::Stream(stream.into()),
            })
            .collect::<Vec<_>>()
            .into_iter()
    }
}

impl Splits for LogicalType {
    /// Splits a logical stream type into simplified stream types.
    ///
    /// [Reference](https://abs-tudelft.github.io/tydi/specification/logical.html#split-function)
    fn split_streams(&self) -> SplitStreams {
        match self {
            LogicalType::Stream(stream_in) => {
                let mut streams = IndexMap::new();

                let split = stream_in.data.split_streams();
                let (element, rest) = (split.signals, split.streams);
                if !element.is_null()
                    || (stream_in.user.is_some() && !stream_in.user.as_ref().unwrap().is_null())
                    || stream_in.keep
                {
                    streams.insert(
                        PathName::new_empty(),
                        // todo: add method
                        Stream::new(
                            element,
                            stream_in.throughput,
                            stream_in.dimensionality,
                            stream_in.synchronicity,
                            stream_in.complexity.clone(),
                            stream_in.direction,
                            stream_in.user.clone().map(|stream| *stream),
                            stream_in.keep,
                        )
                        .into(),
                    );
                }

                streams.extend(rest.into_iter().map(|(name, stream)| match stream {
                    LogicalType::Stream(mut stream) => {
                        if stream_in.direction == Direction::Reverse {
                            stream.reverse();
                        }
                        if stream_in.synchronicity == Synchronicity::Flatten
                            || stream_in.synchronicity == Synchronicity::FlatDesync
                        {
                            stream.set_synchronicity(Synchronicity::FlatDesync);
                        }
                        if stream.synchronicity != Synchronicity::Flatten
                            && stream_in.synchronicity != Synchronicity::FlatDesync
                        {
                            stream.set_dimensionality(
                                stream.dimensionality + stream_in.dimensionality,
                            );
                        };
                        stream.set_throughput(stream.throughput * stream_in.throughput);
                        (name, stream.into())
                    }
                    _ => unreachable!(),
                }));

                SplitStreams {
                    signals: LogicalType::Null,
                    streams,
                }
            }
            LogicalType::Null | LogicalType::Bits(_) => SplitStreams {
                signals: self.clone(),
                streams: IndexMap::new(),
            },
            LogicalType::Group(Group(fields)) | LogicalType::Union(Union(fields)) => {
                let signals = fields
                    .into_iter()
                    .map(|(name, stream)| (name.clone(), stream.split_streams().signals))
                    .collect();

                SplitStreams {
                    signals: match self {
                        LogicalType::Group(_) => LogicalType::Group(Group(signals)),
                        LogicalType::Union(_) => LogicalType::Union(Union(signals)),
                        _ => unreachable!(),
                    },
                    streams: fields
                        .into_iter()
                        .map(|(name, stream)| {
                            stream.split_streams().streams.into_iter().map(
                                move |(mut path_name, stream_)| {
                                    path_name.push(name.clone());
                                    (path_name, stream_)
                                },
                            )
                        })
                        .flatten()
                        .collect(),
                }
            }
        }
    }
}
