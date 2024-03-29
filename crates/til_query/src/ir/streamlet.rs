use std::sync::Arc;

use tydi_common::{
    error::{Error, Result, TryResult},
    map::{InsertionOrderedMap, InsertionOrderedSet},
    name::{Name, PathName, PathNameSelf},
    traits::{Document, Documents, Identify},
};
use tydi_intern::Id;

use super::{
    generics::GenericParameter,
    physical_properties::Domain,
    project::interface::Interface,
    traits::{GetSelf, InternArc, InternSelf, MoveDb, TryIntern},
    Implementation, InterfacePort, Ir,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Streamlet {
    /// Streamlet nodes should be prefixed by their containing namespace
    /// and can be suffixed with their implementation.
    name: PathName,
    implementation: Option<Id<Implementation>>,
    interface: Option<Id<Arc<Interface>>>,
    doc: Option<String>,
}

impl Streamlet {
    pub fn new() -> Self {
        Streamlet {
            name: PathName::new_empty(),
            implementation: None,
            interface: None,
            doc: None,
        }
    }

    pub fn with_domains_ports(
        mut self,
        db: &dyn Ir,
        domains: impl IntoIterator<Item = impl TryResult<Name>>,
        ports: impl IntoIterator<Item = impl TryResult<InterfacePort>>,
    ) -> Result<Streamlet> {
        let interface = Interface::new_ports_domains(db, domains, ports)?;
        self.interface = Some(interface);

        Ok(self)
    }

    pub fn with_ports(
        mut self,
        db: &dyn Ir,
        ports: impl IntoIterator<Item = impl TryResult<InterfacePort>>,
    ) -> Result<Streamlet> {
        if let Some(interface) = self.interface {
            let new_interface = interface.get(db).as_ref().clone().with_ports(db, ports)?;
            self.interface = Some(new_interface);
        } else {
            let interface = Interface::new_ports(db, ports)?;
            self.interface = Some(interface);
        }

        Ok(self)
    }

    pub fn with_parameters(
        mut self,
        db: &dyn Ir,
        parameters: impl IntoIterator<Item = impl TryResult<GenericParameter>>,
    ) -> Result<Streamlet> {
        if let Some(interface) = self.interface {
            let new_interface = interface
                .get(db)
                .as_ref()
                .clone()
                .with_parameters(parameters)?;
            self.interface = Some(new_interface.intern_arc(db));
        } else {
            let interface = Interface::new_parameters(parameters)?.intern_arc(db);
            self.interface = Some(interface);
        }

        Ok(self)
    }

    pub fn with_interface(
        mut self,
        db: &dyn Ir,
        coll: impl TryIntern<Arc<Interface>>,
    ) -> Result<Streamlet> {
        self.interface = Some(coll.try_intern(db)?);

        Ok(self)
    }

    pub fn with_name(mut self, name: impl Into<PathName>) -> Self {
        self.name = name.into();
        self
    }

    pub fn try_with_name(mut self, name: impl TryResult<PathName>) -> Result<Self> {
        self.name = name.try_result()?;
        Ok(self)
    }

    pub fn with_implementation(mut self, implementation: Option<Id<Implementation>>) -> Streamlet {
        self.implementation = implementation;
        self
    }

    pub fn implementation_id(&self) -> Option<Id<Implementation>> {
        self.implementation
    }

    pub fn implementation(&self, db: &dyn Ir) -> Option<Implementation> {
        if let Some(id) = self.implementation {
            Some(db.lookup_intern_implementation(id))
        } else {
            None
        }
    }

    pub fn interface_id(&self) -> Option<Id<Arc<Interface>>> {
        self.interface.clone()
    }

    pub fn interface(&self, db: &dyn Ir) -> Arc<Interface> {
        if let Some(interface_id) = self.interface_id() {
            interface_id.get(db)
        } else {
            let interface = Arc::new(Interface::new_empty());
            interface.clone().intern(db);
            interface
        }
    }

    pub fn ports(&self, db: &dyn Ir) -> InsertionOrderedMap<Name, InterfacePort> {
        self.interface(db).ports().clone()
    }

    pub fn inputs(&self, db: &dyn Ir) -> Vec<InterfacePort> {
        self.interface(db).inputs().cloned().collect()
    }

    pub fn outputs(&self, db: &dyn Ir) -> Vec<InterfacePort> {
        self.interface(db).outputs().cloned().collect()
    }

    pub fn try_get_port(&self, db: &dyn Ir, name: &Name) -> Result<InterfacePort> {
        match self.interface(db).try_get_port(name) {
            Ok(port) => Ok(port),
            Err(_) => Err(Error::InvalidArgument(format!(
                "No port with name {} exists on Streamlet {}",
                name,
                self.identifier()
            ))),
        }
    }

    pub fn try_get_parameter(&self, db: &dyn Ir, name: &Name) -> Result<GenericParameter> {
        match self.interface(db).try_get_parameter(name) {
            Ok(param) => Ok(param),
            Err(_) => Err(Error::InvalidArgument(format!(
                "No parameter with name {} exists on Streamlet {}",
                name,
                self.identifier()
            ))),
        }
    }

    pub fn domains(&self, db: &dyn Ir) -> Option<InsertionOrderedSet<Domain>> {
        self.interface(db).domains().clone()
    }

    pub fn parameters(&self, db: &dyn Ir) -> InsertionOrderedMap<Name, GenericParameter> {
        self.interface(db).parameters().clone()
    }
}

impl PathNameSelf for Streamlet {
    fn path_name(&self) -> &PathName {
        &self.name
    }
}

impl Identify for Streamlet {
    fn identifier(&self) -> String {
        self.path_name().to_string()
    }
}

impl Document for Streamlet {
    fn doc(&self) -> Option<&String> {
        self.doc.as_ref()
    }
}

impl Documents for Streamlet {
    fn set_doc(&mut self, doc: impl Into<String>) {
        self.doc = Some(doc.into());
    }
}

impl MoveDb<Id<Arc<Streamlet>>> for Arc<Streamlet> {
    fn move_db(
        &self,
        original_db: &dyn Ir,
        target_db: &dyn Ir,
        prefix: &Option<Name>,
    ) -> Result<Id<Arc<Streamlet>>> {
        let interface = match &self.interface {
            Some(id) => Some(id.move_db(original_db, target_db, prefix)?),
            None => None,
        };
        let implementation = match &self.implementation {
            Some(id) => Some(id.move_db(original_db, target_db, prefix)?),
            None => None,
        };
        Ok(Arc::new(Streamlet {
            name: self.name.clone().with_parents(prefix),
            implementation,
            interface,
            doc: self.doc.clone(),
        })
        .intern(target_db))
    }
}

impl From<Id<Arc<Interface>>> for Streamlet {
    fn from(id: Id<Arc<Interface>>) -> Self {
        Streamlet {
            name: PathName::new_empty(),
            implementation: None,
            interface: Some(id),
            doc: None,
        }
    }
}
