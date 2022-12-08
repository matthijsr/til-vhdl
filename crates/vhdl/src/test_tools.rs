use indexmap::IndexMap;

use tydi_common::error::Result;

use crate::assignment::ValueAssignment;
use crate::object::object_type::{IntegerType, ObjectType};
use crate::port::GenericParameter;
use crate::{
    common::vhdl_name::VhdlName,
    component::Component,
    object::record::RecordObject,
    port::{Mode, Port},
};

/// Creates a component:
/// ```vhdl
/// component empty_component
///   port (
///
///   );
/// end component;
/// ```
pub(crate) fn empty_component() -> Component {
    Component::try_new("empty_component", vec![], vec![], None).unwrap()
}

/// Creates a component:
/// ```vhdl
/// component test
///   port (
///     -- This is port documentation
///     -- Next line.
///     some_port : in std_logic;
///     some_other_port : out std_logic_vector(43 downto 0);
///     clk : in std_logic
///   );
/// end component;
/// ```
pub(crate) fn simple_component() -> Result<Component> {
    let port1 = Port::try_new_documented(
        "some_port",
        Mode::In,
        ObjectType::Bit,
        "This is port documentation\nNext line.",
    )?;
    let port2 = Port::try_new("some_other_port", Mode::Out, 43..0)?;
    let clk = Port::clk();
    Component::try_new("test", vec![], vec![port1, port2, clk], None)
}

/// Creates a component:
/// ```vhdl
/// component test
///   generic (
///     -- This is parameter documentation
///     -- Next line.
///     some_param : positive := 42;
///     some_other_param : std_logic;
///     some_other_param : integer := -42
///   );
///   port (
///     -- This is port documentation
///     -- Next line.
///     some_port : in std_logic;
///     some_other_port : out std_logic_vector(43 downto 0);
///     clk : in std_logic
///   );
/// end component;
/// ```
pub(crate) fn simple_component_with_generics() -> Result<Component> {
    let port1 = Port::try_new_documented(
        "some_port",
        Mode::In,
        ObjectType::Bit,
        "This is port documentation\nNext line.",
    )?;
    let port2 = Port::try_new("some_other_port", Mode::Out, 43..0)?;
    let clk = Port::clk();

    let param1 = GenericParameter::try_new_documented(
        "some_param",
        Some(ValueAssignment::from(42).into()),
        ObjectType::Integer(IntegerType::Positive),
        "This is parameter documentation\nNext line.",
    )?;
    let param2 = GenericParameter::try_new("some_other_param", None, ObjectType::Bit)?;
    let param3 = GenericParameter::try_new(
        "some_other_param2",
        Some(ValueAssignment::from(-42).into()),
        ObjectType::Integer(IntegerType::Integer),
    )?;
    Component::try_new(
        "test",
        vec![param1, param2, param3],
        vec![port1, port2, clk],
        None,
    )
}

pub(crate) fn record_with_nested_type() -> Result<ObjectType> {
    let nested = ObjectType::array(3, 0, ObjectType::Bit, VhdlName::try_new("nested_type")?)?;
    let mut fields = IndexMap::new();
    fields.insert(VhdlName::try_new("nested")?, nested);
    Ok(RecordObject::new(VhdlName::try_new("record_type")?, fields).into())
}

pub(crate) fn component_with_nested_types() -> Result<Component> {
    let port = Port::try_new("some_other_port", Mode::Out, record_with_nested_type()?)?;
    Component::try_new(
        "component_with_nested_types",
        vec![],
        vec![port, Port::clk()],
        None,
    )
}
