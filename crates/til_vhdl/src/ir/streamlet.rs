use til_query::ir::{implementation::ImplementationKind, Ir};
use tydi_common::{
    cat,
    error::{Result, TryOptional},
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
        let n: String = match &prefix {
            Some(some) => cat!(some, self.identifier(), "com"),
            None => cat!(self.identifier(), "com"),
        };

        let mut ports = vec![];
        ports.push(Port::clk());
        ports.push(Port::rst());
        for input in self.inputs(ir_db) {
            ports.extend(input.canonical(ir_db, arch_db, prefix.clone())?);
        }
        for output in self.outputs(ir_db) {
            ports.extend(output.canonical(ir_db, arch_db, prefix.clone())?);
        }

        let mut component = Component::new(VhdlName::try_new(n)?, vec![], ports, None);
        if let Some(doc) = self.doc() {
            component.set_doc(doc);
        }

        Ok(component)
    }
}

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
                ImplementationKind::Link(_) => todo!(),
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
