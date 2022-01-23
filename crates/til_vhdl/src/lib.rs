extern crate tydi_vhdl;

use til_query::ir::Ir;
use tydi_common::error::{Result, TryOptional};
use tydi_vhdl::{architecture::arch_storage::Arch, common::vhdl_name::VhdlName};

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

// TODO: Once there's a super/project/root node, create a public function which uses all the IntoVhdls to output VHDL
// Also make IntoVhdl pub(crate) rather than pub, and only target the public function with the intergration tests in the "tests" folder.
// Don't want to expose the pub(crate) type aliases.
