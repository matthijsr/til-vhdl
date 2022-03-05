use indexmap::IndexMap;
use tydi_common::error::Result;

use crate::{
    common::vhdl_name::VhdlName,
    component::Component,
    object::{record::RecordObject, ObjectType},
    port::{Mode, Port},
};

pub(crate) fn empty_component() -> Component {
    Component::try_new("empty_component", vec![], vec![], None).unwrap()
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
