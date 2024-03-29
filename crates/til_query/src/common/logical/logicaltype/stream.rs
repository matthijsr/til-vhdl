use core::fmt;
use std::{convert::TryFrom, hash::Hash, ops::Mul, str::FromStr};

use tydi_common::{
    error::{Error, Result, TryResult},
    map::InsertionOrderedMap,
    name::{Name, PathName},
    numbers::{BitCount, NonNegative, Positive, PositiveReal},
    traits::Reverse,
};
use tydi_intern::Id;

use crate::{
    common::{
        logical::{
            logical_stream::{LogicalStream, SynthesizeLogicalStream, TypedStream},
            split_streams::SplitsStreams,
            type_hierarchy::TypeHierarchy,
            type_reference::TypeReference,
        },
        physical::{complexity::Complexity, stream::PhysicalStream},
        stream_direction::StreamDirection,
    },
    ir::{
        generics::{interface::InterfaceGenericKind, GenericKind},
        implementation::structure::streamlet_instance::GenericParameterAssignment,
        traits::{GetSelf, InternSelf, MoveDb},
        Ir,
    },
};

use super::{genericproperty::GenericProperty, IsNull, LogicalType, SplitStreams};

/// Throughput is a struct containing a `PositiveReal`, which implements `Hash` for use in the `salsa` database.
#[derive(Debug, Clone, PartialEq)]
pub struct Throughput(PositiveReal);

impl Throughput {
    pub fn new(throughput: impl Into<PositiveReal>) -> Self {
        Throughput(throughput.into())
    }

    pub fn try_new(throughput: impl TryResult<PositiveReal>) -> Result<Self> {
        Ok(Throughput(throughput.try_result()?))
    }

    pub fn get(&self) -> f64 {
        self.0.get()
    }

    pub fn positive_real(&self) -> PositiveReal {
        self.0
    }

    pub fn non_negative(&self) -> NonNegative {
        self.0.get().ceil() as NonNegative
    }

    pub fn positive(&self) -> Positive {
        Positive::new(self.non_negative()).unwrap()
    }
}

impl From<PositiveReal> for Throughput {
    fn from(val: PositiveReal) -> Self {
        Throughput::new(val)
    }
}

impl TryFrom<f64> for Throughput {
    type Error = Error;

    fn try_from(value: f64) -> Result<Self> {
        Throughput::try_new(value)
    }
}

impl TryFrom<&str> for Throughput {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        Throughput::try_from(value.to_string())
    }
}

impl TryFrom<String> for Throughput {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        match value.parse::<f64>() {
            Ok(val) => Ok(Throughput(PositiveReal::new(val)?)),
            _ => Err(Error::InvalidArgument(format!(
                "{} is not a floating point number",
                value
            ))),
        }
    }
}

impl Eq for Throughput {}

impl Hash for Throughput {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.get().to_ne_bytes().hash(state);
    }
}

impl Mul for Throughput {
    type Output = Throughput;

    fn mul(self, rhs: Self) -> Self::Output {
        Throughput(self.0 * rhs.0)
    }
}

impl Default for Throughput {
    fn default() -> Self {
        Throughput(PositiveReal::new(1.0).unwrap())
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
    dimensionality: GenericProperty<NonNegative>,
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
    direction: StreamDirection,
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
    pub fn try_assign(
        &mut self,
        param_name: &Name,
        param_assignment: GenericParameterAssignment,
    ) -> Result<()> {
        match param_assignment.kind() {
            GenericKind::Behavioral(_) => Ok(()),
            GenericKind::Interface(i) => match i {
                InterfaceGenericKind::Dimensionality(_) => Ok(self.dimensionality = self
                    .dimensionality
                    .try_assign(param_name, param_assignment.value_take())?),
            },
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        db: &dyn Ir,
        data: Id<LogicalType>,
        throughput: impl TryResult<Throughput>,
        dimensionality: impl TryResult<GenericProperty<NonNegative>>,
        synchronicity: Synchronicity,
        complexity: impl TryResult<Complexity>,
        direction: StreamDirection,
        user: Id<LogicalType>,
        keep: bool,
    ) -> Result<Id<Self>> {
        let user_typ = user.get(db);
        if user_typ.is_element_only(db) {
            Ok(db.intern_stream(Stream {
                data: data,
                throughput: throughput.try_result()?,
                dimensionality: dimensionality.try_result()?.try_reduce(),
                synchronicity,
                complexity: complexity.try_result()?,
                direction,
                user,
                keep,
            }))
        } else {
            Err(Error::InvalidArgument(format!("User field must only contain element-manipulating types, and cannot contain Stream types. Is: {}", user_typ)))
        }
    }

    /// For internal use only. Does not validate the User field
    pub(crate) fn new(
        data: Id<LogicalType>,
        throughput: Throughput,
        dimensionality: impl Into<GenericProperty<NonNegative>>,
        synchronicity: Synchronicity,
        complexity: impl Into<Complexity>,
        direction: StreamDirection,
        user: Id<LogicalType>,
        keep: bool,
    ) -> Self {
        Stream {
            data,
            throughput: throughput,
            dimensionality: dimensionality.into().try_reduce(),
            synchronicity,
            complexity: complexity.into(),
            direction,
            user,
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
    pub fn direction(&self) -> StreamDirection {
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
    pub fn dimensionality(&self) -> &GenericProperty<NonNegative> {
        &self.dimensionality
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
            self.dimensionality().clone(),
            self.complexity(),
            self.user(db).fields(db),
            self.direction(),
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
    pub(crate) fn set_dimensionality(&mut self, dimensionality: GenericProperty<NonNegative>) {
        self.dimensionality = dimensionality.try_reduce();
    }
}

impl SynthesizeLogicalStream<BitCount, PhysicalStream> for Id<Stream> {
    fn synthesize(&self, db: &dyn Ir) -> Result<TypedStream<BitCount, PhysicalStream>> {
        let split = &self.split_streams(db)?;
        // NOTE: Signals will currently always be empty, as it refers to user-defined signals.
        let (signals, rest) = (split.signals().get(db).fields(db), split.streams());
        let mut streams = InsertionOrderedMap::new();
        for (path_name, stream) in rest.into_iter() {
            streams.try_insert(path_name.clone(), stream.get(db).physical(db))?;
        }
        let logical_stream = LogicalStream::new(signals, streams);
        let hierarchy = TypeHierarchy::from_stream(db, *self)?;
        let type_reference = TypeReference::collect_reference_from_split_streams(
            db,
            split,
            &hierarchy,
            &PathName::new_empty(),
        )?;
        Ok(TypedStream::new(logical_stream, type_reference))
    }
}

impl SplitsStreams for Id<Stream> {
    fn split_streams(&self, db: &dyn Ir) -> Result<SplitStreams> {
        db.stream_split_streams(*self)
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
            StreamDirection::Forward => self.direction = StreamDirection::Reverse,
            StreamDirection::Reverse => self.direction = StreamDirection::Forward,
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

impl fmt::Display for Synchronicity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Synchronicity::Sync => write!(f, "Sync"),
            Synchronicity::Flatten => write!(f, "Flatten"),
            Synchronicity::Desync => write!(f, "Desync"),
            Synchronicity::FlatDesync => write!(f, "FlatDesync"),
        }
    }
}

impl MoveDb<Id<Stream>> for Stream {
    fn move_db(
        &self,
        original_db: &dyn Ir,
        target_db: &dyn Ir,
        prefix: &Option<Name>,
    ) -> Result<Id<Stream>> {
        Ok(Stream::new(
            self.data_id().move_db(original_db, target_db, prefix)?,
            self.throughput(),
            self.dimensionality().clone(),
            self.synchronicity(),
            self.complexity(),
            self.direction(),
            self.user.move_db(original_db, target_db, prefix)?,
            self.keep(),
        )
        .intern(target_db))
    }
}

#[cfg(test)]
mod tests {
    use crate::ir::{db::Database, traits::TryIntern};

    use super::*;

    #[test]
    fn user_must_be_element() -> Result<()> {
        let _db = Database::default();
        let db = &_db;
        let stream1 = Stream::try_new(
            db,
            LogicalType::null_id(db),
            1.0,
            1,
            Synchronicity::Sync,
            4,
            StreamDirection::Forward,
            LogicalType::null_id(db),
            false,
        )?
        .try_intern(db)?;
        let stream_result = Stream::try_new(
            db,
            LogicalType::null_id(db),
            1.0,
            1,
            Synchronicity::Sync,
            4,
            StreamDirection::Forward,
            stream1,
            false,
        );
        assert!(match stream_result {
            Err(Error::InvalidArgument(_)) => true,
            _ => false,
        });

        Ok(())
    }
}
