extern crate tydi_vhdl;

use std::{path::Path, sync::Arc};

use log::debug;
use til_query::ir::Ir;
use tydi_common::{
    error::{Result, TryOptional},
    traits::Identify,
};
use tydi_vhdl::{
    architecture::arch_storage::Arch,
    common::vhdl_name::VhdlName,
    component::Component,
    declaration::{Declare},
    package::Package,
};

pub mod common;
pub mod ir;

pub trait IntoVhdl<T> {
    fn canonical(
        &self,
        ir_db: &dyn Ir,
        arch_db: &mut dyn Arch,
        prefix: impl TryOptional<VhdlName>,
    ) -> Result<T>;
    fn fancy(&self, _ir_db: &dyn Ir, _arch_db: &dyn Arch) -> Result<T> {
        todo!()
    }
}

/// Generates canonical definitions of all Streamlets defined in the database `db`.
///
/// The `output_folder` is defined relative to the base Project's folder.
///
/// The `indent_style` determines whether to use tabs or spaces, and how many.
pub fn canonical(db: &dyn Ir, output_folder: impl AsRef<Path>) -> Result<()> {
    let project = db.project();

    let mut dir = output_folder.as_ref().to_path_buf();
    dir.push(project.identifier());
    std::fs::create_dir_all(dir.as_path())?;

    let streamlets = db.all_streamlets();

    let mut package = Package::new_named(project.identifier())?;
    let mut streamlet_components = vec![];

    for streamlet in streamlets.iter() {
        let mut arch_db = tydi_vhdl::architecture::arch_storage::db::Database::default();
        let component: Arc<Component> = Arc::new(streamlet.canonical(db, &mut arch_db, "")?);
        package.add_component(component.clone());
        streamlet_components.push((streamlet, component));
    }

    let package = Arc::new(package);

    let mut pkg = dir.clone();
    pkg.push(format!("{}_pkg", package.identifier()));
    pkg.set_extension("vhd");

    // TODO: Make the DB optional on declare.
    let mut arch_db = tydi_vhdl::architecture::arch_storage::db::Database::default();

    std::fs::write(pkg.as_path(), package.declare(&arch_db)?)?;
    debug!("Wrote {}.", pkg.as_path().to_str().unwrap_or(""));

    for (streamlet, component) in streamlet_components.iter() {
        arch_db = tydi_vhdl::architecture::arch_storage::db::Database::default();
        arch_db.set_default_package(package.clone());
        arch_db.set_subject_component(component.clone());

        let streamlet_arch: String = streamlet.canonical(db, &mut arch_db, None)?;

        let mut arch = dir.clone();
        arch.push(streamlet.identifier());
        arch.set_extension("vhd");
        std::fs::write(arch.as_path(), streamlet_arch)?;
        debug!("Wrote {}.", arch.as_path().to_str().unwrap_or(""));
    }

    Ok(())
}

// TODO: Once there's a super/project/root node, create a public function which uses all the IntoVhdls to output VHDL
// Also make IntoVhdl pub(crate) rather than pub, and only target the public function with the intergration tests in the "tests" folder.
// Don't want to expose the pub(crate) type aliases.
