use til_query::common::physical::stream::PhysicalBitCount;
use til_query::common::physical::{complexity::Complexity, signal_list::SignalList};
use til_query::common::stream_direction::StreamDirection;
use til_query::ir::physical_properties::InterfaceDirection;
use til_query::ir::Ir;
use tydi_common::error::TryOptional;
use tydi_common::numbers::Positive;
use tydi_common::{
    cat,
    error::Result,
    numbers::NonNegative,
    traits::{Reverse, Reversed},
};
use tydi_vhdl::architecture::arch_storage::Arch;
use tydi_vhdl::{
    common::vhdl_name::VhdlName,
    port::{Mode, Port},
};

use crate::IntoVhdl;

pub(crate) type PhysicalStream = til_query::common::physical::stream::PhysicalStream;

impl IntoVhdl<VhdlPhysicalStream> for PhysicalStream {
    fn canonical(
        &self,
        _ir_db: &dyn Ir,
        _arch_db: &mut dyn Arch,
        prefix: impl TryOptional<VhdlName>,
    ) -> Result<VhdlPhysicalStream> {
        // The VhdlPhysicalStream initially assumes it is part of an
        // "In" interface.

        let prefix = match prefix.try_optional()? {
            Some(n) => n.to_string(),
            None => "".to_string(),
        };
        let mode = match self.stream_direction() {
            StreamDirection::Forward => Mode::In,
            StreamDirection::Reverse => Mode::Out,
        };

        // TODO: NEED TO IMPLEMENT ARRAYS WITH RANGE BASED ON RELATIONS

        let signal_list: SignalList<PhysicalBitCount> = self.into();
        let mut signal_list = signal_list.map_named(|n, x| {
            if let Some(f) = x.try_eval() {
                // As the prefix is either a VhdlName or empty, and all signal names are valid
                Port::try_new(cat!(prefix, n), mode, f).unwrap()
            } else {
                // TODO: NEED TO IMPLEMENT ARRAYS WITH RANGE BASED ON RELATIONS
                todo!()
            }
        });

        signal_list.set_ready(signal_list.ready().as_ref().map(|ready| ready.reversed()))?;

        let user_bit_count = if let Some(u) = self.user_bit_count() {
            if let Some(f) = u.try_eval() {
                f.get()
            } else {
                // TODO: NEED TO IMPLEMENT ARRAYS WITH RANGE BASED ON RELATIONS
                todo!()
            }
        } else {
            0
        };

        let data_element_bit_count = if let Some(d) = self.data_element_bit_count() {
            if let Some(f) = d.try_eval() {
                f.get()
            } else {
                // TODO: NEED TO IMPLEMENT ARRAYS WITH RANGE BASED ON RELATIONS
                todo!()
            }
        } else {
            0
        };

        // TODO: ALLOW FOR PARAMETERIZED DIMENSIONALITY
        let dimensionality = if let Some(f) = self.dimensionality().try_eval() {
            f
        } else {
            todo!()
        };

        Ok(VhdlPhysicalStream::new(
            signal_list,
            self.element_lanes().clone(),
            dimensionality,
            self.complexity().clone(),
            data_element_bit_count,
            user_bit_count,
            InterfaceDirection::In,
            self.stream_direction(),
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VhdlPhysicalStream {
    signal_list: SignalList<Port>,
    /// Number of element lanes.
    element_lanes: Positive,
    /// Dimensionality.
    dimensionality: NonNegative,
    /// Complexity.
    complexity: Complexity,
    /// The absolute size of a data element
    data_element_size: NonNegative,
    /// The absolute size of the user data
    user_size: NonNegative,
    /// Direction of the parent interface.
    interface_direction: InterfaceDirection,
    /// The (logical) Stream's direction.
    ///
    /// This property is not affected by the `reverse` function, as it is
    /// a quality of the type.
    stream_direction: StreamDirection,
}

impl VhdlPhysicalStream {
    pub fn new(
        signal_list: SignalList<Port>,
        element_lanes: Positive,
        dimensionality: NonNegative,
        complexity: Complexity,
        data_element_size: NonNegative,
        user_size: NonNegative,
        interface_direction: InterfaceDirection,
        stream_direction: StreamDirection,
    ) -> Self {
        VhdlPhysicalStream {
            signal_list,
            element_lanes,
            dimensionality,
            complexity,
            data_element_size,
            user_size,
            interface_direction,
            stream_direction,
        }
    }

    pub fn signal_list(&self) -> &SignalList<Port> {
        &self.signal_list
    }

    pub fn element_lanes(&self) -> &Positive {
        &self.element_lanes
    }

    pub fn dimensionality(&self) -> NonNegative {
        self.dimensionality
    }

    pub fn complexity(&self) -> &Complexity {
        &self.complexity
    }

    pub fn interface_direction(&self) -> InterfaceDirection {
        self.interface_direction
    }

    pub fn with_interface_direction(mut self, direction: InterfaceDirection) -> Self {
        if direction != self.interface_direction {
            self.signal_list.apply(|x| x.reverse());
            self.interface_direction = direction;
        }
        self
    }

    pub fn mut_with_interface_direction(&mut self, direction: InterfaceDirection) -> &mut Self {
        if direction != self.interface_direction {
            self.signal_list.apply(|x| x.reverse());
            self.interface_direction = direction;
        }
        self
    }

    /// The (logical) Stream's direction.
    ///
    /// This property is not affected by the `reverse` function, as it is
    /// a quality of the type.
    pub fn stream_direction(&self) -> StreamDirection {
        self.stream_direction
    }

    /// The absolute size of the user data
    pub fn user_size(&self) -> NonNegative {
        self.user_size
    }

    /// The absolute size of a data element
    pub fn data_element_size(&self) -> NonNegative {
        self.data_element_size
    }
}

impl Reverse for VhdlPhysicalStream {
    fn reverse(&mut self) {
        match &self.interface_direction {
            InterfaceDirection::Out => self.mut_with_interface_direction(InterfaceDirection::In),
            InterfaceDirection::In => self.mut_with_interface_direction(InterfaceDirection::Out),
        };
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use til_query::ir::db::Database;
    use tydi_common::{
        map::InsertionOrderedMap,
        name::{Name, PathName},
        numbers::{BitCount, Positive},
    };
    use tydi_vhdl::declaration::Declare;
    use tydi_vhdl::object::object_type::ObjectType;

    use super::*;

    #[test]
    fn test_into_vhdl() -> Result<()> {
        let ir_db = &Database::default();
        let mut _arch_db = tydi_vhdl::architecture::arch_storage::db::Database::default();
        let arch_db = &mut _arch_db;
        let physical_stream = PhysicalStream::new(
            InsertionOrderedMap::try_new(vec![
                ("a".try_into()?, BitCount::new(3).unwrap()),
                ("b".try_into()?, BitCount::new(2).unwrap()),
            ])?,
            Positive::new(2).unwrap(),
            3,
            8,
            InsertionOrderedMap::new(),
            StreamDirection::Forward,
        );
        let mut signal_list = physical_stream.canonical(
            ir_db,
            arch_db,
            cat!(
                "a",
                PathName::new(vec![Name::try_new("test")?, Name::try_new("sub")?].into_iter())
                    .to_string()
            )
            .as_str(),
        )?;
        let ports = signal_list
            .mut_with_interface_direction(InterfaceDirection::Out)
            .signal_list();
        let result = ports
            .into_iter()
            .map(|x| x.declare(arch_db))
            .collect::<Result<Vec<String>>>()?
            .join("\n");
        assert_eq!(
            r#"a_test__sub_valid : out std_logic
a_test__sub_ready : in std_logic
a_test__sub_data : out std_logic_vector(9 downto 0)
a_test__sub_last : out std_logic_vector(5 downto 0)
a_test__sub_stai : out std_logic
a_test__sub_endi : out std_logic
a_test__sub_strb : out std_logic_vector(1 downto 0)"#,
            result,
            "output with pathname"
        );
        let mut signal_list = physical_stream.canonical(ir_db, arch_db, "a")?;
        let ports = signal_list
            .mut_with_interface_direction(InterfaceDirection::Out)
            .signal_list();
        let result = ports
            .into_iter()
            .map(|x| x.declare(arch_db))
            .collect::<Result<Vec<String>>>()?
            .join("\n");
        assert_eq!(
            r#"a_valid : out std_logic
a_ready : in std_logic
a_data : out std_logic_vector(9 downto 0)
a_last : out std_logic_vector(5 downto 0)
a_stai : out std_logic
a_endi : out std_logic
a_strb : out std_logic_vector(1 downto 0)"#,
            result,
            "output without pathname"
        );
        Ok(())
    }

    #[test]
    fn reverse_all_signal_list() -> Result<()> {
        let mut port_list = VhdlPhysicalStream::new(
            SignalList::try_new(
                Some(Port::try_new("valid", Mode::Out, ObjectType::Bit)?),
                Some(Port::try_new("ready", Mode::Out, ObjectType::Bit)?),
                Some(Port::try_new("data", Mode::Out, ObjectType::Bit)?),
                None,
                None,
                None,
                None,
                None,
            )?,
            Positive::new(1).unwrap(),
            0,
            Complexity::new_major(1),
            1,
            0,
            InterfaceDirection::Out,
            StreamDirection::Forward,
        );

        assert_eq!(3, port_list.signal_list().into_iter().len(), "3 signals");
        assert_eq!(
            InterfaceDirection::Out,
            port_list.interface_direction,
            "signal list has direction Out"
        );
        for port in port_list.signal_list().into_iter() {
            assert_eq!(Mode::Out, port.mode(), "Each Port has Mode::Out");
        }

        port_list.reverse();
        assert_eq!(3, port_list.signal_list().into_iter().len(), "3 signals");
        assert_eq!(
            InterfaceDirection::In,
            port_list.interface_direction,
            "signal list has direction In"
        );
        for port in port_list.signal_list().into_iter() {
            assert_eq!(
                Mode::In,
                port.mode(),
                "Each Port has Mode::In, after reverse"
            );
        }

        let mut signal_list_rev = port_list.mut_with_interface_direction(InterfaceDirection::In);
        assert_eq!(
            3,
            signal_list_rev.signal_list().into_iter().len(),
            "3 signals"
        );
        assert_eq!(
            InterfaceDirection::In,
            signal_list_rev.interface_direction,
            "signal list still has direction In"
        );
        for port in signal_list_rev.signal_list().into_iter() {
            assert_eq!(Mode::In, port.mode(), "Each Port still has Mode::In");
        }

        signal_list_rev = signal_list_rev.mut_with_interface_direction(InterfaceDirection::Out);
        assert_eq!(
            3,
            signal_list_rev.signal_list().into_iter().len(),
            "3 signals"
        );
        assert_eq!(
            InterfaceDirection::Out,
            signal_list_rev.interface_direction,
            "signal list now has direction Out"
        );
        for port in signal_list_rev.signal_list().into_iter() {
            assert_eq!(Mode::Out, port.mode(), "Each Port now has Mode::Out");
        }

        Ok(())
    }

    #[test]
    fn true_reverse_signal_list() -> Result<()> {
        // Verify whether ports are reversed properly, not just assigned a single mode.
        let mut port_list = VhdlPhysicalStream::new(
            SignalList::try_new(
                Some(Port::try_new("valid", Mode::In, ObjectType::Bit)?),
                Some(Port::try_new("ready", Mode::Out, ObjectType::Bit)?),
                Some(Port::try_new("data", Mode::In, ObjectType::Bit)?),
                None,
                None,
                None,
                None,
                None,
            )?,
            Positive::new(1).unwrap(),
            0,
            Complexity::new_major(1),
            1,
            0,
            InterfaceDirection::In,
            StreamDirection::Forward,
        );

        assert_eq!(3, port_list.signal_list().into_iter().len(), "3 signals");
        assert_eq!(
            InterfaceDirection::In,
            port_list.interface_direction,
            "signal list has direction In"
        );
        assert_eq!(
            Mode::In,
            port_list.signal_list().valid().as_ref().unwrap().mode()
        );
        assert_eq!(
            Mode::Out,
            port_list.signal_list().ready().as_ref().unwrap().mode()
        );
        assert_eq!(
            Mode::In,
            port_list.signal_list().data().as_ref().unwrap().mode()
        );

        port_list.reverse();
        assert_eq!(3, port_list.signal_list().into_iter().len(), "3 signals");
        assert_eq!(
            InterfaceDirection::Out,
            port_list.interface_direction,
            "signal list has direction Out"
        );
        assert_eq!(
            Mode::Out,
            port_list.signal_list().valid().as_ref().unwrap().mode()
        );
        assert_eq!(
            Mode::In,
            port_list.signal_list().ready().as_ref().unwrap().mode()
        );
        assert_eq!(
            Mode::Out,
            port_list.signal_list().data().as_ref().unwrap().mode()
        );

        Ok(())
    }
}
