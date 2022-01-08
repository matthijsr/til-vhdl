use std::{collections::BTreeMap, convert::TryInto};

use tydi_common::{
    error::{Error, Result},
    name::Name,
};
use tydi_intern::Id;

use super::{Connection, GetSelf, Implementation, Interface, Ir, Streamlet, connection::InterfaceReference};

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

    pub fn try_add_connection(&mut self, db: &dyn Ir, source: impl TryInto<InterfaceReference, Error = Error>, sink: impl TryInto<InterfaceReference, Error = Error>) -> Result<()> {
        let source = source.try_into()?;
        let sink = sink.try_into()?;
        let source_streamlet = self.try_get_streamlet_instance(source.streamlet_instance().clone())?;
        Ok(())
    }

    pub fn try_add_streamlet_instance(
        &mut self,
        name: impl TryInto<Name, Error = Error>,
        streamlet: Id<Streamlet>,
    ) -> Result<()> {
        let name = name.try_into()?;
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

    pub fn try_get_streamlet_instance(
        &self,
        name: impl TryInto<Name, Error = Error>,
    ) -> Result<Id<Streamlet>> {
        let name = name.try_into()?;
        match self.streamlet_instances().get(&name) {
            Some(streamlet) => Ok(*streamlet),
            None => Err(Error::InvalidArgument(format!(
                "A streamlet instance with name {} does not exist in this context",
                name
            ))),
        }
    }
}
