pub mod structure;

use tydi_common::{
    error::{Result, TryResult},
    name::{Name, PathName, PathNameSelf},
    traits::Identify,
};
use tydi_intern::Id;

use self::structure::Structure;

use super::{
    traits::{InternSelf, MoveDb},
    Ir,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Implementation {
    name: PathName,
    kind: ImplementationKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ImplementationKind {
    Structural(Structure),
    Link,
}

impl Implementation {
    pub fn structural(structure: impl TryResult<Structure>) -> Result<Self> {
        Ok(Implementation {
            name: PathName::new_empty(),
            kind: ImplementationKind::Structural(structure.try_result()?),
        })
    }

    /// TODO
    pub fn link() -> Self {
        Implementation {
            name: PathName::new_empty(),
            kind: ImplementationKind::Link,
        }
    }

    pub fn with_name(mut self, name: impl TryResult<PathName>) -> Result<Self> {
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
        }
    }
}

impl Identify for Implementation {
    fn identifier(&self) -> String {
        self.path_name().to_string()
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
            }
            .intern(target_db),
            ImplementationKind::Link => todo!(),
        })
    }
}
