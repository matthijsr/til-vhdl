use std::str::FromStr;

use indexmap::IndexMap;

use crate::common::{
    logical::{Group, Union},
    physical::{Complexity, Fields, PhysicalStream},
    traits::Reverse,
    util::log2_ceil,
    BitCount, Error, Name, NonNegative, PathName, Positive, PositiveReal, Result,
};

use super::{LogicalType, Signals};

/// Direction of a stream.
///
/// [Reference]
///
/// [Reference]: https://abs-tudelft.github.io/tydi/specification/logical.html#stream
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Direction {
    /// Forward indicates that the child stream flows in the same direction as
    /// its parent, complementing the data of its parent in some way.
    Forward,
    /// Reverse indicates that the child stream acts as a response channel for
    /// the parent stream. If there is no parent stream, Forward indicates that
    /// the stream flows in the natural source to sink direction of the logical
    /// stream, while Reverse indicates a control channel in the opposite
    /// direction. The latter may occur for instance when doing random read
    /// access to a memory; the first stream carrying the read commands then
    /// flows in the sink to source direction.
    Reverse,
}

impl Default for Direction {
    fn default() -> Self {
        Direction::Forward
    }
}

impl FromStr for Direction {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self> {
        match input {
            "Forward" => Ok(Direction::Forward),
            "Reverse" => Ok(Direction::Reverse),
            _ => Err(Error::InvalidArgument(format!(
                "{} is not a valid Direction",
                input
            ))),
        }
    }
}

impl Reverse for Direction {
    /// Reverse this direction.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tydi::{Reverse, Reversed, logical::Direction};
    ///
    /// let mut forward = Direction::Forward;
    /// let mut reverse = Direction::Reverse;
    ///
    /// forward.reverse();
    /// assert_eq!(forward, reverse);
    ///
    /// forward.reverse();
    /// assert_eq!(forward, reverse.reversed());
    /// ```
    fn reverse(&mut self) {
        *self = match self {
            Direction::Forward => Direction::Reverse,
            Direction::Reverse => Direction::Forward,
        };
    }
}

/// The synchronicity of the elements in the child stream with respect to the
/// elements in the parent stream.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Synchronicity {
    /// Indicating that there is a one-to-one relation between the parent and
    /// child elements, and the dimensionality information of the parent stream
    /// is redundantly carried by the child stream as well.
    Sync,
    /// Indicating that there is a one-to-one relation between the parent and
    /// child elements, and the dimensionality information of the parent stream
    /// is omitted in the child stream.
    Flatten,
    /// Desync may be used if the relation between the elements in the child
    /// and parent stream is dependent on context rather than the last flags
    /// in either stream.
    Desync,
    /// FlatDesync, finally, does the same thing as Desync, but also strips the
    /// dimensionality information from the parent. This means there the
    /// relation between the two streams, if any, is fully user-defined.
    FlatDesync,
}

impl Default for Synchronicity {
    fn default() -> Self {
        Synchronicity::Sync
    }
}

impl FromStr for Synchronicity {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self> {
        match input {
            "Sync" => Ok(Synchronicity::Sync),
            "Flatten" => Ok(Synchronicity::Flatten),
            "Desync" => Ok(Synchronicity::Desync),
            "FlatDesync" => Ok(Synchronicity::FlatDesync),
            _ => Err(Error::InvalidArgument(format!(
                "{} is not a valid Synchronicity",
                input
            ))),
        }
    }
}

/// The stream-manipulating logical stream type.
///
/// Defines a new physical stream.
///
/// [Reference](https://abs-tudelft.github.io/tydi/specification/logical.html#stream)
#[derive(Debug, Clone, PartialEq)]
pub struct Stream {
    /// Logical stream type of data elements carried by this stream.
    ///
    /// Any logical stream type representing the data type carried by the
    /// logical stream.
    data: Box<LogicalType>,
    /// Throughput ratio of the stream.
    ///
    /// Positive real number, representing the minimum number of elements that
    /// should be transferrable on the child stream per element in the parent
    /// stream, or if there is no parent stream, the minimum number of elements
    /// that should be transferrable per clock cycle.
    throughput: PositiveReal,
    /// Dimensionality of the stream.
    ///
    /// Nonnegative integer specifying the dimensionality of the child
    /// stream with respect to the parent stream (with no parent, it is the
    /// initial value).
    dimensionality: NonNegative,
    /// Synchronicity of the stream.
    ///
    /// The synchronicity of the d-dimensional elements in the child stream
    /// with respect to the elements in the parent stream.
    synchronicity: Synchronicity,
    /// Complexity level of the stream.
    ///
    /// The complexity number for the physical stream interface, as defined
    /// in the physical stream specification.
    complexity: Complexity,
    /// Direction of the stream.
    ///
    /// The direction of the stream. If there is no parent stream, this
    /// specifies the direction with respect to the natural direction of
    /// the stream (source to sink).
    direction: Direction,
    /// Logical stream type of (optional) user data carried by this stream.
    ///
    /// An optional logical stream type consisting of only
    /// element-manipulating nodes, representing the user data carried by
    /// this logical stream.
    user: Option<Box<LogicalType>>,
    /// Stream carries extra information.
    ///
    /// Keep specifies whether the stream carries "extra" information
    /// beyond the data and user signal payloads. x is normally false,
    /// which implies that the Stream node will not result in a physical
    /// stream if both its data and user signals would be empty according
    /// to the rest of this specification; it is effectively optimized
    /// away. Setting keep to true simply overrides this behavior.
    keep: bool,
}

impl Reverse for Stream {
    /// Reverse the direction of this stream.
    ///
    /// This flips the [`Direction`] of the stream.
    ///
    /// [`Direction`]: ./enum.Direction.html
    fn reverse(&mut self) {
        self.direction.reverse();
    }
}

impl Stream {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        data: LogicalType,
        throughput: PositiveReal,
        dimensionality: NonNegative,
        synchronicity: Synchronicity,
        complexity: impl Into<Complexity>,
        direction: Direction,
        user: Option<LogicalType>,
        keep: bool,
    ) -> Self {
        Stream {
            data: Box::new(data),
            throughput,
            dimensionality,
            synchronicity,
            complexity: complexity.into(),
            direction,
            user: user.map(Box::new),
            keep,
        }
    }

    pub fn new_basic(data: LogicalType) -> Self {
        Stream {
            data: Box::new(data),
            throughput: PositiveReal::new(1.).unwrap(),
            dimensionality: 0,
            synchronicity: Synchronicity::Sync,
            complexity: Complexity::default(),
            direction: Direction::Forward,
            user: None,
            keep: false,
        }
    }

    pub fn data(&self) -> &LogicalType {
        &self.data
    }

    /// Returns the direction of this stream.
    pub fn direction(&self) -> Direction {
        self.direction
    }

    /// Returns the synchronicity of this stream.
    pub fn synchronicity(&self) -> Synchronicity {
        self.synchronicity
    }

    /// Returns the dimensionality of this stream.
    pub fn dimensionality(&self) -> NonNegative {
        self.dimensionality
    }

    /// Returns the throughput ratio of this stream.
    pub fn throughput(&self) -> PositiveReal {
        self.throughput
    }

    /// Returns true if this stream is null i.e. it results in no signals.
    ///
    /// [Reference](https://abs-tudelft.github.io/tydi/specification/logical.html#null-detection-function)
    pub fn is_null(&self) -> bool {
        self.data.is_null()
            && (self.user.is_some() && self.user.as_ref().unwrap().is_null())
            && !self.keep
    }

    /// Set the throughput ratio of this stream.
    fn set_throughput(&mut self, throughput: PositiveReal) {
        self.throughput = throughput;
    }

    /// Set the synchronicity of this stream.
    fn set_synchronicity(&mut self, synchronicity: Synchronicity) {
        self.synchronicity = synchronicity;
    }

    /// Set the dimensionality of this stream.
    fn set_dimensionality(&mut self, dimensionality: NonNegative) {
        self.dimensionality = dimensionality;
    }
}

impl From<Stream> for LogicalType {
    /// Wraps this stream in a [`LogicalType`].
    ///
    /// [`LogicalType`]: ./enum.LogicalType.html
    fn from(stream: Stream) -> Self {
        LogicalType::Stream(stream)
    }
}

/// An element stream with a path name and LogicalType. Contains no nested
/// streams.
#[derive(Debug, Clone, PartialEq)]
pub struct ElementStream {
    path_name: PathName,
    logical_type: LogicalType,
}

impl ElementStream {
    pub fn path_name(&self) -> &[Name] {
        self.path_name.as_ref()
    }
    /// Returns the LogicalType of this element. Contains no nested streams.
    pub fn logical_type(&self) -> &LogicalType {
        &self.logical_type
    }
    /// Return all fields in this element stream
    pub fn fields(&self) -> Fields {
        let mut fields = Fields::new_empty();
        match &self.logical_type {
            LogicalType::Stream(stream) => match &*stream.data {
                LogicalType::Null => fields,
                LogicalType::Bits(b) => {
                    fields.insert(self.path_name.clone(), *b).unwrap();
                    fields
                }
                LogicalType::Group(Group(inner)) => {
                    inner.iter().for_each(|(name, stream)| {
                        stream.fields().iter().for_each(|(path_name, bit_count)| {
                            fields
                                .insert(
                                    path_name
                                        .with_parent(name.clone())
                                        .with_parents(self.path_name.clone()),
                                    *bit_count,
                                )
                                .unwrap();
                        })
                    });
                    fields
                }
                LogicalType::Union(Union(inner)) => {
                    if inner.len() > 1 {
                        fields
                            .insert(
                                PathName::try_new(vec!["tag"])
                                    .unwrap()
                                    .with_parents(self.path_name.clone()),
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
                                PathName::try_new(vec!["union"])
                                    .unwrap()
                                    .with_parents(self.path_name.clone()),
                                BitCount::new(b).unwrap(),
                            )
                            .unwrap();
                    }
                    fields
                }
                LogicalType::Stream(_) => unreachable!(),
            },
            _ => unreachable!(),
        }
    }
}

impl From<ElementStream> for PhysicalStream {
    fn from(element_stream: ElementStream) -> PhysicalStream {
        match element_stream.logical_type {
            LogicalType::Stream(stream) => PhysicalStream::new(
                stream.data.fields(),
                Positive::new(stream.throughput.get().ceil() as NonNegative).unwrap(),
                stream.dimensionality,
                stream.complexity,
                stream
                    .user
                    .map(|stream| stream.fields())
                    .unwrap_or_else(Fields::new_empty),
            ),
            _ => unreachable!(),
        }
    }
}

/// A split item is either an async signal (outside streamspace) or an element
/// stream (no nested streams).
#[derive(Debug, Clone, PartialEq)]
pub enum LogicalSplitItem {
    Signals(Signals),
    Stream(ElementStream),
}

impl LogicalSplitItem {
    pub fn is_stream(&self) -> bool {
        match self {
            LogicalSplitItem::Signals(_) => false,
            LogicalSplitItem::Stream(_) => true,
        }
    }
    pub fn is_signals(&self) -> bool {
        match self {
            LogicalSplitItem::Signals(_) => true,
            LogicalSplitItem::Stream(_) => false,
        }
    }
    pub fn logical_type(&self) -> &LogicalType {
        match self {
            LogicalSplitItem::Signals(signals) => signals.logical_type(),
            LogicalSplitItem::Stream(stream) => stream.logical_type(),
        }
    }
    pub fn fields(&self) -> Fields {
        match self {
            LogicalSplitItem::Signals(signals) => signals.fields(),
            LogicalSplitItem::Stream(stream) => stream.fields(),
        }
    }
}

/// A split item is either an async signal (outside streamspace) or a physical
/// stream.
#[derive(Debug, Clone, PartialEq)]
pub enum PhysicalSplitItem {
    Signals(Signals),
    Stream(PhysicalStream),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SplitStreams {
    signals: LogicalType,
    streams: IndexMap<PathName, LogicalType>,
}

pub(crate) trait Splits {
    fn split_streams(&self) -> SplitStreams;
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

impl Splits for Stream {
    fn split_streams(&self) -> SplitStreams {
        let mut streams = IndexMap::new();

        let split = self.data.split_streams();
        let (element, rest) = (split.signals, split.streams);
        if !element.is_null()
            || (self.user.is_some() && !self.user.as_ref().unwrap().is_null())
            || self.keep
        {
            streams.insert(
                PathName::new_empty(),
                // todo: add method
                Stream::new(
                    element,
                    self.throughput,
                    self.dimensionality,
                    self.synchronicity,
                    self.complexity.clone(),
                    self.direction,
                    self.user.clone().map(|stream| *stream),
                    self.keep,
                )
                .into(),
            );
        }

        streams.extend(rest.into_iter().map(|(name, stream)| match stream {
            LogicalType::Stream(mut stream) => {
                if self.direction == Direction::Reverse {
                    stream.reverse();
                }
                if self.synchronicity == Synchronicity::Flatten
                    || self.synchronicity == Synchronicity::FlatDesync
                {
                    stream.set_synchronicity(Synchronicity::FlatDesync);
                }
                if stream.synchronicity != Synchronicity::Flatten
                    && self.synchronicity != Synchronicity::FlatDesync
                {
                    stream.set_dimensionality(stream.dimensionality + self.dimensionality);
                };
                stream.set_throughput(stream.throughput * self.throughput);
                (name, stream.into())
            }
            _ => unreachable!(),
        }));

        SplitStreams {
            signals: LogicalType::Null,
            streams,
        }
    }
}
