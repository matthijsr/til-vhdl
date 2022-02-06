use std::collections::BTreeMap;

use tydi_common::{
    error::{Error, Result, TryResult},
    name::Name,
};
use tydi_intern::Id;

use crate::ir::{
    interface::Interface,
    traits::{InternSelf, MoveDb},
    Ir,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InterfaceCollection {
    ports: BTreeMap<Name, Id<Interface>>,
    port_order: Vec<Name>,
}

impl InterfaceCollection {
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
}

impl Into<BTreeMap<Name, Id<Interface>>> for InterfaceCollection {
    fn into(self) -> BTreeMap<Name, Id<Interface>> {
        self.ports
    }
}

impl MoveDb<InterfaceCollection> for InterfaceCollection {
    fn move_db(
        &self,
        original_db: &dyn Ir,
        target_db: &dyn Ir,
        prefix: &Option<Name>,
    ) -> Result<InterfaceCollection> {
        let port_order = self.port_order.clone();
        let ports = self
            .ports
            .iter()
            .map(|(k, v)| Ok((k.clone(), v.move_db(original_db, target_db, prefix)?)))
            .collect::<Result<_>>()?;
        Ok(InterfaceCollection { ports, port_order })
    }
}
