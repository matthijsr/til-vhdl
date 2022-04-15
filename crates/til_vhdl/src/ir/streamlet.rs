use std::{fs, sync::Arc};

use til_query::{
    common::{
        logical::logical_stream::TypedStream,
        physical::{complexity::Complexity, signal_list::SignalList},
        stream_direction::StreamDirection,
    },
    ir::{
        connection::InterfaceReference,
        implementation::{structure::Structure, Implementation, ImplementationKind},
        physical_properties::InterfaceDirection,
        Ir,
    },
};
use tydi_common::{
    cat,
    error::{Error, Result, TryOptional},
    map::InsertionOrderedMap,
    name::{Name, PathName, PathNameSelf},
    numbers::{NonNegative, Positive},
    traits::{Document, Identify},
};

use tydi_intern::Id;
use tydi_vhdl::{
    architecture::{arch_storage::Arch, Architecture},
    assignment::Assign,
    common::vhdl_name::{VhdlName, VhdlNameSelf},
    component::Component,
    declaration::{Declare, ObjectDeclaration},
    port::Port,
    statement::PortMapping,
};

use crate::IntoVhdl;

use super::interface::VhdlInterface;

pub(crate) type Streamlet = til_query::ir::streamlet::Streamlet;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PortObject {
    typed_stream: TypedStream<Id<ObjectDeclaration>, PhysicalStreamObject>,
    interface_direction: InterfaceDirection,
    is_local: bool,
}

impl PortObject {
    pub fn typed_stream(&self) -> &TypedStream<Id<ObjectDeclaration>, PhysicalStreamObject> {
        &self.typed_stream
    }
    pub fn interface_direction(&self) -> &InterfaceDirection {
        &self.interface_direction
    }
    pub fn is_local(&self) -> bool {
        self.is_local
    }

    pub fn is_sink(&self) -> bool {
        match self.interface_direction() {
            InterfaceDirection::Out => self.is_local(),
            InterfaceDirection::In => !self.is_local(),
        }
    }

    pub fn is_source(&self) -> bool {
        !self.is_sink()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PhysicalStreamObject {
    /// Signals associated with this stream
    signal_list: SignalList<Id<ObjectDeclaration>>,
    /// Number of element lanes.
    element_lanes: Positive,
    /// Dimensionality.
    dimensionality: NonNegative,
    /// Complexity.
    complexity: Complexity,
    /// Overall direction of the physical stream
    stream_direction: StreamDirection,
}

impl PhysicalStreamObject {
    /// Signals associated with this stream
    pub fn signal_list(&self) -> &SignalList<Id<ObjectDeclaration>> {
        &self.signal_list
    }
    /// Number of element lanes.
    pub fn element_lanes(&self) -> &Positive {
        &self.element_lanes
    }
    /// Dimensionality.
    pub fn dimensionality(&self) -> NonNegative {
        self.dimensionality
    }
    /// Complexity.
    pub fn complexity(&self) -> &Complexity {
        &self.complexity
    }
    /// Overall direction of the physical stream
    pub fn stream_direction(&self) -> StreamDirection {
        self.stream_direction
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VhdlStreamlet {
    prefix: Option<VhdlName>,
    name: PathName,
    implementation: Option<Id<Implementation>>,
    interface: InsertionOrderedMap<Name, VhdlInterface>,
    doc: Option<String>,
    component: Option<Arc<Component>>,
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

    pub fn to_component(&mut self) -> Arc<Component> {
        if let Some(component) = &self.component {
            component.clone()
        } else {
            let n = match self.prefix() {
                Some(some) => cat!(some, self.identifier(), "com"),
                None => cat!(self.identifier(), "com"),
            };

            let mut ports = vec![];
            ports.push(Port::clk());
            ports.push(Port::rst());

            for (_, vhdl_interface) in self.interface() {
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

            let component = Arc::new(component);
            self.component = Some(component.clone());

            component
        }
    }

    // TODO: For now, assume architecture output will be a string.
    // The architecture for Structural and None is stored in the arch_db.
    // Might make more sense/be safer if we could either parse Linked architectures to an object,
    // or have some enclosing type which returns either an architecture or a string.
    pub fn to_architecture(&self, ir_db: &dyn Ir, arch_db: &mut dyn Arch) -> Result<String> {
        match self.implementation(ir_db) {
            Some(implementation) => match implementation.kind() {
                ImplementationKind::Structural(structure) => {
                    self.structural_arch(structure, ir_db, arch_db, &implementation)
                }
                ImplementationKind::Link(link) => self.link_arch(link, &implementation, arch_db),
            },
            None => {
                let architecture = Architecture::from_database(arch_db, "Behavioral")?;

                let result_string = architecture.declare(arch_db)?;
                arch_db.set_architecture(architecture);

                Ok(result_string)
            }
        }
    }

    fn link_arch(
        &self,
        link: &til_query::ir::implementation::link::Link,
        implementation: &Implementation,
        arch_db: &mut dyn Arch,
    ) -> Result<String> {
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

    fn structural_arch(
        &self,
        structure: &Structure,
        ir_db: &dyn Ir,
        arch_db: &mut dyn Arch,
        implementation: &Implementation,
    ) -> Result<String> {
        structure.validate_connections(ir_db)?;

        let name = implementation.path_name();
        let mut architecture = if name.len() > 0 {
            Architecture::from_database(arch_db, name)
        } else {
            Architecture::from_database(arch_db, "Behaviour")
        }?;

        if let Some(doc) = implementation.doc() {
            architecture.set_doc(doc);
        }

        let mut ports = InsertionOrderedMap::new();
        let entity_port_obj = |p| ObjectDeclaration::from_port(arch_db, &p, true);
        for (name, port) in self.interface() {
            ports.try_insert(
                InterfaceReference::new(None, name.clone()),
                PortObject {
                    interface_direction: port.physical_properties().direction(),
                    typed_stream: port.typed_stream().map_logical_stream(|ls| {
                        ls.clone()
                            .map(entity_port_obj, |stream| PhysicalStreamObject {
                                signal_list: stream.signal_list().clone().map(entity_port_obj),
                                element_lanes: stream.element_lanes().clone(),
                                dimensionality: stream.dimensionality(),
                                complexity: stream.complexity().clone(),
                                stream_direction: stream.stream_direction(),
                            })
                    }),
                    is_local: true,
                },
            )?;
        }

        let clk = ObjectDeclaration::entity_clk(arch_db);
        let rst = ObjectDeclaration::entity_rst(arch_db);

        for (instance_name, streamlet) in structure.streamlet_instances(ir_db) {
            let mut streamlet = streamlet.canonical(ir_db, arch_db, self.prefix().clone())?;
            let identifier = streamlet.identifier();
            let wrap_portmap_err = |result: Result<()>| -> Result<()> {
                match result {
                        Ok(result) => Ok(result),
                        Err(err) => Err(Error::BackEndError(format!(
                    "Something went wrong trying to generate port mappings for streamlet instance {} (type: {}):\n\t{}",
                    &instance_name, identifier, err
                ))),
                    }
            };

            let component = streamlet.to_component();
            let mut port_mapping =
                PortMapping::from_component(arch_db, &component, instance_name.clone())?;

            for (name, port) in streamlet.interface() {
                let mut try_signal_decl = |p: Port| {
                    let signal = ObjectDeclaration::signal(
                        arch_db,
                        format!("{}__{}", instance_name, p.identifier()),
                        p.typ().clone(),
                        None,
                    )?;
                    wrap_portmap_err(port_mapping.map_port(
                        arch_db,
                        p.vhdl_name().clone(),
                        &signal,
                    ))?;

                    architecture.add_declaration(arch_db, signal)?;

                    Ok(signal)
                };

                ports.try_insert(
                    InterfaceReference::new(Some(instance_name.clone()), name.clone()),
                    PortObject {
                        interface_direction: port.physical_properties().direction(),
                        typed_stream: port.typed_stream().try_map_logical_stream(|ls| {
                            ls.clone()
                                .try_map_fields(&mut try_signal_decl)?
                                .try_map_streams(|stream| {
                                    Ok(PhysicalStreamObject {
                                        signal_list: stream
                                            .signal_list()
                                            .clone()
                                            .try_map(&mut try_signal_decl)?,
                                        element_lanes: stream.element_lanes().clone(),
                                        dimensionality: stream.dimensionality(),
                                        complexity: stream.complexity().clone(),
                                        stream_direction: stream.stream_direction(),
                                    })
                                })
                        })?,
                        is_local: false,
                    },
                )?;

                wrap_portmap_err(port_mapping.map_port(arch_db, "clk", &clk))?;
                wrap_portmap_err(port_mapping.map_port(arch_db, "rst", &rst))?;
            }
            architecture.add_statement(arch_db, port_mapping.finish()?)?;
        }

        for connection in structure.connections() {
            let sink = ports
                .get(connection.sink())
                .ok_or(Error::ProjectError(format!(
                    "Port {} does not exist, cannot connect {}.",
                    connection.sink(),
                    connection,
                )))?;
            let source = ports
                .get(connection.source())
                .ok_or(Error::ProjectError(format!(
                    "Port {} does not exist, cannot connect {}.",
                    connection.source(),
                    connection,
                )))?;
            if sink.is_sink() && source.is_sink() || sink.is_source() && source.is_source() {
                todo!()
            }
            let (sink, source) = if sink.is_sink() {
                (sink, source)
            } else {
                (source, sink)
            };

            for (name, field) in sink.typed_stream().logical_stream().fields() {
                architecture.add_statement(
                    arch_db,
                    field.assign(
                        arch_db,
                        source
                            .typed_stream()
                            .logical_stream()
                            .fields()
                            .try_get(name)?,
                    )?,
                )?;
            }

            let mut assign = |left: &Option<Id<ObjectDeclaration>>,
                              right: &Option<Id<ObjectDeclaration>>|
             -> Result<()> {
                match (left, right) {
                    (Some(left), Some(right)) => {
                        architecture.add_statement(arch_db, left.assign(arch_db, right)?)
                    }
                    (None, None) => Ok(()),
                    _ => todo!(),
                }
            };

            for (name, sink_obj) in sink.typed_stream().logical_stream().streams() {
                let source_obj = source
                    .typed_stream()
                    .logical_stream()
                    .streams()
                    .try_get(name)?;
                if sink_obj.stream_direction() == source_obj.stream_direction() {
                    let (sink_obj, source_obj) =
                        if sink_obj.stream_direction() == StreamDirection::Reverse {
                            (source_obj, sink_obj)
                        } else {
                            (sink_obj, source_obj)
                        };
                    assign(
                        sink_obj.signal_list().valid(),
                        source_obj.signal_list().valid(),
                    )?;
                    assign(
                        source_obj.signal_list().ready(),
                        sink_obj.signal_list().ready(),
                    )?;
                    assign(
                        sink_obj.signal_list().data(),
                        source_obj.signal_list().data(),
                    )?;
                    assign(
                        sink_obj.signal_list().last(),
                        source_obj.signal_list().last(),
                    )?;
                    assign(
                        sink_obj.signal_list().stai(),
                        source_obj.signal_list().stai(),
                    )?;
                    assign(
                        sink_obj.signal_list().endi(),
                        source_obj.signal_list().endi(),
                    )?;
                    assign(
                        sink_obj.signal_list().strb(),
                        source_obj.signal_list().strb(),
                    )?;
                    assign(
                        sink_obj.signal_list().user(),
                        source_obj.signal_list().user(),
                    )?;
                } else {
                    todo!()
                }
            }
        }

        let result_string = architecture.declare(arch_db)?;
        arch_db.set_architecture(architecture);
        Ok(result_string)
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
            component: None,
        })
    }
}
