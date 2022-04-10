use std::fs;

use til_query::{
    common::{logical::logical_stream::LogicalStream, physical::signal_list::SignalList},
    ir::{
        implementation::{Implementation, ImplementationKind},
        Ir,
    },
};
use tydi_common::{
    cat,
    error::{Error, Result, TryOptional},
    map::InsertionOrderedMap,
    name::{Name, PathName, PathNameSelf},
    traits::{Document, Identify},
};

use tydi_intern::Id;
use tydi_vhdl::{
    architecture::{arch_storage::Arch, Architecture},
    common::vhdl_name::VhdlName,
    component::Component,
    declaration::Declare,
    port::Port,
};

use crate::IntoVhdl;

use super::interface::VhdlInterface;

pub(crate) type Streamlet = til_query::ir::streamlet::Streamlet;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VhdlStreamlet {
    prefix: Option<VhdlName>,
    name: PathName,
    implementation: Option<Id<Implementation>>,
    interface: InsertionOrderedMap<Name, VhdlInterface>,
    doc: Option<String>,
}

impl VhdlStreamlet {
    pub fn prefix(&self) -> &Option<VhdlName> {
        &self.prefix
    }

    pub fn implementation_id(&self) -> Option<Id<Implementation>> {
        self.implementation
    }

    pub fn implementation(&self, db: &dyn Ir) -> Option<Implementation> {
        if let Some(id) = self.implementation {
            Some(db.lookup_intern_implementation(id))
        } else {
            None
        }
    }

    pub fn interface(&self) -> &InsertionOrderedMap<Name, VhdlInterface> {
        &self.interface
    }

    pub fn to_component(&self) -> Component {
        let n = match self.prefix() {
            Some(some) => cat!(some, self.identifier(), "com"),
            None => cat!(self.identifier(), "com"),
        };

        let mut ports = vec![];
        ports.push(Port::clk());
        ports.push(Port::rst());

        for (name, vhdl_interface) in self.interface() {
            let logical_stream = vhdl_interface.typed_stream().logical_stream();
            let field_ports = logical_stream.fields().iter().map(|(_, p)| p);
            let stream_ports = logical_stream
                .streams()
                .iter()
                .map(|(_, s)| s.signal_list().into_iter())
                .flatten();
            let mut result_ports = field_ports
                .chain(stream_ports)
                .cloned()
                .collect::<Vec<Port>>();
            if let Some(doc) = vhdl_interface.doc() {
                if let Some(port) = result_ports.first_mut() {
                    port.set_doc(doc);
                }
            }
            ports.extend(result_ports);
        }

        let mut component = Component::try_new(n, vec![], ports, None).unwrap();
        if let Some(doc) = self.doc() {
            component.set_doc(doc);
        }

        component
    }
}

impl Identify for VhdlStreamlet {
    fn identifier(&self) -> String {
        self.name.to_string()
    }
}

impl PathNameSelf for VhdlStreamlet {
    fn path_name(&self) -> &PathName {
        &self.name
    }
}

impl Document for VhdlStreamlet {
    fn doc(&self) -> Option<String> {
        self.doc.clone()
    }
}

impl IntoVhdl<VhdlStreamlet> for Streamlet {
    fn canonical(
        &self,
        ir_db: &dyn Ir,
        arch_db: &mut dyn Arch,
        prefix: impl TryOptional<VhdlName>,
    ) -> Result<VhdlStreamlet> {
        let prefix = prefix.try_optional()?;

        let mut interface = InsertionOrderedMap::new();
        for port in self.interface(ir_db).ports(ir_db) {
            interface.try_insert(
                port.name().clone(),
                port.canonical(ir_db, arch_db, prefix.clone())?,
            )?;
        }

        Ok(VhdlStreamlet {
            prefix,
            name: self.path_name().clone(),
            implementation: self.implementation_id(),
            interface,
            doc: self.doc(),
        })
    }
}

// impl IntoVhdl<Component> for Streamlet {
//     fn canonical(
//         &self,
//         ir_db: &dyn Ir,
//         arch_db: &mut dyn Arch,
//         prefix: impl TryOptional<VhdlName>,
//     ) -> Result<Component> {
//         let prefix = prefix.try_optional()?;
//         let n = match &prefix {
//             Some(some) => cat!(some, self.identifier(), "com"),
//             None => cat!(self.identifier(), "com"),
//         };

//         let mut ports = vec![];
//         ports.push(Port::clk());
//         ports.push(Port::rst());
//         let logical_stream_to_ports = |logical_stream: LogicalStream<Port, SignalList<Port>>| {
//             let field_ports = logical_stream.fields().iter().map(|(_, p)| p);
//             let stream_ports = logical_stream
//                 .streams()
//                 .iter()
//                 .map(|(_, s)| s.into_iter())
//                 .flatten();
//             field_ports
//                 .chain(stream_ports)
//                 .cloned()
//                 .collect::<Vec<Port>>()
//         };

//         for input in self.inputs(ir_db) {
//             ports.extend(logical_stream_to_ports(
//                 input
//                     .canonical(ir_db, arch_db, prefix.clone())?
//                     .logical_stream()
//                     .clone(),
//             ));
//         }
//         for output in self.outputs(ir_db) {
//             ports.extend(logical_stream_to_ports(
//                 output
//                     .canonical(ir_db, arch_db, prefix.clone())?
//                     .logical_stream()
//                     .clone(),
//             ));
//         }

//         let mut component = Component::try_new(n, vec![], ports, None)?;
//         if let Some(doc) = self.doc() {
//             component.set_doc(doc);
//         }

//         Ok(component)
//     }
// }

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
