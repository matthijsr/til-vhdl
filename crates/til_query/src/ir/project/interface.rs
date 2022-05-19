use core::fmt;
use std::convert::{TryFrom, TryInto};

use tydi_common::{
    error::{Error, Result, TryResult},
    map::{InsertionOrderedMap, InsertionOrderedSet},
    name::Name,
    traits::Identify,
};
use tydi_intern::Id;

use crate::ir::{
    implementation::structure::Structure,
    interface_port::InterfacePort,
    physical_properties::InterfaceDirection,
    streamlet::Streamlet,
    traits::{GetSelf, InternSelf, MoveDb},
    Ir,
};

use super::domain::Domain;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Interface {
    domains: Option<InsertionOrderedSet<Domain>>,
    ports: InsertionOrderedMap<Name, Id<InterfacePort>>,
}

impl Interface {
    pub fn new_empty() -> Self {
        Interface {
            domains: None,
            ports: InsertionOrderedMap::new(),
        }
    }

    pub fn new(
        db: &dyn Ir,
        domains: impl IntoIterator<Item = impl TryResult<Name>>,
        ports: impl IntoIterator<Item = impl TryResult<InterfacePort>>,
    ) -> Result<Self> {
        let mut domain_set = InsertionOrderedSet::new();
        for domain in domains {
            let domain = domain.try_result()?;
            domain_set.try_insert(domain)?;
        }

        let mut port_map = InsertionOrderedMap::new();
        for port in ports {
            let port = port.try_result()?;
            port_map.try_insert(port.name().clone(), port.intern(db))?
        }

        let domain_set = if domain_set.len() > 0 {
            Some(domain_set)
        } else {
            None
        };

        Ok(Interface {
            domains: domain_set,
            ports: port_map,
        })
    }

    pub fn new_domains(domains: impl IntoIterator<Item = impl TryResult<Name>>) -> Result<Self> {
        let mut domain_set = InsertionOrderedSet::new();
        for domain in domains {
            let domain = domain.try_result()?;
            domain_set.try_insert(domain)?;
        }

        let domain_set = if domain_set.len() > 0 {
            Some(domain_set)
        } else {
            None
        };

        Ok(Interface {
            domains: domain_set,
            ports: InsertionOrderedMap::new(),
        })
    }

    pub fn new_ports(
        db: &dyn Ir,
        ports: impl IntoIterator<Item = impl TryResult<InterfacePort>>,
    ) -> Result<Self> {
        let mut port_map = InsertionOrderedMap::new();
        for port in ports {
            let port = port.try_result()?;
            port_map.try_insert(port.name().clone(), port.intern(db))?
        }

        Ok(Interface {
            domains: None,
            ports: port_map,
        })
    }

    pub fn with_domains(
        mut self,
        domains: impl IntoIterator<Item = impl TryResult<Name>>,
    ) -> Result<Self> {
        let mut domain_set = InsertionOrderedSet::new();
        for domain in domains {
            let domain = domain.try_result()?;
            domain_set.try_insert(domain)?;
        }

        let domain_set = if domain_set.len() > 0 {
            Some(domain_set)
        } else {
            None
        };

        self.domains = domain_set;
        Ok(self)
    }

    pub fn with_ports(
        mut self,
        db: &dyn Ir,
        ports: impl IntoIterator<Item = impl TryResult<InterfacePort>>,
    ) -> Result<Self> {
        let mut port_map = InsertionOrderedMap::new();
        for port in ports {
            let port = port.try_result()?;
            port_map.try_insert(port.name().clone(), port.intern(db))?
        }

        self.ports = port_map;
        Ok(self)
    }

    pub fn push_domain(&mut self, domain: impl TryResult<Name>) -> Result<()> {
        let domain = domain.try_result()?;
        if let Some(domains) = &mut self.domains {
            domains.try_insert(domain)
        } else {
            let mut domains = InsertionOrderedSet::new();
            domains.try_insert(domain)?;
            self.domains = Some(domains);
            Ok(())
        }
    }

    pub fn push_port(&mut self, db: &dyn Ir, port: impl TryResult<InterfacePort>) -> Result<()> {
        let port = port.try_result()?;
        self.ports.try_insert(port.name().clone(), port.intern(db))
    }

    pub fn port_ids(&self) -> &InsertionOrderedMap<Name, Id<InterfacePort>> {
        &self.ports
    }

    pub fn ports(&self, db: &dyn Ir) -> Vec<InterfacePort> {
        let mut result = vec![];
        for (_, port) in &self.ports {
            result.push(port.get(db));
        }
        result
    }

    pub fn inputs(&self, db: &dyn Ir) -> Vec<InterfacePort> {
        self.ports(db)
            .into_iter()
            .filter(|x| x.physical_properties().direction() == InterfaceDirection::In)
            .collect()
    }

    pub fn outputs(&self, db: &dyn Ir) -> Vec<InterfacePort> {
        self.ports(db)
            .into_iter()
            .filter(|x| x.physical_properties().direction() == InterfaceDirection::Out)
            .collect()
    }

    pub fn try_get_port(&self, db: &dyn Ir, name: &Name) -> Result<InterfacePort> {
        match self.port_ids().get(name) {
            Some(port) => Ok(port.get(db)),
            None => Err(Error::InvalidArgument(format!(
                "No port with name {} exists on this interface collection",
                name
            ))),
        }
    }

    /// When `None`, this Interface only has a Default domain.
    pub fn domains(&self) -> &Option<InsertionOrderedSet<Domain>> {
        &self.domains
    }
}

impl MoveDb<Id<Interface>> for Interface {
    fn move_db(
        &self,
        original_db: &dyn Ir,
        target_db: &dyn Ir,
        prefix: &Option<Name>,
    ) -> Result<Id<Interface>> {
        let mut ports = InsertionOrderedMap::new();
        for (name, port) in self.port_ids() {
            ports.try_insert(name.clone(), port.move_db(original_db, target_db, prefix)?)?;
        }
        Ok(Interface {
            domains: self.domains.clone(),
            ports,
        }
        .intern(target_db))
    }
}

impl From<Structure> for Id<Interface> {
    fn from(st: Structure) -> Self {
        st.interface_id()
    }
}

impl From<&Structure> for Id<Interface> {
    fn from(st: &Structure) -> Self {
        st.interface_id()
    }
}

impl TryFrom<Streamlet> for Id<Interface> {
    type Error = Error;

    fn try_from(streamlet: Streamlet) -> Result<Self> {
        (&streamlet).try_into()
    }
}

impl TryFrom<&Streamlet> for Id<Interface> {
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

impl fmt::Display for Interface {
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
