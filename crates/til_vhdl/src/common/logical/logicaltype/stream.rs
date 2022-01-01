use std::convert::{TryFrom, TryInto};
use std::hash::{Hash, Hasher};
use std::ops::Mul;
use std::str::FromStr;

use indexmap::IndexMap;
use salsa::Database;
use tydi_common::name::PathName;
use tydi_common::numbers::Positive;
use tydi_common::traits::Reverse;
use tydi_intern::Id;

use crate::common::logical::logical_stream::{LogicalStream, SynthesizeLogicalStream};
use crate::common::logical::split_streams::SplitsStreams;
use crate::common::physical::complexity::Complexity;
use crate::common::physical::stream::PhysicalStream;
use crate::ir::{GetSelf, InternSelf, Ir};
use tydi_common::{
    error::{Error, Result},
    numbers::{NonNegative, PositiveReal},
};

use super::{IsNull, LogicalType, SplitStreams};

/// Floats cannot be hashed or consistently reproduced on all hardware
///
/// Throughput is stored as a string to ensure consistency between IR and back-end
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Throughput(String);

impl Throughput {
    pub fn try_new(throughput: String) -> Result<Self> {
        match throughput.parse::<f64>() {
            Ok(val) if PositiveReal::new(val).is_ok() => Ok(Throughput(throughput)),
            _ => Err(Error::InvalidArgument(format!(
                "{} is not a positive real number",
                throughput
            ))),
        }
    }

    pub fn as_real(&self) -> PositiveReal {
        PositiveReal::new(self.0.parse().unwrap()).unwrap()
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn non_negative(&self) -> NonNegative {
        self.as_real().get().ceil() as NonNegative
    }

    pub fn positive(&self) -> Positive {
        Positive::new(self.non_negative()).unwrap()
    }
}

impl TryFrom<&str> for Throughput {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        Throughput::try_new(value.to_string())
    }
}

impl TryFrom<String> for Throughput {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        Throughput::try_new(value)
    }
}

impl Mul for Throughput {
    type Output = Throughput;

    fn mul(self, rhs: Self) -> Self::Output {
        Throughput::try_new((self.as_real() * rhs.as_real()).get().to_string()).unwrap()
    }
}

/// The stream-manipulating logical stream type.
///
/// Defines a new physical stream.
///
/// [Reference](https://abs-tudelft.github.io/tydi/specification/logical.html#stream)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Stream {
    /// Logical stream type of data elements carried by this stream.
    ///
    /// Any logical stream type representing the data type carried by the
    /// logical stream.
    data: Id<LogicalType>,
    /// Throughput ratio of the stream.
    ///
    /// Positive real number, representing the minimum number of elements that
    /// should be transferrable on the child stream per element in the parent
    /// stream, or if there is no parent stream, the minimum number of elements
    /// that should be transferrable per clock cycle.
    throughput: Throughput,
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
    user: Id<LogicalType>,
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

impl Stream {
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        db: &dyn Ir,
        data: Id<LogicalType>,
        throughput: impl TryInto<Throughput, Error = Error>,
        dimensionality: NonNegative,
        synchronicity: Synchronicity,
        complexity: impl Into<Complexity>,
        direction: Direction,
        user: Id<LogicalType>,
        keep: bool,
    ) -> Result<Id<Self>> {
        Ok(db.intern_stream(Stream {
            data: data,
            throughput: throughput.try_into()?,
            dimensionality,
            synchronicity,
            complexity: complexity.into(),
            direction,
            user: user,
            keep,
        }))
    }

    pub(crate) fn new(
        data: Id<LogicalType>,
        throughput: Throughput,
        dimensionality: NonNegative,
        synchronicity: Synchronicity,
        complexity: impl Into<Complexity>,
        direction: Direction,
        user: Id<LogicalType>,
        keep: bool,
    ) -> Self {
        Stream {
            data: data,
            throughput: throughput,
            dimensionality,
            synchronicity,
            complexity: complexity.into(),
            direction,
            user: user,
            keep,
        }
    }

    pub fn data(&self, db: &dyn Ir) -> LogicalType {
        db.lookup_intern_type(self.data)
    }

    pub fn data_id(&self) -> Id<LogicalType> {
        self.data
    }

    pub fn user(&self, db: &dyn Ir) -> LogicalType {
        db.lookup_intern_type(self.user)
    }

    pub fn user_id(&self) -> Id<LogicalType> {
        self.user
    }

    /// Returns the direction of this stream.
    pub fn direction(&self) -> Direction {
        self.direction
    }

    /// Returns the synchronicity of this stream.
    pub fn synchronicity(&self) -> Synchronicity {
        self.synchronicity
    }

    /// Returns whether the stream synchronicity should be flattened
    pub fn flattens(&self) -> bool {
        self.synchronicity == Synchronicity::Flatten
            || self.synchronicity == Synchronicity::FlatDesync
    }

    /// Returns the dimensionality of this stream.
    pub fn dimensionality(&self) -> NonNegative {
        self.dimensionality
    }

    /// Returns the throughput ratio of this stream.
    pub fn throughput(&self) -> Throughput {
        self.throughput.clone()
    }

    // Returns the complexity of this stream.
    pub fn complexity(&self) -> Complexity {
        self.complexity.clone()
    }

    // Returns the keep flag of this stream.
    pub fn keep(&self) -> bool {
        self.keep
    }

    // Converts this Stream type into a Physical Stream.
    pub fn physical(&self, db: &dyn Ir) -> PhysicalStream {
        PhysicalStream::new(
            self.data(db).fields(db),
            self.throughput().positive(),
            self.dimensionality(),
            self.complexity(),
            self.user(db).fields(db),
        )
    }

    /// Set the throughput ratio of this stream.
    pub(crate) fn set_throughput(&mut self, throughput: Throughput) {
        self.throughput = throughput;
    }

    /// Set the synchronicity of this stream.
    pub(crate) fn set_synchronicity(&mut self, synchronicity: Synchronicity) {
        self.synchronicity = synchronicity;
    }

    /// Set the dimensionality of this stream.
    pub(crate) fn set_dimensionality(&mut self, dimensionality: NonNegative) {
        self.dimensionality = dimensionality;
    }
}

impl SynthesizeLogicalStream for Id<Stream> {
    fn synthesize(&self, db: &dyn Ir) -> LogicalStream {
        let split = self.split_streams(db);
        let (signals, rest) = (split.signals().get(db).fields(db), split.streams());
        LogicalStream::new(
            signals,
            rest.into_iter()
                .map(|(path_name, stream)| (path_name.clone(), stream.get(db).physical(db)))
                .collect(),
        )
    }
}

impl SplitsStreams for Id<Stream> {
    fn split_streams(&self, db: &dyn Ir) -> SplitStreams {
        let this_stream = self.get(db);
        let split = this_stream.data.split_streams(db);
        let mut streams = IndexMap::new();
        let (element, rest) = (split.signals(), split.streams());
        if this_stream.keep() || !element.is_null(db) || !this_stream.user_id().is_null(db) {
            streams.insert(
                PathName::new_empty(),
                Stream::new(
                    element,
                    this_stream.throughput(),
                    this_stream.dimensionality(),
                    this_stream.synchronicity(),
                    this_stream.complexity().clone(),
                    this_stream.direction(),
                    this_stream.user_id(),
                    this_stream.keep(),
                )
                .intern(db),
            );
        }

        streams.extend(rest.into_iter().map(|(name, stream_id)| {
            let mut stream = stream_id.get(db);
            if this_stream.direction() == Direction::Reverse {
                stream.reverse();
            }
            if this_stream.flattens() {
                stream.set_synchronicity(Synchronicity::FlatDesync);
            } else {
                stream.set_dimensionality(stream.dimensionality() + this_stream.dimensionality());
            }
            stream.set_throughput(stream.throughput() * this_stream.throughput());

            (name.clone(), stream.intern(db))
        }));

        SplitStreams::new(db.intern_type(LogicalType::Null), streams)
    }
}

impl IsNull for Stream {
    /// Returns true if this stream is null i.e. it results in no signals.
    ///
    /// [Reference](https://abs-tudelft.github.io/tydi/specification/logical.html#null-detection-function)
    fn is_null(&self, db: &dyn Ir) -> bool {
        self.data(db).is_null(db) && self.user(db).is_null(db) && !self.keep
    }
}

impl IsNull for Id<Stream> {
    fn is_null(&self, db: &dyn Ir) -> bool {
        db.lookup_intern_stream(*self).is_null(db)
    }
}

impl From<Id<Stream>> for LogicalType {
    /// Wraps this stream in a [`LogicalType`].
    ///
    /// [`LogicalType`]: ./enum.LogicalType.html
    fn from(stream: Id<Stream>) -> Self {
        LogicalType::Stream(stream)
    }
}

impl Reverse for Stream {
    fn reverse(&mut self) {
        match self.direction() {
            Direction::Forward => self.direction = Direction::Reverse,
            Direction::Reverse => self.direction = Direction::Forward,
        }
    }
}

/// The synchronicity of the elements in the child stream with respect to the
/// elements in the parent stream.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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

/// Direction of a stream.
///
/// [Reference]
///
/// [Reference]: https://abs-tudelft.github.io/tydi/specification/logical.html#stream
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
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
