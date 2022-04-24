use std::convert::TryInto;

use tydi_common::{
    error::{Error, Result},
    map::InsertionOrderedMap,
    name::PathName,
    numbers::{BitCount, NonNegative, Positive},
    util::log2_ceil,
};

use crate::common::stream_direction::StreamDirection;

use super::{complexity::Complexity, signal_list::SignalList};

/// Physical stream.
///
/// A physical stream carries a stream of elements, dimensionality information
/// for said elements, and (optionally) user-defined transfer information from
/// a source to a sink
///
/// [Reference](https://abs-tudelft.github.io/tydi/specification/physical.html#physical-stream-specification)
#[derive(Debug, Clone, PartialEq)]
pub struct PhysicalStream {
    /// Element content.
    element_fields: InsertionOrderedMap<PathName, BitCount>,
    /// Number of element lanes.
    element_lanes: Positive,
    /// Dimensionality.
    dimensionality: NonNegative,
    /// Complexity.
    complexity: Complexity,
    /// User-defined transfer content.
    user: InsertionOrderedMap<PathName, BitCount>,
    /// The Stream's direction.
    stream_direction: StreamDirection,
}

impl PhysicalStream {
    pub fn try_new<T, U>(
        element_fields: T,
        element_lanes: usize,
        dimensionality: usize,
        complexity: impl Into<Complexity>,
        user: T,
        stream_direction: StreamDirection,
    ) -> Result<Self>
    where
        T: IntoIterator<Item = (U, usize)>,
        U: TryInto<PathName, Error = Error>,
    {
        let mut element_fields_result = InsertionOrderedMap::new();
        for (path_name, bit_count) in element_fields.into_iter() {
            let path_name = path_name.try_into()?;
            match Positive::new(bit_count as NonNegative) {
                Some(bit_count) => element_fields_result.try_insert(path_name, bit_count)?,
                None => {
                    return Err(Error::InvalidArgument(
                        "element lanes cannot be zero".to_string(),
                    ));
                }
            }
        }
        let element_lanes = Positive::new(element_lanes as NonNegative)
            .ok_or_else(|| Error::InvalidArgument("element lanes cannot be zero".to_string()))?;
        let dimensionality = dimensionality as NonNegative;
        let complexity = complexity.into();
        let mut user_result = InsertionOrderedMap::new();
        for (path_name, bit_count) in user.into_iter() {
            let path_name = path_name.try_into()?;
            match Positive::new(bit_count as NonNegative) {
                Some(bit_count) => user_result.try_insert(path_name, bit_count)?,
                None => {
                    return Err(Error::InvalidArgument(
                        "element lanes cannot be zero".to_string(),
                    ));
                }
            }
        }
        Ok(PhysicalStream::new(
            element_fields_result,
            element_lanes,
            dimensionality,
            complexity,
            user_result,
            stream_direction,
        ))
    }
    /// Constructs a new PhysicalStream using provided arguments. Returns an
    /// error when provided argument are not valid.
    pub fn new(
        element_fields: impl Into<InsertionOrderedMap<PathName, BitCount>>,
        element_lanes: Positive,
        dimensionality: NonNegative,
        complexity: impl Into<Complexity>,
        user: impl Into<InsertionOrderedMap<PathName, BitCount>>,
        stream_direction: StreamDirection,
    ) -> Self {
        PhysicalStream {
            element_fields: element_fields.into(),
            element_lanes,
            dimensionality,
            complexity: complexity.into(),
            user: user.into(),
            stream_direction,
        }
    }

    /// Returns the element fields in this physical stream.
    pub fn element_fields(&self) -> &InsertionOrderedMap<PathName, BitCount> {
        &self.element_fields
    }

    /// Returns the number of element lanes in this physical stream.
    pub fn element_lanes(&self) -> Positive {
        self.element_lanes
    }

    /// Returns the dimensionality of this physical stream.
    pub fn dimensionality(&self) -> NonNegative {
        self.dimensionality
    }

    /// Returns the complexity of this physical stream.
    pub fn complexity(&self) -> &Complexity {
        &self.complexity
    }

    /// Returns the user fields in this physical stream.
    pub fn user(&self) -> &InsertionOrderedMap<PathName, BitCount> {
        &self.user
    }

    /// Returns the bit count of a single data element in this physical
    /// stream. The bit count is equal to the combined bit count of all fields.
    pub fn data_element_bit_count(&self) -> NonNegative {
        self.element_fields
            .values()
            .map(|b| b.get())
            .sum::<NonNegative>()
    }

    /// Returns the bit count of the data (element) fields in this physical
    /// stream. The bit count is equal to the combined bit count of all fields
    /// multiplied by the number of lanes.
    pub fn data_bit_count(&self) -> NonNegative {
        self.data_element_bit_count() * self.element_lanes.get()
    }

    /// Returns the number of last bits in this physical stream. The number of
    /// last bits equals the dimensionality.
    pub fn last_bit_count(&self) -> NonNegative {
        if self.complexity().major() >= 8 {
            self.dimensionality * self.element_lanes().get()
        } else {
            self.dimensionality
        }
    }

    /// Returns the number of `stai` (start index) bits in this physical
    /// stream.
    pub fn stai_bit_count(&self) -> NonNegative {
        if self.complexity.major() >= 6 && self.element_lanes.get() > 1 {
            log2_ceil(self.element_lanes)
        } else {
            0
        }
    }

    /// Returns the number of `endi` (end index) bits in this physical stream.
    pub fn endi_bit_count(&self) -> NonNegative {
        if (self.complexity.major() >= 5 || self.dimensionality >= 1)
            && self.element_lanes.get() > 1
        {
            log2_ceil(self.element_lanes)
        } else {
            0
        }
    }

    /// Returns the number of `strb` (strobe) bits in this physical stream.
    pub fn strb_bit_count(&self) -> NonNegative {
        if self.complexity.major() >= 7 || self.dimensionality >= 1 {
            self.element_lanes.get()
        } else {
            0
        }
    }

    /// Returns the bit count of the user fields in this physical stream.
    pub fn user_bit_count(&self) -> NonNegative {
        self.user.values().map(|b| b.get()).sum::<NonNegative>()
    }

    /// The Stream's direction.
    pub fn stream_direction(&self) -> StreamDirection {
        self.stream_direction
    }
}

impl From<&PhysicalStream> for SignalList<Positive> {
    fn from(phys: &PhysicalStream) -> Self {
        SignalList::try_new(
            Positive::new(1),
            Positive::new(1),
            Positive::new(phys.data_bit_count()),
            Positive::new(phys.last_bit_count()),
            Positive::new(phys.stai_bit_count()),
            Positive::new(phys.endi_bit_count()),
            Positive::new(phys.strb_bit_count()),
            Positive::new(phys.user_bit_count()),
        )
        .unwrap()
    }
}

impl From<PhysicalStream> for SignalList<Positive> {
    fn from(phys: PhysicalStream) -> Self {
        (&phys).into()
    }
}
