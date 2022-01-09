use std::{collections::BTreeMap, convert::TryInto};

use tydi_common::{
    error::{Error, Result, TryResult},
    name::Name,
};
use tydi_intern::Id;

use super::{
    connection::InterfaceReference, physical_properties::InterfaceDirection, Connection, GetSelf,
    Implementation, Interface, Ir, Streamlet,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Context {
    ports: Vec<Id<Interface>>,
    streamlet_instances: BTreeMap<Name, Id<Streamlet>>,
    connections: Vec<Connection>,
    implementations: BTreeMap<Name, Id<Implementation>>,
}

impl Context {
    pub fn new(ports: Vec<Id<Interface>>) -> Self {
        Context {
            ports,
            streamlet_instances: BTreeMap::new(),
            connections: vec![],
            implementations: BTreeMap::new(),
        }
    }

    pub fn port_ids(&self) -> &Vec<Id<Interface>> {
        &self.ports
    }

    pub fn ports(&self, db: &dyn Ir) -> Vec<Interface> {
        self.port_ids().iter().map(|x| x.get(db)).collect()
    }

    pub fn streamlet_instances(&self) -> &BTreeMap<Name, Id<Streamlet>> {
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
        let left_streamlet = self.try_get_streamlet_instance(left.streamlet_instance().clone())?;
        let right_streamlet =
            self.try_get_streamlet_instance(right.streamlet_instance().clone())?;
        let left_interface = left_streamlet
            .get(db)
            .try_get_port(db, left.port().clone())?;
        let right_interface = right_streamlet
            .get(db)
            .try_get_port(db, right.port().clone())?;
        if left_interface.is_compatible(&right_interface) {
            let (source, sink) = match left_interface.physical_properties().direction() {
                InterfaceDirection::Out => (left, right),
                InterfaceDirection::In => (right, left),
            };
            self.connections.push(Connection::new(source, sink));
            Ok(())
        } else {
            Err(Error::InvalidTarget(format!(
                "The ports {}.{} and {}.{} are incompatible",
                left.streamlet_instance(),
                left.port(),
                right.streamlet_instance(),
                right.port()
            )))
        }
    }

    pub fn try_add_streamlet_instance(
        &mut self,
        name: impl TryResult<Name>,
        streamlet: Id<Streamlet>,
    ) -> Result<()> {
        let name = name.try_result()?;
        if self.streamlet_instances().contains_key(&name) {
            Err(Error::InvalidArgument(format!(
                "A streamlet instance with name {} already exists in this context",
                name
            )))
        } else {
            self.streamlet_instances.insert(name, streamlet);
            Ok(())
        }
    }

    pub fn try_get_streamlet_instance(&self, name: impl TryResult<Name>) -> Result<Id<Streamlet>> {
        let name = name.try_result()?;
        match self.streamlet_instances().get(&name) {
            Some(streamlet) => Ok(*streamlet),
            None => Err(Error::InvalidArgument(format!(
                "A streamlet instance with name {} does not exist in this context",
                name
            ))),
        }
    }
}
