use indexmap::IndexMap;
use tydi_common::error::{Error, Result};
use tydi_common::name::Name;

use crate::common::vhdl_name::VhdlName;
use crate::{
    component::Component,
    object::{record::RecordObject, ObjectType},
    port::{Mode, Port},
};

pub(crate) fn empty_component() -> Component {
    Component::new(
        VhdlName::try_new("empty_component").unwrap(),
        vec![],
        vec![],
        None,
    )
}

pub(crate) fn record_with_nested_type() -> Result<ObjectType> {
    let nested = ObjectType::array(3, 0, ObjectType::Bit, Name::try_new("nested_type")?)?;
    let mut fields = IndexMap::new();
    fields.insert(Name::try_new("nested")?, nested);
    Ok(RecordObject::new(Name::try_new("record_type")?, fields).into())
}

pub(crate) fn component_with_nested_types() -> Result<Component> {
    let port = Port::new(
        VhdlName::try_new("some_other_port")?,
        Mode::Out,
        record_with_nested_type()?,
    );
    Ok(Component::new(
        VhdlName::try_new("component_with_nested_types")?,
        vec![],
        vec![port, Port::clk()],
        None,
    ))
}
