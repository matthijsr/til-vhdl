pub mod link;
pub mod structure;

use tydi_common::{
    error::{Result, TryResult},
    name::{Name, PathName, PathNameSelf},
    traits::{Document, Identify},
};
use tydi_intern::Id;

use self::{link::Link, structure::Structure};

use super::{
    traits::{InternSelf, MoveDb},
    Ir,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Implementation {
    name: PathName,
    kind: ImplementationKind,
    doc: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ImplementationKind {
    Structural(Structure),
    Link(Link),
}

impl Implementation {
    pub fn structural(structure: impl TryResult<Structure>) -> Result<Self> {
        Ok(Implementation {
            name: PathName::new_empty(),
            kind: ImplementationKind::Structural(structure.try_result()?),
            doc: None,
        })
    }

    pub fn link(link: impl TryResult<Link>) -> Result<Self> {
        Ok(Implementation {
            name: PathName::new_empty(),
            kind: ImplementationKind::Link(link.try_result()?),
            doc: None,
        })
    }

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
}

impl From<Structure> for Implementation {
    fn from(value: Structure) -> Self {
        Implementation {
            name: PathName::new_empty(),
            kind: ImplementationKind::Structural(value),
            doc: None,
        }
    }
}

impl From<Link> for Implementation {
    fn from(value: Link) -> Self {
        Implementation {
            name: PathName::new_empty(),
            kind: ImplementationKind::Link(value),
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
    fn doc(&self) -> Option<&String> {
        self.doc.as_ref()
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
                kind: ImplementationKind::Structural(structure.move_db(
                    original_db,
                    target_db,
                    prefix,
                )?),
                doc: self.doc.clone(),
            }
            .intern(target_db),
            ImplementationKind::Link(_) => todo!(),
        })
    }
}
