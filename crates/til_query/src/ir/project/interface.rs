use core::fmt;
use std::{
    convert::{TryFrom, TryInto},
    sync::Arc,
};

use tydi_common::{
    error::{Error, Result, TryResult},
    map::{InsertionOrderedMap, InsertionOrderedSet},
    name::{Name, NameSelf},
    traits::Identify,
};
use tydi_intern::Id;

use crate::ir::{
    generics::GenericParameter,
    implementation::structure::Structure,
    interface_port::InterfacePort,
    physical_properties::{Domain, InterfaceDirection},
    streamlet::Streamlet,
    traits::{InternSelf, MoveDb},
    Ir,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Interface {
    domains: Option<InsertionOrderedSet<Domain>>,
    parameters: InsertionOrderedMap<Name, GenericParameter>,
    ports: InsertionOrderedMap<Name, InterfacePort>,
}

impl Interface {
    pub fn new_empty() -> Self {
        Interface {
            domains: None,
            parameters: InsertionOrderedMap::new(),
            ports: InsertionOrderedMap::new(),
        }
    }

    pub fn new(
        domains: impl IntoIterator<Item = impl TryResult<Name>>,
        ports: impl IntoIterator<Item = impl TryResult<InterfacePort>>,
    ) -> Result<Self> {
        Self::new_domains(domains)?.with_ports(ports)
    }

    pub fn new_parameters(
        parameters: impl IntoIterator<Item = impl TryResult<GenericParameter>>,
    ) -> Result<Self> {
        Self::new_empty().with_parameters(parameters)
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
            parameters: InsertionOrderedMap::new(),
            ports: InsertionOrderedMap::new(),
        })
    }

    pub fn new_ports(
        ports: impl IntoIterator<Item = impl TryResult<InterfacePort>>,
    ) -> Result<Self> {
        Self::new_empty().with_ports(ports)
    }

    fn verify_port(&self, port: &InterfacePort) -> Result<()> {
        if let Some(domains) = self.domains() {
            let domains_string = domains
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join(", ");

            if let Some(domain) = port.domain() {
                if domains.contains(domain) {
                    Ok(())
                } else {
                    Err(Error::InvalidArgument(format!(
                        "Port {} has domain {}, but Interface has domains: {}",
                        port.name(),
                        domain,
                        domains_string
                    )))
                }
            } else {
                Err(Error::InvalidArgument(format!(
                    "Port {} has Default domain, but Interface has domains: {}",
                    port.name(),
                    domains_string
                )))
            }
        } else {
            if let Some(domain) = port.domain() {
                Err(Error::InvalidArgument(format!(
                    "Port {} has domain {}, but Interface has Default domain",
                    port.name(),
                    domain
                )))
            } else {
                Ok(())
            }
        }
    }

    pub fn with_ports(
        mut self,
        ports: impl IntoIterator<Item = impl TryResult<InterfacePort>>,
    ) -> Result<Self> {
        let mut port_map = InsertionOrderedMap::new();
        for port in ports {
            let port = port.try_result()?;
            self.verify_port(&port)?;
            port_map.try_insert(port.name().clone(), port)?
        }

        self.ports = port_map;
        Ok(self)
    }

    pub fn with_parameters(
        mut self,
        parameters: impl IntoIterator<Item = impl TryResult<GenericParameter>>,
    ) -> Result<Self> {
        let mut param_map = InsertionOrderedMap::new();
        for param in parameters {
            let param = param.try_result()?;
            param_map.try_insert(param.name().clone(), param)?
        }

        self.parameters = param_map;
        Ok(self)
    }

    pub fn push_port(&mut self, port: impl TryResult<InterfacePort>) -> Result<()> {
        let port = port.try_result()?;
        self.verify_port(&port)?;
        self.ports.try_insert(port.name().clone(), port)
    }

    pub fn ports(&self) -> &InsertionOrderedMap<Name, InterfacePort> {
        &self.ports
    }

    pub fn inputs(&self) -> impl Iterator<Item = &InterfacePort> {
        self.ports()
            .into_iter()
            .map(|(_, x)| x)
            .filter(|x| x.physical_properties().direction() == InterfaceDirection::In)
    }

    pub fn outputs(&self) -> impl Iterator<Item = &InterfacePort> {
        self.ports()
            .into_iter()
            .map(|(_, x)| x)
            .filter(|x| x.physical_properties().direction() == InterfaceDirection::Out)
    }

    pub fn try_get_port(&self, name: &Name) -> Result<InterfacePort> {
        match self.ports().get(name) {
            Some(port) => Ok(port.clone()),
            None => Err(Error::InvalidArgument(format!(
                "No port with name {} exists on this interface",
                name
            ))),
        }
    }

    /// When `None`, this Interface only has a Default domain.
    pub fn domains(&self) -> &Option<InsertionOrderedSet<Domain>> {
        &self.domains
    }
}

impl MoveDb<Id<Arc<Interface>>> for Arc<Interface> {
    fn move_db(
        &self,
        _original_db: &dyn Ir,
        target_db: &dyn Ir,
        _prefix: &Option<Name>,
    ) -> Result<Id<Arc<Interface>>> {
        Ok(self.clone().intern(target_db))
    }
}

impl From<Structure> for Id<Arc<Interface>> {
    fn from(st: Structure) -> Self {
        st.interface_id()
    }
}

impl From<&Structure> for Id<Arc<Interface>> {
    fn from(st: &Structure) -> Self {
        st.interface_id()
    }
}

impl TryFrom<Streamlet> for Id<Arc<Interface>> {
    type Error = Error;

    fn try_from(streamlet: Streamlet) -> Result<Self> {
        (&streamlet).try_into()
    }
}

impl TryFrom<&Streamlet> for Id<Arc<Interface>> {
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
            .ports()
            .iter()
            .map(|(name, port)| format!("{}: {}", name, port))
            .collect::<Vec<String>>()
            .join(", ");
        write!(f, "InterfaceCollection({})", fields)
    }
}
