use std::{fs, sync::Arc};

use til_query::{
    common::{
        logical::logical_stream::TypedStream,
        physical::{complexity::Complexity, signal_list::SignalList},
        stream_direction::StreamDirection,
    },
    ir::{
        connection::InterfaceReference,
        implementation::{link::Link, structure::Structure, Implementation, ImplementationKind},
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
    traits::{Document, Documents, Identify},
};

use tydi_intern::Id;
use tydi_vhdl::{
    architecture::{arch_storage::Arch, Architecture},
    assignment::Assign,
    common::vhdl_name::{VhdlName, VhdlNameSelf},
    component::Component,
    declaration::{Declare, DeclareWithIndent, ObjectDeclaration},
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
    /// The name of the Stream, including its interface
    name: PathName,
    /// Signals associated with this stream
    signal_list: SignalList<Id<ObjectDeclaration>>,
    /// Number of element lanes.
    element_lanes: Positive,
    /// Dimensionality.
    dimensionality: NonNegative,
    /// Complexity.
    complexity: Complexity,
    /// Direction of the parent interface.
    interface_direction: InterfaceDirection,
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

    /// Get the physical stream object's interface direction.
    #[must_use]
    pub fn interface_direction(&self) -> InterfaceDirection {
        self.interface_direction
    }
}

impl PathNameSelf for PhysicalStreamObject {
    fn path_name(&self) -> &PathName {
        &self.name
    }
}

impl Identify for PhysicalStreamObject {
    fn identifier(&self) -> String {
        self.path_name().to_string()
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StreamletArchitecture {
    Imported(String),
    Generated(Architecture),
}

impl DeclareWithIndent for StreamletArchitecture {
    fn declare_with_indent(&self, db: &dyn Arch, indent_style: &str) -> Result<String> {
        match self {
            StreamletArchitecture::Imported(i) => Ok(i.clone()),
            StreamletArchitecture::Generated(g) => g.declare_with_indent(db, indent_style),
        }
    }
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
    pub fn to_architecture(
        &self,
        ir_db: &dyn Ir,
        arch_db: &mut dyn Arch,
    ) -> Result<StreamletArchitecture> {
        match self.implementation(ir_db) {
            Some(implementation) => match implementation.kind() {
                ImplementationKind::Structural(structure) => {
                    self.structural_arch(structure, ir_db, arch_db, &implementation)
                }
                ImplementationKind::Link(link) => self.link_arch(link, &implementation, arch_db),
            },
            None => {
                let architecture = Architecture::from_database(arch_db, "Behavioral")?;

                Ok(StreamletArchitecture::Generated(architecture))
            }
        }
    }

    fn link_arch(
        &self,
        link: &Link,
        implementation: &Implementation,
        arch_db: &mut dyn Arch,
    ) -> Result<StreamletArchitecture> {
        let mut file_pth = link.path().to_path_buf();
        file_pth.push(self.identifier());
        file_pth.set_extension("vhd");
        if file_pth.exists() {
            if file_pth.is_file() {
                let result_string = fs::read_to_string(file_pth.as_path())
                    .map_err(|err| Error::FileIOError(err.to_string()))?;
                Ok(StreamletArchitecture::Imported(result_string))
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

            // TODO for much later: Try to incorporate "fancy wrapper" work into this

            Ok(StreamletArchitecture::Generated(architecture))
        }
    }

    /// Creates a port-mapping for this Streamlet and returns a map of signals
    /// for its interfaces
    pub fn to_instance(
        &mut self,
        arch_db: &mut dyn Arch,
        instance_name: Name,
        architecture: &mut Architecture,
        clk: Id<ObjectDeclaration>,
        rst: Id<ObjectDeclaration>,
    ) -> Result<InsertionOrderedMap<InterfaceReference, PortObject>> {
        let identifier = self.identifier();
        let wrap_portmap_err = |result: Result<()>| -> Result<()> {
            match result {
                        Ok(result) => Ok(result),
                        Err(err) => Err(Error::BackEndError(format!(
                    "Something went wrong trying to generate port mappings for streamlet instance {} (type: {}):\n\t{}",
                    &instance_name, identifier, err
                ))),
                    }
        };

        let component = self.to_component();
        let mut port_mapping =
            PortMapping::from_component(arch_db, &component, instance_name.clone())?;

        let mut signals = InsertionOrderedMap::new();

        for (name, port) in self.interface() {
            let mut try_signal_decl = |p: Port| {
                let signal = ObjectDeclaration::signal(
                    arch_db,
                    format!("{}__{}", instance_name, p.identifier()),
                    p.typ().clone(),
                    None,
                )?;
                wrap_portmap_err(port_mapping.map_port(arch_db, p.vhdl_name().clone(), &signal))?;

                architecture.add_declaration(arch_db, signal)?;

                Ok(signal)
            };

            signals.try_insert(
                InterfaceReference::new(Some(instance_name.clone()), name.clone()),
                PortObject {
                    interface_direction: port.physical_properties().direction(),
                    typed_stream: port.typed_stream().try_map_logical_stream(|ls| {
                        ls.clone()
                            .try_map_fields(&mut try_signal_decl)?
                            .try_map_streams_named(|stream_name, stream| {
                                Ok(PhysicalStreamObject {
                                    name: stream_name.clone(),
                                    signal_list: stream
                                        .signal_list()
                                        .clone()
                                        .try_map(&mut try_signal_decl)?,
                                    element_lanes: stream.element_lanes().clone(),
                                    dimensionality: stream.dimensionality(),
                                    complexity: stream.complexity().clone(),
                                    interface_direction: stream.interface_direction(),
                                    stream_direction: stream.stream_direction(),
                                })
                            })
                    })?,
                    is_local: false,
                },
            )?;
        }
        wrap_portmap_err(port_mapping.map_port(arch_db, "clk", &clk))?;
        wrap_portmap_err(port_mapping.map_port(arch_db, "rst", &rst))?;
        architecture.add_statement(arch_db, port_mapping.finish()?)?;

        Ok(signals)
    }

    fn structural_arch(
        &self,
        structure: &Structure,
        ir_db: &dyn Ir,
        arch_db: &mut dyn Arch,
        implementation: &Implementation,
    ) -> Result<StreamletArchitecture> {
        structure.validate_connections(ir_db)?;

        let mut architecture = if implementation.path_name().len() > 0 {
            Architecture::from_database(arch_db, implementation.path_name())
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
                        ls.clone().map_fields(entity_port_obj).map_streams_named(
                            |stream_name, stream| PhysicalStreamObject {
                                name: stream_name.clone(),
                                signal_list: stream.signal_list().clone().map(entity_port_obj),
                                element_lanes: stream.element_lanes().clone(),
                                dimensionality: stream.dimensionality(),
                                complexity: stream.complexity().clone(),
                                interface_direction: stream.interface_direction(),
                                stream_direction: stream.stream_direction(),
                            },
                        )
                    }),
                    is_local: true,
                },
            )?;
        }

        let clk = ObjectDeclaration::entity_clk(arch_db);
        let rst = ObjectDeclaration::entity_rst(arch_db);

        for (instance_name, streamlet) in structure.streamlet_instances(ir_db) {
            let mut streamlet = streamlet.canonical(ir_db, arch_db, self.prefix().clone())?;
            ports.try_append(streamlet.to_instance(
                arch_db,
                instance_name,
                &mut architecture,
                clk,
                rst,
            )?)?;
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
                return Err(Error::ProjectError(format!(
                    "Something went wrong with connection {}: Both ports are a {}.",
                    connection,
                    if sink.is_sink() { "sink" } else { "source" }
                )));
            }
            let (sink, source) = if sink.is_sink() {
                (sink, source)
            } else {
                (source, sink)
            };

            for (field_name, field) in sink.typed_stream().logical_stream().fields() {
                architecture.add_statement(
                    arch_db,
                    field.assign(
                        arch_db,
                        source
                            .typed_stream()
                            .logical_stream()
                            .fields()
                            .try_get(field_name)?,
                    )?,
                )?;
            }

            let mut assign = |left: &Option<Id<ObjectDeclaration>>,
                              right: &Option<Id<ObjectDeclaration>>,
                              sig_name: &str|
             -> Result<()> {
                match (left, right) {
                    (Some(left), Some(right)) => {
                        architecture.add_statement(arch_db, left.assign(arch_db, right)?)
                    }
                    (None, None) => Ok(()),
                    (Some(_), None) => Err(Error::ProjectError(format!(
                        "Something went wrong with connection {}: Signal {} does not exist on the source.",
                        connection,
                        sig_name,
                    ))),
                    (None, Some(_)) => Err(Error::ProjectError(format!(
                        "Something went wrong with connection {}: Signal {} does not exist on the sink.",
                        connection,
                        sig_name,
                    ))),
                }
            };

            for (stream_name, sink_obj) in sink.typed_stream().logical_stream().streams() {
                let source_obj = source
                    .typed_stream()
                    .logical_stream()
                    .streams()
                    .try_get(stream_name)?;
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
                        "valid",
                    )?;
                    assign(
                        source_obj.signal_list().ready(),
                        sink_obj.signal_list().ready(),
                        "ready",
                    )?;
                    assign(
                        sink_obj.signal_list().data(),
                        source_obj.signal_list().data(),
                        "data",
                    )?;
                    assign(
                        sink_obj.signal_list().last(),
                        source_obj.signal_list().last(),
                        "last",
                    )?;
                    assign(
                        sink_obj.signal_list().stai(),
                        source_obj.signal_list().stai(),
                        "stai",
                    )?;
                    assign(
                        sink_obj.signal_list().endi(),
                        source_obj.signal_list().endi(),
                        "endi",
                    )?;
                    assign(
                        sink_obj.signal_list().strb(),
                        source_obj.signal_list().strb(),
                        "strb",
                    )?;
                    assign(
                        sink_obj.signal_list().user(),
                        source_obj.signal_list().user(),
                        "user",
                    )?;
                } else {
                    return Err(Error::ProjectError(format!("Something went wrong with connection {}: The stream {} has an opposite direction on these ports.", connection, stream_name)));
                }
            }
        }

        Ok(StreamletArchitecture::Generated(architecture))
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
    fn doc(&self) -> Option<&String> {
        self.doc.as_ref()
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
            doc: self.doc().cloned(),
            component: None,
        })
    }
}
