use std::sync::Arc;

use til_query::ir::{db::Database, streamlet::Streamlet};
use til_vhdl::IntoVhdl;
use tydi_common::error::Result;
use tydi_vhdl::{
    architecture::arch_storage::Arch, common::vhdl_name::VhdlNameSelf, package::Package,
};

pub fn ir_streamlet_to_vhdl(
    streamlet: Streamlet,
    db: &mut Database,
    arch_db: &mut tydi_vhdl::architecture::arch_storage::db::Database,
    mut package: Package,
) -> Result<til_vhdl::ir::streamlet::VhdlStreamlet> {
    let mut streamlet = streamlet.canonical(db, arch_db, None)?;
    let component = streamlet.to_component();
    arch_db.set_subject_component_name(Arc::new(component.vhdl_name().clone()));
    package.add_component(component);
    let package = Arc::new(package);
    arch_db.set_default_package(package);
    Ok(streamlet)
}
