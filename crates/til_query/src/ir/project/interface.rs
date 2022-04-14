use core::fmt;
use std::convert::{TryFrom, TryInto};

use tydi_common::{
    error::{Error, Result, TryResult},
    map::InsertionOrderedMap,
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
    ports: InsertionOrderedMap<Name, Id<Interface>>,
}

impl InterfaceCollection {
    pub fn new_empty() -> Self {
        InterfaceCollection {
            ports: InsertionOrderedMap::new(),
        }
    }

    pub fn new(db: &dyn Ir, ports: Vec<impl TryResult<Interface>>) -> Result<Self> {
        let mut port_map = InsertionOrderedMap::new();
        for port in ports {
            let port = port.try_result()?;
            port_map.try_insert(port.name().clone(), port.intern(db))?
        }

        Ok(InterfaceCollection { ports: port_map })
    }

    pub fn push(&mut self, db: &dyn Ir, port: impl TryResult<Interface>) -> Result<()> {
        let port = port.try_result()?;
        self.ports.try_insert(port.name().clone(), port.intern(db))
    }

    pub fn port_ids(&self) -> &InsertionOrderedMap<Name, Id<Interface>> {
        &self.ports
    }

    pub fn ports(&self, db: &dyn Ir) -> Vec<Interface> {
        let mut result = vec![];
        for (_, port) in &self.ports {
            result.push(port.get(db));
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

impl MoveDb<Id<InterfaceCollection>> for InterfaceCollection {
    fn move_db(
        &self,
        original_db: &dyn Ir,
        target_db: &dyn Ir,
        prefix: &Option<Name>,
    ) -> Result<Id<InterfaceCollection>> {
        let mut ports = InsertionOrderedMap::new();
        for (name, port) in self.port_ids() {
            ports.try_insert(name.clone(), port.move_db(original_db, target_db, prefix)?)?;
        }
        Ok(InterfaceCollection { ports }.intern(target_db))
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
            .port_ids()
            .iter()
            .map(|(name, id)| format!("{}: Id({})", name, id))
            .collect::<Vec<String>>()
            .join(", ");
        write!(f, "InterfaceCollection({})", fields)
    }
}
