use std::{
    collections::{BTreeMap, HashSet},
    convert::{TryFrom, TryInto},
    sync::Arc,
};

use tydi_common::{
    error::{Error, Result, TryOptional, TryResult},
    map::InsertionOrderedMap,
    name::Name,
};
use tydi_intern::Id;

use crate::ir::{
    connection::{Connection, InterfaceReference},
    physical_properties::{Domain, InterfaceDirection},
    project::interface::Interface,
    traits::{GetSelf, MoveDb},
    InterfacePort, Ir, Streamlet,
};

use self::streamlet_instance::StreamletInstance;

pub mod streamlet_instance;

/// This node represents a structural `Implementation`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Structure {
    interface: Id<Arc<Interface>>,
    streamlet_instances: BTreeMap<Name, StreamletInstance>,
    connections: Vec<Connection>,
}

impl Structure {
    pub fn new(interface: Id<Arc<Interface>>) -> Self {
        Structure {
            interface,
            streamlet_instances: BTreeMap::new(),
            connections: vec![],
        }
    }

    pub fn interface_id(&self) -> Id<Arc<Interface>> {
        self.interface
    }

    pub fn interface(&self, db: &dyn Ir) -> Arc<Interface> {
        self.interface_id().get(db)
    }

    pub fn ports(&self, db: &dyn Ir) -> InsertionOrderedMap<Name, InterfacePort> {
        self.interface(db).ports().clone()
    }

    pub fn interface_references(&self, db: &dyn Ir) -> Vec<InterfaceReference> {
        let mut result = self.local_interface_references(db);
        result.extend(self.streamlet_instance_interface_references());
        result
    }

    pub fn local_interface_references(&self, db: &dyn Ir) -> Vec<InterfaceReference> {
        self.interface(db)
            .ports()
            .keys()
            .map(|name| InterfaceReference::new(None, name.clone()))
            .collect()
    }

    pub fn streamlet_instance_interface_references(&self) -> Vec<InterfaceReference> {
        self.streamlet_instances()
            .iter()
            .flat_map(|(name, streamlet)| {
                streamlet
                    .ports()
                    .keys()
                    .map(|port| InterfaceReference::new(Some(name.clone()), port.clone()))
                    .collect::<Vec<InterfaceReference>>()
            })
            .collect()
    }

    pub fn streamlet_instances(&self) -> &BTreeMap<Name, StreamletInstance> {
        &self.streamlet_instances
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
            interface: InterfacePort,
        }

        let get_port = |i: &InterfaceReference| match i.streamlet_instance() {
            Some(streamlet_instance) => self
                .try_get_streamlet_instance(streamlet_instance)
                .and_then(|x| {
                    Ok(InterfaceAndStructure {
                        on_streamlet: true,
                        interface: x.try_get_port(i.port())?,
                    })
                }),
            None => match self.interface(db).ports().get(i.port()) {
                Some(port) => Ok(InterfaceAndStructure {
                    on_streamlet: false,
                    interface: port.clone(),
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
        db: &dyn Ir,
        name: impl TryResult<Name>,
        streamlet: Id<Arc<Streamlet>>,
        assignments: impl IntoIterator<Item = (impl TryOptional<Domain>, impl TryResult<Domain>)>,
    ) -> Result<()> {
        let name = name.try_result()?;
        if self.streamlet_instances().contains_key(&name) {
            Err(Error::InvalidArgument(format!(
                "A streamlet instance with name {} already exists in this structure",
                name
            )))
        } else {
            self.streamlet_instances.insert(name.clone(), StreamletInstance::new(db, name, streamlet, assignments)?);
            Ok(())
        }
    }

    pub fn try_add_streamlet_instance_default(
        &mut self,
        db: &dyn Ir,
        name: impl TryResult<Name>,
        streamlet: Id<Arc<Streamlet>>,
    ) -> Result<()> {
        let name = name.try_result()?;
        if self.streamlet_instances().contains_key(&name) {
            Err(Error::InvalidArgument(format!(
                "A streamlet instance with name {} already exists in this structure",
                name
            )))
        } else {
            self.streamlet_instances.insert(name.clone(), StreamletInstance::new_assign_default(db, name, streamlet)?);
            Ok(())
        }
    }

    pub fn try_get_streamlet_instance(&self, name: &Name) -> Result<&StreamletInstance> {
        match self.streamlet_instances().get(name) {
            Some(streamlet) => Ok(streamlet),
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

impl TryFrom<&Streamlet> for Structure {
    type Error = Error;
    fn try_from(streamlet: &Streamlet) -> Result<Self> {
        Ok(Structure::new(streamlet.try_into()?))
    }
}

impl MoveDb<Structure> for Structure {
    fn move_db(
        &self,
        original_db: &dyn Ir,
        target_db: &dyn Ir,
        prefix: &Option<Name>,
    ) -> Result<Structure> {
        let interface = self.interface.move_db(original_db, target_db, prefix)?;
        // TODO
        // let streamlet_instances = self
        //     .streamlet_instances
        //     .iter()
        //     .map(|(k, v)| Ok((k.clone(), v.move_db(original_db, target_db, prefix)?)))
        //     .collect::<Result<_>>()?;
        let connections = self.connections.clone();
        Ok(Structure {
            streamlet_instances: BTreeMap::new(),
            connections,
            interface,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ir::{db::Database, traits::InternArc},
        test_utils::test_stream_id,
    };

    use super::*;

    #[test]
    fn try_add_connection() -> Result<()> {
        let _db = Database::default();
        let db = &_db;
        let stream = test_stream_id(db, 4)?;
        let streamlet = Streamlet::new().try_with_name("a")?.with_ports(
            db,
            vec![
                ("a", stream, InterfaceDirection::In),
                ("b", stream, InterfaceDirection::Out),
            ],
        )?;
        let mut structure = Structure::try_from(&streamlet)?;
        structure.try_add_streamlet_instance_default(
            db,
            "instance",
            streamlet.with_implementation(None).intern_arc(db),
        )?;
        structure.try_add_connection(db, "a", ("instance", "a"))?;

        Ok(())
    }

    #[test]
    fn try_validate_connections() -> Result<()> {
        let _db = Database::default();
        let db = &_db;
        let stream = test_stream_id(db, 4)?;
        let streamlet = Streamlet::new().try_with_name("a")?.with_ports(
            db,
            vec![
                ("a", stream, InterfaceDirection::In),
                ("b", stream, InterfaceDirection::Out),
            ],
        )?;

        let mut structure = Structure::try_from(&streamlet)?;
        structure.try_add_streamlet_instance_default(
            db,
            "instance",
            streamlet.with_implementation(None).intern_arc(db),
        )?;
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
        let streamlet = Streamlet::new().try_with_name("a")?.with_ports(
            db,
            vec![
                ("a", stream, InterfaceDirection::In),
                ("b", stream, InterfaceDirection::Out),
            ],
        )?;
        let mut structure = Structure::try_from(&streamlet)?;
        structure.try_add_streamlet_instance_default(
            db,
            "instance",
            streamlet.with_implementation(None).intern_arc(db),
        )?;
        structure.try_get_streamlet_instance(&("instance".try_result()?))?;

        Ok(())
    }
}
