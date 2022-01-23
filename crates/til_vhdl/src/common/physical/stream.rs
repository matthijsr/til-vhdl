use til_query::ir::physical_properties::InterfaceDirection;
use til_query::ir::Ir;
use tydi_common::error::TryOptional;
use tydi_common::{
    cat,
    error::Result,
    numbers::{BitCount, NonNegative},
    traits::{Reverse, Reversed},
};
use tydi_vhdl::architecture::arch_storage::Arch;
use tydi_vhdl::{
    common::vhdl_name::VhdlName,
    object::ObjectType,
    port::{Mode, Port},
};

use crate::IntoVhdl;

pub(crate) type PhysicalStream = til_query::common::physical::stream::PhysicalStream;

impl IntoVhdl<SignalList> for PhysicalStream {
    fn canonical(
        &self,
        _ir_db: &dyn Ir,
        _arch_db: &mut dyn Arch,
        prefix: impl TryOptional<VhdlName>,
    ) -> Result<SignalList> {
        let prefix = match prefix.try_optional()? {
            Some(n) => n.to_string(),
            None => "".to_string(),
        };
        let mode = Mode::In;

        let valid = Port::new(
            VhdlName::try_new(cat!(prefix, "valid"))?,
            mode,
            ObjectType::Bit,
        );
        let ready = Port::new(
            VhdlName::try_new(cat!(prefix, "ready"))?,
            mode.reversed(),
            ObjectType::Bit,
        );

        let opt = |x: NonNegative, name: &str| -> Result<Option<Port>> {
            if let Some(bits) = BitCount::new(x) {
                Ok(Some(Port::new(
                    VhdlName::try_new(cat!(prefix, name))?,
                    mode,
                    bits.into(),
                )))
            } else {
                Ok(None)
            }
        };

        Ok(SignalList::new(
            valid,
            ready,
            opt(self.data_bit_count(), "data")?,
            opt(self.last_bit_count(), "last")?,
            opt(self.stai_bit_count(), "stai")?,
            opt(self.endi_bit_count(), "endi")?,
            opt(self.strb_bit_count(), "strb")?,
            opt(self.user_bit_count(), "user")?,
            InterfaceDirection::In,
        ))
    }
}

pub struct SignalList {
    valid: Port,
    ready: Port,
    data: Option<Port>,
    last: Option<Port>,
    stai: Option<Port>,
    endi: Option<Port>,
    strb: Option<Port>,
    user: Option<Port>,
    direction: InterfaceDirection,
}

impl SignalList {
    pub fn new(
        valid: Port,
        ready: Port,
        data: Option<Port>,
        last: Option<Port>,
        stai: Option<Port>,
        endi: Option<Port>,
        strb: Option<Port>,
        user: Option<Port>,
        direction: InterfaceDirection,
    ) -> Self {
        SignalList {
            valid,
            ready,
            data,
            last,
            stai,
            endi,
            strb,
            user,
            direction,
        }
    }

    pub fn valid(&self) -> &Port {
        &self.valid
    }

    pub fn ready(&self) -> &Port {
        &self.ready
    }

    pub fn data(&self) -> Option<&Port> {
        self.data.as_ref()
    }

    pub fn last(&self) -> Option<&Port> {
        self.last.as_ref()
    }

    pub fn stai(&self) -> Option<&Port> {
        self.stai.as_ref()
    }

    pub fn endi(&self) -> Option<&Port> {
        self.endi.as_ref()
    }

    pub fn strb(&self) -> Option<&Port> {
        self.strb.as_ref()
    }

    pub fn user(&self) -> Option<&Port> {
        self.user.as_ref()
    }

    /// Returns references to all defined ports within the signal list
    pub fn ports<'a>(&'a self) -> Vec<&'a Port> {
        let mut result = vec![];
        result.push(self.valid());
        result.push(self.ready());
        if let Some(data) = self.data() {
            result.push(data);
        }
        if let Some(last) = self.last() {
            result.push(last);
        }
        if let Some(stai) = self.stai() {
            result.push(stai);
        }
        if let Some(endi) = self.endi() {
            result.push(endi);
        }
        if let Some(strb) = self.strb() {
            result.push(strb);
        }
        if let Some(user) = self.user() {
            result.push(user);
        }

        result
    }

    pub fn with_direction(&mut self, direction: InterfaceDirection) -> &mut Self {
        if direction != self.direction {
            self.valid.reverse();
            self.ready.reverse();
            if let Some(data) = &mut self.data {
                data.reverse();
            }
            if let Some(last) = &mut self.last {
                last.reverse();
            }
            if let Some(stai) = &mut self.stai {
                stai.reverse();
            }
            if let Some(endi) = &mut self.endi {
                endi.reverse();
            }
            if let Some(strb) = &mut self.strb {
                strb.reverse();
            }
            if let Some(user) = &mut self.user {
                user.reverse();
            }
            self.direction = direction;
        }
        self
    }
}

impl Reverse for SignalList {
    fn reverse(&mut self) {
        match &self.direction {
            InterfaceDirection::Out => self.with_direction(InterfaceDirection::In),
            InterfaceDirection::In => self.with_direction(InterfaceDirection::Out),
        };
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use til_query::{common::physical::fields::Fields, ir::db::Database};
    use tydi_common::{
        name::{Name, PathName},
        numbers::{BitCount, Positive},
    };
    use tydi_vhdl::declaration::Declare;

    use super::*;

    #[test]
    fn test_into_vhdl() -> Result<()> {
        let ir_db = &Database::default();
        let mut _arch_db = tydi_vhdl::architecture::arch_storage::db::Database::default();
        let arch_db = &mut _arch_db;
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
        let ports = signal_list.with_direction(InterfaceDirection::Out).ports();
        let result = ports
            .into_iter()
            .map(|x| x.declare(arch_db))
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
            result,
            "output with pathname"
        );
        let mut signal_list = physical_stream.canonical(ir_db, arch_db, "a")?;
        let ports = signal_list.with_direction(InterfaceDirection::Out).ports();
        let result = ports
            .into_iter()
            .map(|x| x.declare(arch_db))
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
            result,
            "output without pathname"
        );
        Ok(())
    }

    #[test]
    fn reverse_all_signal_list() -> Result<()> {
        let mut signal_list = SignalList::new(
            Port::new(VhdlName::try_new("valid")?, Mode::Out, ObjectType::Bit),
            Port::new(VhdlName::try_new("ready")?, Mode::Out, ObjectType::Bit),
            Some(Port::new(
                VhdlName::try_new("data")?,
                Mode::Out,
                ObjectType::Bit,
            )),
            None,
            None,
            None,
            None,
            None,
            InterfaceDirection::Out,
        );

        assert_eq!(3, signal_list.ports().len(), "3 signals");
        assert_eq!(
            InterfaceDirection::Out,
            signal_list.direction,
            "signal list has direction Out"
        );
        for port in signal_list.ports() {
            assert_eq!(Mode::Out, port.mode(), "Each Port has Mode::Out");
        }

        signal_list.reverse();
        assert_eq!(3, signal_list.ports().len(), "3 signals");
        assert_eq!(
            InterfaceDirection::In,
            signal_list.direction,
            "signal list has direction In"
        );
        for port in signal_list.ports() {
            assert_eq!(
                Mode::In,
                port.mode(),
                "Each Port has Mode::In, after reverse"
            );
        }

        let mut signal_list_rev = signal_list.with_direction(InterfaceDirection::In);
        assert_eq!(3, signal_list_rev.ports().len(), "3 signals");
        assert_eq!(
            InterfaceDirection::In,
            signal_list_rev.direction,
            "signal list still has direction In"
        );
        for port in signal_list_rev.ports() {
            assert_eq!(Mode::In, port.mode(), "Each Port still has Mode::In");
        }

        signal_list_rev = signal_list_rev.with_direction(InterfaceDirection::Out);
        assert_eq!(3, signal_list_rev.ports().len(), "3 signals");
        assert_eq!(
            InterfaceDirection::Out,
            signal_list_rev.direction,
            "signal list now has direction Out"
        );
        for port in signal_list_rev.ports() {
            assert_eq!(Mode::Out, port.mode(), "Each Port now has Mode::Out");
        }

        Ok(())
    }

    #[test]
    fn true_reverse_signal_list() -> Result<()> {
        // Verify whether ports are reversed properly, not just assigned a single mode.
        let mut signal_list = SignalList::new(
            Port::new(VhdlName::try_new("valid")?, Mode::In, ObjectType::Bit),
            Port::new(VhdlName::try_new("ready")?, Mode::Out, ObjectType::Bit),
            Some(Port::new(
                VhdlName::try_new("data")?,
                Mode::In,
                ObjectType::Bit,
            )),
            None,
            None,
            None,
            None,
            None,
            InterfaceDirection::In,
        );

        assert_eq!(3, signal_list.ports().len(), "3 signals");
        assert_eq!(
            InterfaceDirection::In,
            signal_list.direction,
            "signal list has direction In"
        );
        assert_eq!(Mode::In, signal_list.valid().mode());
        assert_eq!(Mode::Out, signal_list.ready().mode());
        assert_eq!(Mode::In, signal_list.data().unwrap().mode());

        signal_list.reverse();
        assert_eq!(3, signal_list.ports().len(), "3 signals");
        assert_eq!(
            InterfaceDirection::Out,
            signal_list.direction,
            "signal list has direction Out"
        );
        assert_eq!(Mode::Out, signal_list.valid().mode());
        assert_eq!(Mode::In, signal_list.ready().mode());
        assert_eq!(Mode::Out, signal_list.data().unwrap().mode());

        Ok(())
    }
}
