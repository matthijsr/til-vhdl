use tydi_common::{
    error::{Error, Result, TryResult},
    name::{Name, PathName, PathNameSelf},
    traits::{Document, Identify},
};
use tydi_intern::Id;

use super::{
    implementation::ImplementationKind,
    project::interface::InterfaceCollection,
    traits::{GetSelf, InternSelf, MoveDb, TryIntern},
    Implementation, Interface, Ir,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Streamlet {
    /// Streamlet nodes should be prefixed by their containing namespace
    /// and can be suffixed with their implementation.
    name: PathName,
    /// If this streamlet is defined through a hard link or intrinsic, its name may need to be fixed.
    lock_name: bool,
    implementation: Option<Id<Implementation>>,
    interface: Option<Id<InterfaceCollection>>,
    doc: Option<String>,
}

impl Streamlet {
    pub fn new() -> Self {
        Streamlet {
            name: PathName::new_empty(),
            implementation: None,
            lock_name: false,
            interface: None,
            doc: None,
        }
    }

    pub fn with_ports(
        mut self,
        db: &dyn Ir,
        ports: Vec<impl TryResult<Interface>>,
    ) -> Result<Streamlet> {
        let interface = InterfaceCollection::new(db, ports)?.intern(db);
        self.interface = Some(interface);
        Ok(self)
    }

    pub fn with_interface_collection(
        mut self,
        db: &dyn Ir,
        coll: impl TryIntern<InterfaceCollection>,
    ) -> Result<Streamlet> {
        self.interface = Some(coll.try_intern(db)?);

        Ok(self)
    }

    pub fn with_name(mut self, name: impl Into<PathName>) -> Self {
        if !self.name_is_locked() {
            self.name = name.into();
        }
        self
    }

    pub fn with_doc(mut self, doc: impl Into<String>) -> Self {
        self.doc = Some(doc.into());
        self
    }

    pub fn set_doc(&mut self, doc: impl Into<String>) {
        self.doc = Some(doc.into());
    }

    pub fn try_with_name(mut self, name: impl TryResult<PathName>) -> Result<Self> {
        self = self.with_name(name.try_result()?);
        Ok(self)
    }

    pub fn with_implementation(
        mut self,
        db: &dyn Ir,
        implementation: Option<Implementation>,
    ) -> Streamlet {
        if let Some(implementation) = implementation {
            if let ImplementationKind::Link(link) = implementation.kind() {
                self = self.with_name(link.path_name());
                self.lock_name = true;
            }
            self.implementation = Some(implementation.intern(db));
        } else {
            self.implementation = None;
            self.lock_name = false;
        }
        self
    }

    /// Indicate whether the name of this streamlet is locked. (try_with_name will still return OK, but will not )
    pub fn name_is_locked(&self) -> bool {
        self.lock_name
    }

    pub fn implementation(&self, db: &dyn Ir) -> Option<Implementation> {
        if let Some(id) = self.implementation {
            Some(db.lookup_intern_implementation(id))
        } else {
            None
        }
    }

    pub fn interface_id(&self) -> Option<Id<InterfaceCollection>> {
        self.interface.clone()
    }

    pub fn interface(&self, db: &dyn Ir) -> InterfaceCollection {
        if let Some(interface_id) = self.interface_id() {
            interface_id.get(db)
        } else {
            let interface = InterfaceCollection::new_empty();
            interface.clone().intern(db);
            interface
        }
    }

    pub fn ports(&self, db: &dyn Ir) -> Vec<Interface> {
        self.interface(db).ports(db)
    }

    pub fn inputs(&self, db: &dyn Ir) -> Vec<Interface> {
        self.interface(db).inputs(db)
    }

    pub fn outputs(&self, db: &dyn Ir) -> Vec<Interface> {
        self.interface(db).outputs(db)
    }

    pub fn try_get_port(&self, db: &dyn Ir, name: &Name) -> Result<Interface> {
        match self.interface(db).try_get_port(db, name) {
            Ok(port) => Ok(port),
            Err(_) => Err(Error::InvalidArgument(format!(
                "No port with name {} exists on Streamlet {}",
                name,
                self.identifier()
            ))),
        }
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
    fn doc(&self) -> Option<String> {
        self.doc.clone()
    }
}

impl MoveDb<Id<Streamlet>> for Streamlet {
    fn move_db(
        &self,
        original_db: &dyn Ir,
        target_db: &dyn Ir,
        prefix: &Option<Name>,
    ) -> Result<Id<Streamlet>> {
        let interface = match &self.interface {
            Some(id) => Some(id.move_db(original_db, target_db, prefix)?),
            None => None,
        };
        let implementation = match &self.implementation {
            Some(id) => Some(id.move_db(original_db, target_db, prefix)?),
            None => None,
        };
        Ok(Streamlet {
            name: self.name.with_parents(prefix),
            implementation,
            lock_name: self.lock_name,
            interface,
            doc: self.doc.clone(),
        }
        .intern(target_db))
    }
}

impl From<Id<InterfaceCollection>> for Streamlet {
    fn from(id: Id<InterfaceCollection>) -> Self {
        Streamlet {
            name: PathName::new_empty(),
            implementation: None,
            lock_name: false,
            interface: Some(id),
            doc: None,
        }
    }
}
