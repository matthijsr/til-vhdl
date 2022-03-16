use std::fs;

use til_query::{
    common::{logical::logical_stream::LogicalStream, physical::signal_list::SignalList},
    ir::{implementation::ImplementationKind, Ir},
};
use tydi_common::{
    cat,
    error::{Error, Result, TryOptional},
    name::PathNameSelf,
    traits::{Document, Identify},
};

use tydi_vhdl::{
    architecture::{arch_storage::Arch, Architecture},
    common::vhdl_name::VhdlName,
    component::Component,
    declaration::Declare,
    port::Port,
};

use crate::IntoVhdl;

pub(crate) type Streamlet = til_query::ir::streamlet::Streamlet;

impl IntoVhdl<Component> for Streamlet {
    fn canonical(
        &self,
        ir_db: &dyn Ir,
        arch_db: &mut dyn Arch,
        prefix: impl TryOptional<VhdlName>,
    ) -> Result<Component> {
        let prefix = prefix.try_optional()?;
        let n = match &prefix {
            Some(some) => cat!(some, self.identifier(), "com"),
            None => cat!(self.identifier(), "com"),
        };

        let mut ports = vec![];
        ports.push(Port::clk());
        ports.push(Port::rst());
        let logical_stream_to_ports = |logical_stream: LogicalStream<Port, SignalList<Port>>| {
            let field_ports = logical_stream.fields().iter().map(|(_, p)| p);
            let stream_ports = logical_stream
                .streams()
                .iter()
                .map(|(_, s)| s.into_iter())
                .flatten();
            field_ports
                .chain(stream_ports)
                .cloned()
                .collect::<Vec<Port>>()
        };

        for input in self.inputs(ir_db) {
            ports.extend(logical_stream_to_ports(
                input
                    .canonical(ir_db, arch_db, prefix.clone())?
                    .logical_stream()
                    .clone(),
            ));
        }
        for output in self.outputs(ir_db) {
            ports.extend(logical_stream_to_ports(
                output
                    .canonical(ir_db, arch_db, prefix.clone())?
                    .logical_stream()
                    .clone(),
            ));
        }

        let mut component = Component::try_new(n, vec![], ports, None)?;
        if let Some(doc) = self.doc() {
            component.set_doc(doc);
        }

        Ok(component)
    }
}

// TODO: Add Component/Entity (or change existing ones) which keeps track of the LogicalStreams and original Interfaces

// TODO: For now, assume architecture output will be a string.
// The architecture for Structural and None is stored in the arch_db.
// Might make more sense/be safer if we could either parse Linked architectures to an object,
// or have some enclosing type which returns either an architecture or a string.
impl IntoVhdl<String> for Streamlet {
    fn canonical(
        &self,
        ir_db: &dyn Ir,
        arch_db: &mut dyn Arch,
        prefix: impl TryOptional<VhdlName>,
    ) -> Result<String> {
        let prefix = prefix.try_optional()?;

        match self.implementation(ir_db) {
            Some(implementation) => match implementation.kind() {
                ImplementationKind::Structural(structure) => {
                    let arch_body = structure.canonical(ir_db, arch_db, prefix)?;
                    let name = implementation.path_name();

                    let mut architecture = if name.len() > 0 {
                        Architecture::from_database(arch_db, name)
                    } else {
                        Architecture::from_database(arch_db, "Behaviour")
                    }?;
                    architecture.add_body(arch_db, &arch_body)?;
                    if let Some(doc) = implementation.doc() {
                        architecture.set_doc(doc);
                    }

                    let result_string = architecture.declare(arch_db)?;
                    arch_db.set_architecture(architecture);

                    Ok(result_string)
                }
                ImplementationKind::Link(link) => {
                    let mut file_pth = link.path().to_path_buf();
                    file_pth.push(self.identifier());
                    file_pth.set_extension("vhd");
                    if file_pth.exists() {
                        if file_pth.is_file() {
                            let result_string = fs::read_to_string(file_pth.as_path())
                                .map_err(|err| Error::FileIOError(err.to_string()))?;
                            Ok(result_string)
                        } else {
                            Err(Error::FileIOError(format!(
                                "Path {} exists, but is not a file.",
                                file_pth.display()
                            )))
                        }
                    } else {
                        let name = implementation.path_name();

                        let architecture = if name.len() > 0 {
                            Architecture::from_database(arch_db, name)
                        } else {
                            Architecture::from_database(arch_db, "Behaviour")
                        }?;

                        // TODO: Make whether to create a file if one doesn't exist configurable (Yes/No/Ask)
                        let result_string = architecture.declare(arch_db)?;
                        fs::write(file_pth.as_path(), &result_string)
                            .map_err(|err| Error::FileIOError(err.to_string()))?;
                        arch_db.set_architecture(architecture);

                        // TODO for much later: Try to incorporate "fancy wrapper" work into this

                        Ok(result_string)
                    }
                }
            },
            None => {
                let architecture = Architecture::from_database(arch_db, "Behavioral")?;

                let result_string = architecture.declare(arch_db)?;
                arch_db.set_architecture(architecture);

                Ok(result_string)
            }
        }
    }
}
