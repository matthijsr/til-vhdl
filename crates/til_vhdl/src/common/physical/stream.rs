use std::convert::TryInto;

use tydi_common::{
    cat,
    error::{Error, Result},
    name::{Name, PathName},
    numbers::{BitCount, NonNegative, Positive},
    util::log2_ceil, traits::Reversed,
};
use tydi_vhdl::{
    common::vhdl_name::VhdlName,
    object::ObjectType,
    port::{Mode, Port},
};

use crate::ir::{physical_properties::InterfaceDirection, IntoVhdl};

use super::{complexity::Complexity, fields::Fields};

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
    element_fields: Fields,
    /// Number of element lanes.
    element_lanes: Positive,
    /// Dimensionality.
    dimensionality: NonNegative,
    /// Complexity.
    complexity: Complexity,
    /// User-defined transfer content.
    user: Fields,
}

impl PhysicalStream {
    pub fn try_new<T, U>(
        element_fields: T,
        element_lanes: usize,
        dimensionality: usize,
        complexity: impl Into<Complexity>,
        user: T,
    ) -> Result<Self>
    where
        T: IntoIterator<Item = (U, usize)>,
        U: TryInto<PathName, Error = Error>,
    {
        let element_fields = Fields::new(
            element_fields
                .into_iter()
                .map(|(path_name, bit_count)| {
                    (
                        path_name.try_into(),
                        Positive::new(bit_count as NonNegative),
                    )
                })
                .map(|(path_name, bit_count)| match (path_name, bit_count) {
                    (Ok(path_name), Some(bit_count)) => Ok((path_name, bit_count)),
                    (Err(e), _) => Err(e),
                    (_, None) => Err(Error::InvalidArgument(
                        "element lanes cannot be zero".to_string(),
                    )),
                })
                .collect::<Result<Vec<_>>>()?,
        )?;
        let element_lanes = Positive::new(element_lanes as NonNegative)
            .ok_or_else(|| Error::InvalidArgument("element lanes cannot be zero".to_string()))?;
        let dimensionality = dimensionality as NonNegative;
        let complexity = complexity.into();
        let user = Fields::new(
            user.into_iter()
                .map(|(path_name, bit_count)| {
                    (
                        path_name.try_into(),
                        Positive::new(bit_count as NonNegative),
                    )
                })
                .map(|(path_name, bit_count)| match (path_name, bit_count) {
                    (Ok(path_name), Some(bit_count)) => Ok((path_name, bit_count)),
                    (Err(e), _) => Err(e),
                    (_, None) => Err(Error::InvalidArgument(
                        "element lanes cannot be zero".to_string(),
                    )),
                })
                .collect::<Result<Vec<_>>>()?,
        )?;
        Ok(PhysicalStream::new(
            element_fields,
            element_lanes,
            dimensionality,
            complexity,
            user,
        ))
    }
    /// Constructs a new PhysicalStream using provided arguments. Returns an
    /// error when provided argument are not valid.
    pub fn new(
        element_fields: impl Into<Fields>,
        element_lanes: Positive,
        dimensionality: NonNegative,
        complexity: impl Into<Complexity>,
        user: impl Into<Fields>,
    ) -> Self {
        PhysicalStream {
            element_fields: element_fields.into(),
            element_lanes,
            dimensionality,
            complexity: complexity.into(),
            user: user.into(),
        }
    }

    /// Returns the element fields in this physical stream.
    pub fn element_fields(&self) -> &Fields {
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
    pub fn user(&self) -> &Fields {
        &self.user
    }

    /// Returns the bit count of the data (element) fields in this physical
    /// stream. The bit count is equal to the combined bit count of all fields
    /// multiplied by the number of lanes.
    pub fn data_bit_count(&self) -> NonNegative {
        self.element_fields
            .values()
            .map(|b| b.get())
            .sum::<NonNegative>()
            * self.element_lanes.get()
    }

    /// Returns the number of last bits in this physical stream. The number of
    /// last bits equals the dimensionality.
    pub fn last_bit_count(&self) -> NonNegative {
        self.dimensionality
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

    pub(crate) fn into_vhdl(
        &self,
        base_name: &str,
        path_name: &PathName,
        interface_origin: InterfaceDirection,
    ) -> Result<Vec<Port>> {
        let mut ports = vec![];

        let mode = match interface_origin {
            InterfaceDirection::Out => Mode::Out,
            InterfaceDirection::In => Mode::In,
        };

        ports.push(Port::new(
            VhdlName::try_new(cat!(base_name, path_name, "valid"))?,
            mode,
            ObjectType::Bit,
        ));
        ports.push(Port::new(
            VhdlName::try_new(cat!(base_name, path_name, "ready"))?,
            mode.reversed(),
            ObjectType::Bit,
        ));

        let mut opt = |x: NonNegative, name: &str| -> Result<()> {
            if let Some(bits) = BitCount::new(x) {
                ports.push(Port::new(
                    VhdlName::try_new(cat!(base_name, path_name, name))?,
                    mode,
                    bits.into(),
                ));
            }
            Ok(())
        };
        opt(self.data_bit_count(), "data")?;
        opt(self.last_bit_count(), "last")?;
        opt(self.stai_bit_count(), "stai")?;
        opt(self.endi_bit_count(), "endi")?;
        opt(self.strb_bit_count(), "strb")?;
        opt(self.user_bit_count(), "user")?;

        Ok(ports)
    }
}

#[cfg(test)]
mod tests {
    use tydi_common::numbers::BitCount;
    use tydi_vhdl::declaration::Declare;

    use super::*;

    #[test]
    fn test_into_vhdl() -> Result<()> {
        let _vhdl_db = tydi_vhdl::architecture::arch_storage::db::Database::default();
        let vhdl_db = &_vhdl_db;
        let physical_stream = PhysicalStream::new(
            Fields::new(vec![
                ("a".try_into()?, BitCount::new(3).unwrap()),
                ("b".try_into()?, BitCount::new(2).unwrap()),
            ])?,
            Positive::new(2).unwrap(),
            3,
            8,
            Fields::new(vec![])?,
        );
        let ports = physical_stream.into_vhdl(
            "a",
            &PathName::new(vec![Name::try_new("test")?, Name::try_new("sub")?].into_iter()),
            InterfaceDirection::Out,
        )?;
        let result = ports
            .into_iter()
            .map(|x| x.declare(vhdl_db))
            .collect::<Result<Vec<String>>>()?
            .join("\n");
        assert_eq!(
            r#"a_test__sub_valid : out std_logic
a_test__sub_ready : in std_logic
a_test__sub_data : out std_logic_vector(9 downto 0)
a_test__sub_last : out std_logic_vector(2 downto 0)
a_test__sub_stai : out std_logic
a_test__sub_endi : out std_logic
a_test__sub_strb : out std_logic_vector(1 downto 0)"#,
            result
        );
        let ports = physical_stream.into_vhdl("a", &PathName::new_empty(), InterfaceDirection::Out)?;
        let result = ports
            .into_iter()
            .map(|x| x.declare(vhdl_db))
            .collect::<Result<Vec<String>>>()?
            .join("\n");
        assert_eq!(
            r#"a_valid : out std_logic
a_ready : in std_logic
a_data : out std_logic_vector(9 downto 0)
a_last : out std_logic_vector(2 downto 0)
a_stai : out std_logic
a_endi : out std_logic
a_strb : out std_logic_vector(1 downto 0)"#,
            result
        );
        Ok(())
    }
}
