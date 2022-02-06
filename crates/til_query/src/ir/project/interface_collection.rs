use std::{
    collections::BTreeMap,
    convert::{TryFrom, TryInto},
};

use tydi_common::{
    error::{Error, Result, TryResult},
    name::Name,
    traits::Identify,
};
use tydi_intern::Id;

use crate::ir::{
    implementation::{structure::Structure, Implementation},
    interface::Interface,
    physical_properties::InterfaceDirection,
    streamlet::Streamlet,
    traits::{GetSelf, InternSelf, MoveDb},
    Ir,
};

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

    pub fn port_ids(&self) -> &BTreeMap<Name, Id<Interface>> {
        &self.ports
    }

    pub fn ordered_port_ids(&self) -> (&BTreeMap<Name, Id<Interface>>, &Vec<Name>) {
        (&self.ports, &self.port_order)
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

impl From<Implementation> for Id<InterfaceCollection> {
    fn from(imp: Implementation) -> Self {
        imp.interface_id()
    }
}

impl From<&Implementation> for Id<InterfaceCollection> {
    fn from(imp: &Implementation) -> Self {
        imp.interface_id()
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
