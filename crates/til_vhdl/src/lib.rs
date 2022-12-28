extern crate tydi_vhdl;

use std::sync::Arc;

use log::debug;
use til_query::ir::Ir;
use tydi_common::{
    error::{Error, Result, TryOptional},
    traits::Identify,
};
use tydi_vhdl::{
    architecture::arch_storage::Arch,
    common::vhdl_name::{VhdlName, VhdlNameSelf},
    declaration::Declare,
    package::Package,
};

use crate::ir::streamlet::StreamletArchitecture;

pub mod common;
pub mod ir;

// TODO: To improve performance, it might make sense to put these
// implementations on a database trait, instead.
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
pub fn canonical(db: &dyn Ir) -> Result<()> {
    let mut dir = db
        .project_ref()
        .output_path()
        .clone()
        .ok_or(Error::ProjectError(
            "VHDL project requires an output path, project output path is None".to_string(),
        ))?;
    dir.push(db.project_ref().identifier());
    std::fs::create_dir_all(dir.as_path())?;

    let streamlets = db.all_streamlets();

    let mut package = Package::new_named(db.project_ref().identifier())?;
    let mut streamlet_component_names = vec![];

    let mut arch_db = tydi_vhdl::architecture::arch_storage::db::Database::default();
    for streamlet in streamlets.iter() {
        let mut streamlet = streamlet.canonical(db, &mut arch_db, "")?;
        let component = streamlet.to_component();
        streamlet_component_names.push((streamlet, component.vhdl_name().clone()));
        package.add_component(component);
    }

    let package = Arc::new(package);
    let mut pkg = dir.clone();
    pkg.push(format!("{}_pkg", package.vhdl_name()));
    pkg.set_extension("vhd");

    arch_db.set_default_package(package);

    std::fs::write(pkg.as_path(), arch_db.default_package().declare(&arch_db)?)?;
    debug!("Wrote {}.", pkg.as_path().to_str().unwrap_or(""));

    for (streamlet, component_name) in streamlet_component_names.into_iter() {
        arch_db.set_subject_component_name(Arc::new(component_name));
        let streamlet_arch = streamlet.to_architecture(db, &mut arch_db)?;
        let arch_string = match streamlet_arch {
            StreamletArchitecture::Imported(i) => i,
            StreamletArchitecture::Generated(g) => g.declare(&arch_db)?,
        };

        let mut arch = dir.clone();
        arch.push(streamlet.identifier());
        arch.set_extension("vhd");
        std::fs::write(arch.as_path(), arch_string)?;
        debug!("Wrote {}.", arch.as_path().to_str().unwrap_or(""));
    }

    Ok(())
}

// TODO: Once there's a super/project/root node, create a public function which uses all the IntoVhdls to output VHDL
// Also make IntoVhdl pub(crate) rather than pub, and only target the public function with the intergration tests in the "tests" folder.
// Don't want to expose the pub(crate) type aliases.
