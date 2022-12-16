use std::convert::TryInto;

use tydi_common::{
    error::{Error, Result, TryOptional, TryResult},
    map::InsertionOrderedMap,
    name::{Name, PathName},
    numbers::{BitCount, NonNegative, Positive},
    util::log2_ceil,
};

use crate::common::{
    logical::logicaltype::stream::{Dimensionality, StreamProperty, StreamPropertyOperator},
    stream_direction::StreamDirection,
};

use super::{complexity::Complexity, signal_list::SignalList};

#[derive(Debug, Clone, PartialEq)]
pub enum PhysicalBitCountBase {
    Combination(Box<Self>, StreamPropertyOperator, Box<Self>),
    Fixed(Positive),
    Parameterized(Name),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhysicalBitCount {
    base: PhysicalBitCountBase,
    multipliers: Vec<NonNegative>,
}

impl PhysicalBitCount {
    pub fn parameterized(name: impl Into<Name>) -> Self {
        PhysicalBitCount {
            base: PhysicalBitCountBase::Parameterized(name.into()),
            multipliers: vec![],
        }
    }

    pub fn fixed(val: NonNegative) -> Option<Self> {
        if let Some(val) = Positive::new(val) {
            Some(Self {
                base: PhysicalBitCountBase::Fixed(val),
                multipliers: vec![],
            })
        } else {
            None
        }
    }

    pub fn with_multiplier(mut self, m: NonNegative) -> Self {
        self.multipliers.push(m);
        self
    }

    pub fn base(&self) -> &PhysicalBitCountBase {
        &self.base
    }

    /// Multipliers. If empty, there are no multipliers (i.e., multiplier is 1)
    pub fn multipliers(&self) -> &Vec<NonNegative> {
        &self.multipliers
    }
}

impl From<StreamProperty<NonNegative>> for Option<PhysicalBitCount> {
    fn from(d: StreamProperty<NonNegative>) -> Option<PhysicalBitCount> {
        match d {
            StreamProperty::Combination(l, op, r) => {
                let lv = Option::<PhysicalBitCount>::from(l.as_ref().clone());
                let rv = Option::<PhysicalBitCount>::from(l.as_ref().clone());
                match (lv, rv) {
                    (None, None) => None,
                    (None, Some(rv)) => Some(rv),
                    (Some(lv), None) => Some(lv),
                    (Some(lv), Some(rv)) => Some(PhysicalBitCount {
                        base: PhysicalBitCountBase::Combination(
                            Box::new(lv.base),
                            op,
                            Box::new(rv.base),
                        ),
                        multipliers: vec![],
                    }),
                }
            }
            StreamProperty::Fixed(f) => PhysicalBitCount::fixed(f),
            StreamProperty::Parameterized(n) => Some(PhysicalBitCount::parameterized(n)),
        }
    }
}

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
    dimensionality: Dimensionality,
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
        dimensionality: impl TryResult<Dimensionality>,
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
            dimensionality.try_result()?,
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
        dimensionality: impl Into<Dimensionality>,
        complexity: impl Into<Complexity>,
        user: impl Into<InsertionOrderedMap<PathName, BitCount>>,
        stream_direction: StreamDirection,
    ) -> Self {
        PhysicalStream {
            element_fields: element_fields.into(),
            element_lanes,
            dimensionality: dimensionality.into(),
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
    pub fn dimensionality(&self) -> &Dimensionality {
        &self.dimensionality
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
    pub fn data_element_bit_count(&self) -> Option<PhysicalBitCount> {
        PhysicalBitCount::fixed(
            self.element_fields
                .values()
                .map(|b| b.get())
                .sum::<NonNegative>(),
        )
    }

    /// Returns the bit count of the data (element) fields in this physical
    /// stream. The bit count is equal to the combined bit count of all fields
    /// multiplied by the number of lanes.
    pub fn data_bit_count(&self) -> Option<PhysicalBitCount> {
        if let Some(d) = self.data_element_bit_count() {
            Some(d.with_multiplier(self.element_lanes.get()))
        } else {
            None
        }
    }

    /// Returns the number of last bits in this physical stream. The number of
    /// last bits equals the dimensionality.
    pub fn last_bit_count(&self) -> Option<PhysicalBitCount> {
        let bitcount = Option::<PhysicalBitCount>::from(self.dimensionality().clone());
        if let Some(bitcount) = bitcount {
            Some(if self.complexity().major() >= 8 {
                bitcount.with_multiplier(self.element_lanes().get())
            } else {
                bitcount
            })
        } else {
            None
        }
    }

    /// Returns the number of `stai` (start index) bits in this physical
    /// stream.
    pub fn stai_bit_count(&self) -> Option<PhysicalBitCount> {
        PhysicalBitCount::fixed(
            if self.complexity.major() >= 6 && self.element_lanes.get() > 1 {
                log2_ceil(self.element_lanes)
            } else {
                0
            },
        )
    }

    fn has_dimensions(&self) -> bool {
        match self.dimensionality() {
            StreamProperty::Combination(_, _, _) => match self.dimensionality().try_eval() {
                Some(f) => f >= 1,
                None => true,
            },
            StreamProperty::Fixed(f) => *f >= 1,
            StreamProperty::Parameterized(_) => true,
        }
    }

    /// Returns the number of `endi` (end index) bits in this physical stream.
    pub fn endi_bit_count(&self) -> Option<PhysicalBitCount> {
        PhysicalBitCount::fixed(
            if (self.complexity.major() >= 5 || self.has_dimensions())
                && self.element_lanes.get() > 1
            {
                log2_ceil(self.element_lanes)
            } else {
                0
            },
        )
    }

    /// Returns the number of `strb` (strobe) bits in this physical stream.
    pub fn strb_bit_count(&self) -> Option<PhysicalBitCount> {
        PhysicalBitCount::fixed(if self.complexity.major() >= 7 || self.has_dimensions() {
            self.element_lanes.get()
        } else {
            0
        })
    }

    /// Returns the bit count of the user fields in this physical stream.
    pub fn user_bit_count(&self) -> Option<PhysicalBitCount> {
        PhysicalBitCount::fixed(self.user.values().map(|b| b.get()).sum::<NonNegative>())
    }

    /// The Stream's direction.
    pub fn stream_direction(&self) -> StreamDirection {
        self.stream_direction
    }
}

impl From<&PhysicalStream> for SignalList<PhysicalBitCount> {
    fn from(phys: &PhysicalStream) -> Self {
        SignalList::try_new(
            PhysicalBitCount::fixed(1),
            PhysicalBitCount::fixed(1),
            phys.data_bit_count(),
            phys.last_bit_count(),
            phys.stai_bit_count(),
            phys.endi_bit_count(),
            phys.strb_bit_count(),
            phys.user_bit_count(),
        )
        .unwrap()
    }
}

impl From<PhysicalStream> for SignalList<PhysicalBitCount> {
    fn from(phys: PhysicalStream) -> Self {
        (&phys).into()
    }
}
