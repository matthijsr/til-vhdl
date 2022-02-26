use til_query::{
    common::logical::logical_stream::SynthesizeLogicalStream,
    ir::{physical_properties::InterfaceDirection, Ir},
};
use tydi_common::{
    cat,
    error::{Result, TryOptional},
    traits::{Document, Identify},
};

use tydi_vhdl::{
    architecture::arch_storage::Arch,
    common::vhdl_name::VhdlName,
    port::{Mode, Port},
};

use crate::IntoVhdl;

pub(crate) type Interface = til_query::ir::interface::Interface;

impl IntoVhdl<Vec<Port>> for Interface {
    fn canonical(
        &self,
        ir_db: &dyn Ir,
        arch_db: &mut dyn Arch,
        prefix: impl TryOptional<VhdlName>,
    ) -> Result<Vec<Port>> {
        let n: VhdlName = match prefix.try_optional()? {
            Some(some) => VhdlName::try_new(cat!(some, self.identifier()))?,
            None => self.name().clone().into(),
        };
        let mut ports = Vec::new();

        let synth = self.stream_id().synthesize(ir_db);

        for (path, width) in synth.signals() {
            let signal_path = format!("{}__{}", &n, path);
            ports.push(Port::new(
                VhdlName::try_new(signal_path)?,
                match self.physical_properties().direction() {
                    InterfaceDirection::Out => Mode::Out,
                    InterfaceDirection::In => Mode::In,
                },
                width.clone().into(),
            ));
        }

        for (path, phys) in synth.streams() {
            let phys_name = if path.len() > 0 {
                format!("{}__{}", &n, path)
            } else {
                n.to_string()
            };
            for port in phys
                .canonical(ir_db, arch_db, phys_name.as_str())?
                .with_direction(self.physical_properties().direction())
                .signal_list()
            {
                ports.push(port.clone());
            }
        }

        if let Some(doc) = self.doc() {
            if ports.len() > 0 {
                ports[0].set_doc(doc);
            }
        }

        Ok(ports)
    }
}
