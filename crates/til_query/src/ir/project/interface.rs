use core::fmt;
use std::{
    collections::BTreeMap,
    convert::{TryFrom, TryInto},
};

use indexmap::IndexMap;
use tydi_common::{
    error::{Error, Result, TryResult},
    name::Name,
    traits::Identify,
};
use tydi_intern::Id;

use crate::ir::{
    implementation::structure::Structure,
    interface::Interface,
    physical_properties::InterfaceDirection,
    streamlet::Streamlet,
    traits::{GetSelf, InternSelf, MoveDb},
    Ir,
};

// TODO: Can probably replace this with an InsertionOrderedMap
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InterfaceCollection {
    ports: BTreeMap<Name, Id<Interface>>,
    port_order: Vec<Name>,
}

impl InterfaceCollection {
    pub fn new_empty() -> Self {
        InterfaceCollection {
            ports: BTreeMap::new(),
            port_order: vec![],
        }
    }

    pub fn new(db: &dyn Ir, ports: Vec<impl TryResult<Interface>>) -> Result<Self> {
        let mut port_order = vec![];
        let mut port_map = BTreeMap::new();
        for port in ports {
            let port = port.try_result()?;
            port_order.push(port.name().clone());
            if port_map.insert(port.name().clone(), port.intern(db)) != None {
                return Err(Error::UnexpectedDuplicate);
            }
        }

        Ok(InterfaceCollection {
            ports: port_map,
            port_order,
        })
    }

    pub fn push(&mut self, db: &dyn Ir, port: impl TryResult<Interface>) -> Result<()> {
        let port = port.try_result()?;
        let port_name = port.name().clone();
        if let Some(_) = self.ports.insert(port.name().clone(), port.intern(db)) {
            Err(Error::InvalidArgument(format!(
                "A port with name {} already exists on this interface.",
                port_name
            )))
        } else {
            self.port_order.push(port_name);
            Ok(())
        }
    }

    pub fn port_ids(&self) -> &BTreeMap<Name, Id<Interface>> {
        &self.ports
    }

    pub fn ordered_port_ids(&self) -> IndexMap<Name, Id<Interface>> {
        let mut result = IndexMap::new();
        for name in self.port_order.iter() {
            result.insert(name.clone(), self.port_ids().get(name).unwrap().clone());
        }
        result
    }

    pub fn ports(&self, db: &dyn Ir) -> Vec<Interface> {
        let mut result = vec![];
        for name in &self.port_order {
            result.push(self.port_ids().get(name).unwrap().get(db));
        }
        result
    }

    pub fn inputs(&self, db: &dyn Ir) -> Vec<Interface> {
        self.ports(db)
            .into_iter()
            .filter(|x| x.physical_properties().direction() == InterfaceDirection::In)
            .collect()
    }

    pub fn outputs(&self, db: &dyn Ir) -> Vec<Interface> {
        self.ports(db)
            .into_iter()
            .filter(|x| x.physical_properties().direction() == InterfaceDirection::Out)
            .collect()
    }

    pub fn try_get_port(&self, db: &dyn Ir, name: &Name) -> Result<Interface> {
        match self.port_ids().get(name) {
            Some(port) => Ok(port.get(db)),
            None => Err(Error::InvalidArgument(format!(
                "No port with name {} exists on this interface collection",
                name
            ))),
        }
    }
}

impl Into<BTreeMap<Name, Id<Interface>>> for InterfaceCollection {
    fn into(self) -> BTreeMap<Name, Id<Interface>> {
        self.ports
    }
}

impl MoveDb<Id<InterfaceCollection>> for InterfaceCollection {
    fn move_db(
        &self,
        original_db: &dyn Ir,
        target_db: &dyn Ir,
        prefix: &Option<Name>,
    ) -> Result<Id<InterfaceCollection>> {
        let port_order = self.port_order.clone();
        let ports = self
            .ports
            .iter()
            .map(|(k, v)| Ok((k.clone(), v.move_db(original_db, target_db, prefix)?)))
            .collect::<Result<_>>()?;
        Ok(InterfaceCollection { ports, port_order }.intern(target_db))
    }
}

impl From<Structure> for Id<InterfaceCollection> {
    fn from(st: Structure) -> Self {
        st.interface_id()
    }
}

impl From<&Structure> for Id<InterfaceCollection> {
    fn from(st: &Structure) -> Self {
        st.interface_id()
    }
}

impl TryFrom<Streamlet> for Id<InterfaceCollection> {
    type Error = Error;

    fn try_from(streamlet: Streamlet) -> Result<Self> {
        (&streamlet).try_into()
    }
}

impl TryFrom<&Streamlet> for Id<InterfaceCollection> {
    type Error = Error;

    fn try_from(streamlet: &Streamlet) -> Result<Self> {
        if let Some(interface_id) = streamlet.interface_id() {
            Ok(interface_id)
        } else {
            Err(Error::BackEndError(format!(
                "Streamlet {} does not have an interface collection",
                streamlet.identifier()
            )))
        }
    }
}

impl fmt::Display for InterfaceCollection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = self
            .ordered_port_ids()
            .iter()
            .map(|(name, id)| format!("{}: Id({})", name, id))
            .collect::<Vec<String>>()
            .join(", ");
        write!(f, "InterfaceCollection({})", fields)
    }
}
