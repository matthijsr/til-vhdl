extern crate tydi_vhdl;

use crate::ir::Ir;
use tydi_common::error;
use tydi_common::error::TryOptional;
use tydi_vhdl::architecture::arch_storage::Arch;
use tydi_vhdl::common::vhdl_name::VhdlName;

pub mod common;
pub mod ir;
pub mod test_utils;

pub trait IntoVhdl<T> {
    fn canonical(
        &self,
        ir_db: &dyn Ir,
        arch_db: &mut dyn Arch,
        prefix: impl TryOptional<VhdlName>,
    ) -> error::Result<T>;
    fn fancy(&self, _ir_db: &dyn Ir, _arch_db: &dyn Arch) -> error::Result<T> {
        todo!()
    }
}
