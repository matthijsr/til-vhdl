use std::collections::{BTreeMap, HashSet};

use tydi_common::{
    error::{Error, Result, TryOptional, TryResult},
    name::Name,
    traits::Identify,
};
use tydi_intern::Id;

use crate::ir::{
    connection::InterfaceReference, physical_properties::InterfaceDirection, Connection, GetSelf,
    Interface, IntoVhdl, Ir, Streamlet,
};

/// This node represents a structural `Implementation`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Structure {
    ports: BTreeMap<Name, Id<Interface>>,
    streamlet_instances: BTreeMap<Name, Id<Streamlet>>,
    connections: Vec<Connection>,
}

impl Structure {
    pub fn new(ports: impl Into<BTreeMap<Name, Id<Interface>>>) -> Self {
        Structure {
            ports: ports.into(),
            streamlet_instances: BTreeMap::new(),
            connections: vec![],
        }
    }

    pub fn port_ids(&self) -> &BTreeMap<Name, Id<Interface>> {
        &self.ports
    }

    pub fn ports(&self, db: &dyn Ir) -> BTreeMap<Name, Interface> {
        self.port_ids()
            .iter()
            .map(|(name, id)| (name.clone(), id.get(db)))
            .collect()
    }

    pub fn interface_references(&self, db: &dyn Ir) -> Vec<InterfaceReference> {
        let mut result = self.local_interface_references();
        result.extend(self.streamlet_instance_interface_references(db));
        result
    }

    pub fn local_interface_references(&self) -> Vec<InterfaceReference> {
        self.port_ids()
            .keys()
            .map(|name| InterfaceReference::new(None, name.clone()))
            .collect()
    }

    pub fn streamlet_instance_interface_references(&self, db: &dyn Ir) -> Vec<InterfaceReference> {
        self.streamlet_instances(db)
            .iter()
            .flat_map(|(name, streamlet)| {
                streamlet
                    .port_ids()
                    .keys()
                    .map(|port| InterfaceReference::new(Some(name.clone()), port.clone()))
                    .collect::<Vec<InterfaceReference>>()
            })
            .collect()
    }

    pub fn streamlet_instance_ids(&self) -> &BTreeMap<Name, Id<Streamlet>> {
        &self.streamlet_instances
    }

    pub fn streamlet_instances(&self, db: &dyn Ir) -> BTreeMap<Name, Streamlet> {
        self.streamlet_instance_ids()
            .iter()
            .map(|(name, id)| (name.clone(), id.get(db)))
            .collect()
    }

    pub fn try_add_connection(
        &mut self,
        db: &dyn Ir,
        left: impl TryResult<InterfaceReference>,
        right: impl TryResult<InterfaceReference>,
    ) -> Result<()> {
        let left = left.try_result()?;
        let right = right.try_result()?;

        struct InterfaceAndStructure {
            on_streamlet: bool,
            interface: Interface,
        }

        let get_port = |i: &InterfaceReference| match i.streamlet_instance() {
            Some(streamlet_instance) => self
                .try_get_streamlet_instance(streamlet_instance)
                .and_then(|x| {
                    Ok(InterfaceAndStructure {
                        on_streamlet: true,
                        interface: x.get(db).try_get_port(db, i.port())?,
                    })
                }),
            None => match self.port_ids().get(i.port()) {
                Some(port) => Ok(InterfaceAndStructure {
                    on_streamlet: false,
                    interface: port.get(db),
                }),
                None => Err(Error::InvalidArgument(format!(
                    "No port with name {} exists within this structure",
                    i.port()
                ))),
            },
        };
        let left_i = get_port(&left)?;
        let right_i = get_port(&right)?;
        // Interfaces are on the same layer if they both either belong to the structure or to a streamlet instance
        let same_layer = left_i.on_streamlet == right_i.on_streamlet;

        if left_i.interface.stream_id() == right_i.interface.stream_id()
            // If the interfaces are on the same layer, their directions should be opposite.
            // If they are not on the same layer, their directions should be the same.
            && same_layer == (left_i.interface.direction() != right_i.interface.direction())
        {
            let (source, sink) = match left_i.interface.direction() {
                // If left_interface belongs to a streamlet instance, Out means it's a Source
                InterfaceDirection::Out if left_i.on_streamlet => (left, right),
                // Otherwise, it belongs to the structure, and is a Sink
                InterfaceDirection::Out => (right, left),
                // Likewise, In means it is a Sink if left_interface is a streamlet instance
                InterfaceDirection::In if left_i.on_streamlet => (right, left),
                // But it is a Source if it belongs to the structure
                InterfaceDirection::In => (left, right),
            };

            self.connections.push(Connection::new(source, sink));
            Ok(())
        } else {
            Err(Error::InvalidTarget(format!(
                "The ports {} and {} are incompatible",
                left, right
            )))
        }
    }

    pub fn try_add_streamlet_instance(
        &mut self,
        name: impl TryResult<Name>,
        streamlet: Id<Streamlet>,
    ) -> Result<()> {
        let name = name.try_result()?;
        if self.streamlet_instance_ids().contains_key(&name) {
            Err(Error::InvalidArgument(format!(
                "A streamlet instance with name {} already exists in this structure",
                name
            )))
        } else {
            self.streamlet_instances.insert(name, streamlet);
            Ok(())
        }
    }

    pub fn try_get_streamlet_instance(&self, name: &Name) -> Result<Id<Streamlet>> {
        match self.streamlet_instance_ids().get(name) {
            Some(streamlet) => Ok(*streamlet),
            None => Err(Error::InvalidArgument(format!(
                "A streamlet instance with name {} does not exist in this structure",
                name
            ))),
        }
    }

    pub fn connections(&self) -> &Vec<Connection> {
        &self.connections
    }

    /// Verifies whether all ports (on the structure and all instances) have been connected,
    /// also verifies whether ports have duplicate connections.
    ///
    /// Returns a ProjectError if not all ports have been connected, or if some ports are used multiple times.
    pub fn validate_connections(&self, db: &dyn Ir) -> Result<()> {
        let mut sources = HashSet::new();
        let mut sinks = HashSet::new();
        for connection in self.connections() {
            if !sources.insert(connection.source().clone()) {
                return Err(Error::ProjectError(format!(
                    "Duplicate use of Source {}",
                    connection.source()
                )));
            }
            if !sinks.insert(connection.sink().clone()) {
                return Err(Error::ProjectError(format!(
                    "Duplicate use of Sink {}",
                    connection.sink()
                )));
            }
        }

        for interface in self.interface_references(db) {
            if !(sources.contains(&interface) || sinks.contains(&interface)) {
                return Err(Error::ProjectError(format!(
                    "Port {} has not been connected",
                    interface
                )));
            }
        }

        Ok(())
    }
}

impl From<&Streamlet> for Structure {
    fn from(streamlet: &Streamlet) -> Self {
        Structure::new(streamlet.port_ids().clone())
    }
}

impl IntoVhdl<ArchitectureBody> for Structure {
    fn canonical(
        &self,
        ir_db: &dyn Ir,
        arch_db: &mut dyn Arch,
        prefix: impl TryOptional<Name>,
    ) -> Result<ArchitectureBody> {
        self.validate_connections(ir_db)?;
        let prefix = prefix.try_optional()?;

        let mut declarations = vec![];
        let mut statements = vec![];
        let own_ports = self
            .ports(ir_db)
            .iter()
            .map(|(name, port)| {
                Ok((
                    name.clone(),
                    port.canonical(ir_db, arch_db, prefix.clone())?
                        .iter()
                        .map(|vhdl_port| ObjectDeclaration::from_port(arch_db, vhdl_port, true))
                        .collect(),
                ))
            })
            .collect::<Result<BTreeMap<Name, Vec<Id<ObjectDeclaration>>>>>()?;
        let clk = ObjectDeclaration::entity_clk(arch_db);
        let rst = ObjectDeclaration::entity_rst(arch_db);
        let mut streamlet_ports = BTreeMap::new();
        let mut streamlet_components = BTreeMap::new();
        for (instance_name, streamlet) in self.streamlet_instances(ir_db) {
            let component = streamlet.canonical(ir_db, arch_db, prefix.clone())?;
            let mut port_mapping =
                PortMapping::from_component(arch_db, &component, instance_name.clone())?;
            streamlet_components.insert(instance_name.clone(), component);
            let port_map_signals = streamlet
                .port_ids()
                .iter()
                .map(|(name, id)| {
                    let ports = id.get(ir_db).canonical(ir_db, arch_db, prefix.clone())?;
                    let mut signals = vec![];
                    for port in ports {
                        let signal = ObjectDeclaration::signal(
                            arch_db,
                            format!("{}__{}", instance_name, port.identifier()),
                            port.typ().clone(),
                            None,
                        )?;
                        port_mapping.map_port(arch_db, port.vhdl_name().clone(), &signal)?;
                        signals.push(signal);
                        declarations.push(signal.into());
                    }

                    Ok((name.clone(), signals))
                })
                .collect::<Result<BTreeMap<Name, Vec<Id<ObjectDeclaration>>>>>()?;
            streamlet_ports.insert(instance_name, port_map_signals);
            port_mapping.map_port(arch_db, "clk", &clk)?;
            port_mapping.map_port(arch_db, "rst", &rst)?;
            statements.push(port_mapping.finish()?.into());
        }
        for connection in self.connections() {
            let get_objs = |interface: &InterfaceReference| -> &Vec<Id<ObjectDeclaration>> {
                match interface.streamlet_instance() {
                    Some(streamlet_instance) => streamlet_ports
                        .get(streamlet_instance)
                        .unwrap()
                        .get(interface.port())
                        .unwrap(),
                    None => own_ports.get(interface.port()).unwrap(),
                }
            };
            let sink_objs = get_objs(connection.sink()).clone();
            let source_objs = get_objs(connection.source()).clone();
            let sink_source: Vec<(Id<ObjectDeclaration>, Id<ObjectDeclaration>)> =
                sink_objs.into_iter().zip(source_objs.into_iter()).collect();
            for (sink, source) in sink_source {
                statements.push(sink.assign(arch_db, &source)?.into());
            }
        }

        Ok(ArchitectureBody::new(declarations, statements))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        common::logical::logicaltype::LogicalType,
        ir::{Database, TryIntern},
        test_utils::{test_stream_id, test_stream_id_custom},
    };

    use super::*;

    #[test]
    fn try_add_connection() -> Result<()> {
        let _db = Database::default();
        let db = &_db;
        let stream = test_stream_id(db, 4)?;
        let streamlet = Streamlet::try_new(
            db,
            "a",
            vec![
                ("a", stream, InterfaceDirection::In),
                ("b", stream, InterfaceDirection::Out),
            ],
        )?;
        let mut structure = Structure::from(&streamlet);
        structure
            .try_add_streamlet_instance("instance", streamlet.with_implementation(db, None))?;
        structure.try_add_connection(db, "a", ("instance", "a"))?;

        Ok(())
    }

    #[test]
    fn try_validate_connections() -> Result<()> {
        let _db = Database::default();
        let db = &_db;
        let stream = test_stream_id(db, 4)?;
        let streamlet = Streamlet::try_new(
            db,
            "a",
            vec![
                ("a", stream, InterfaceDirection::In),
                ("b", stream, InterfaceDirection::Out),
            ],
        )?;

        let mut structure = Structure::from(&streamlet);
        structure
            .try_add_streamlet_instance("instance", streamlet.with_implementation(db, None))?;
        structure.try_add_connection(db, "a", ("instance", "a"))?;

        // Test: should throw an error if a port is unconnected
        assert_eq!(
            structure.validate_connections(db),
            Err(Error::ProjectError(
                "Port b has not been connected".to_string()
            )),
        );

        // Test: Should no longer throw an error if all ports are connected
        structure.try_add_connection(db, "b", ("instance", "b"))?;
        structure.validate_connections(db)?;

        // Test: should throw an error if a port is connected multiple times
        structure.try_add_connection(db, "a", ("instance", "a"))?;
        assert_eq!(
            structure.validate_connections(db),
            Err(Error::ProjectError("Duplicate use of Source a".to_string())),
        );

        Ok(())
    }

    #[test]
    fn try_get_streamlet_instance() -> Result<()> {
        let _db = Database::default();
        let db = &_db;
        let stream = test_stream_id(db, 4)?;
        let streamlet = Streamlet::try_new(
            db,
            "a",
            vec![
                ("a", stream, InterfaceDirection::In),
                ("b", stream, InterfaceDirection::Out),
            ],
        )?;
        let mut structure = Structure::from(&streamlet);
        structure
            .try_add_streamlet_instance("instance", streamlet.with_implementation(db, None))?;
        structure.try_get_streamlet_instance(&("instance".try_result()?))?;

        Ok(())
    }
}
