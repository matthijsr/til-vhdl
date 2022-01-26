pub mod structure;

use tydi_common::{
    error::{Result, TryResult},
    name::{Name, NameSelf},
    traits::Identify,
};
use tydi_intern::Id;

use self::structure::Structure;

use super::{InternSelf, MoveDb};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Implementation {
    name: Name,
    kind: ImplementationKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ImplementationKind {
    Structural(Structure),
    Link,
}

impl Implementation {
    pub fn structural(
        name: impl TryResult<Name>,
        structure: impl TryResult<Structure>,
    ) -> Result<Self> {
        Ok(Implementation {
            name: name.try_result()?,
            kind: ImplementationKind::Structural(structure.try_result()?),
        })
    }

    /// TODO
    pub fn link(name: impl TryResult<Name>) -> Result<Self> {
        Ok(Implementation {
            name: name.try_result()?,
            kind: ImplementationKind::Link,
        })
    }

    pub fn kind(&self) -> &ImplementationKind {
        &self.kind
    }
}

impl Identify for Implementation {
    fn identifier(&self) -> String {
        self.name().to_string()
    }
}

impl NameSelf for Implementation {
    fn name(&self) -> &Name {
        &self.name
    }
}

impl MoveDb<Id<Implementation>> for Implementation {
    fn move_db(
        &self,
        original_db: &dyn super::Ir,
        target_db: &dyn super::Ir,
    ) -> Result<Id<Implementation>> {
        Ok(match self.kind() {
            ImplementationKind::Structural(structure) => Implementation {
                name: self.name.clone(),
                kind: ImplementationKind::Structural(structure.move_db(original_db, target_db)?),
            }
            .intern(target_db),
            ImplementationKind::Link => todo!(),
        })
    }
}
