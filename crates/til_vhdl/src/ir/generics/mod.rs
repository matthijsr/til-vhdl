use til_query::ir::generics::behavioral::integer::IntegerGenericKind;
use til_query::ir::generics::behavioral::BehavioralGenericKind;
use til_query::ir::generics::interface::InterfaceGenericKind;
use til_query::ir::generics::GenericKind;
use tydi_common::map::InsertionOrderedMap;
use tydi_common::name::{Name, NameSelf};
use tydi_common::{error::Result, traits::Document};
use tydi_vhdl::object::object_type::{IntegerType, ObjectType};
use tydi_vhdl::{architecture::arch_storage::Arch, port::GenericParameter};

use self::param_value::param_value_to_vhdl;

pub mod param_value;

pub fn param_to_param(
    arch_db: &dyn Arch,
    val: &til_query::ir::generics::GenericParameter,
    parent_params: &InsertionOrderedMap<Name, GenericParameter>,
) -> Result<GenericParameter> {
    let default = param_value_to_vhdl(arch_db, val.default_value(), parent_params)?;
    let typ = match val.kind() {
        GenericKind::Behavioral(b) => match b {
            BehavioralGenericKind::Integer(i) => match i.kind() {
                IntegerGenericKind::Integer => IntegerType::Integer,
                IntegerGenericKind::Natural => IntegerType::Natural,
                IntegerGenericKind::Positive => IntegerType::Positive,
            },
        },
        GenericKind::Interface(i) => match i {
            InterfaceGenericKind::Dimensionality(_) => IntegerType::Positive,
        },
    };
    if let Some(doc) = val.doc() {
        GenericParameter::try_new_documented(val.name().clone(), Some(default), typ, doc)
    } else {
        GenericParameter::try_new(val.name().clone(), Some(default), typ)
    }
}
