pub mod structure;

use tydi_common::{
    error::{Result, TryResult},
    name::{Name, PathName, PathNameSelf},
    traits::{Document, Identify},
};
use tydi_intern::Id;

use self::structure::Structure;

use super::{
    project::interface::InterfaceCollection,
    traits::{GetSelf, InternSelf, MoveDb},
    Ir,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Implementation {
    name: PathName,
    interface: Id<InterfaceCollection>,
    kind: ImplementationKind,
    doc: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ImplementationKind {
    Structural(Structure),
    Link,
}

impl Implementation {
    pub fn structural(structure: impl TryResult<Structure>) -> Result<Self> {
        let structure = structure.try_result()?;
        Ok(Implementation {
            name: PathName::new_empty(),
            interface: structure.interface_id(),
            kind: ImplementationKind::Structural(structure.try_result()?),
            doc: None,
        })
    }

    /// TODO
    // pub fn link() -> Self {
    //     Implementation {
    //         name: PathName::new_empty(),
    //         kind: ImplementationKind::Link,
    //     }
    // }

    pub fn set_doc(&mut self, doc: impl Into<String>) {
        self.doc = Some(doc.into())
    }

    pub fn with_doc(mut self, doc: impl Into<String>) -> Self {
        self.doc = Some(doc.into());
        self
    }

    pub fn with_name(mut self, name: impl Into<PathName>) -> Self {
        self.name = name.into();
        self
    }

    pub fn try_with_name(mut self, name: impl TryResult<PathName>) -> Result<Self> {
        self.name = name.try_result()?;
        Ok(self)
    }

    pub fn kind(&self) -> &ImplementationKind {
        &self.kind
    }

    pub fn interface_id(&self) -> Id<InterfaceCollection> {
        self.interface
    }

    pub fn interface(&self, db: &dyn Ir) -> InterfaceCollection {
        self.interface_id().get(db)
    }
}

impl From<Structure> for Implementation {
    fn from(value: Structure) -> Self {
        Implementation {
            name: PathName::new_empty(),
            interface: value.interface_id(),
            kind: ImplementationKind::Structural(value),
            doc: None,
        }
    }
}

impl Identify for Implementation {
    fn identifier(&self) -> String {
        self.path_name().to_string()
    }
}

impl Document for Implementation {
    fn doc(&self) -> Option<String> {
        self.doc.clone()
    }
}

impl PathNameSelf for Implementation {
    fn path_name(&self) -> &PathName {
        &self.name
    }
}

impl MoveDb<Id<Implementation>> for Implementation {
    fn move_db(
        &self,
        original_db: &dyn Ir,
        target_db: &dyn Ir,
        prefix: &Option<Name>,
    ) -> Result<Id<Implementation>> {
        Ok(match self.kind() {
            ImplementationKind::Structural(structure) => Implementation {
                name: self.name.clone(),
                interface: structure.interface_id(),
                kind: ImplementationKind::Structural(structure.move_db(
                    original_db,
                    target_db,
                    prefix,
                )?),
                doc: self.doc.clone(),
            }
            .intern(target_db),
            ImplementationKind::Link => todo!(),
        })
    }
}
