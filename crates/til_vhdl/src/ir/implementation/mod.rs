pub mod structural;

use tydi_common::{
    error::{Result, TryResult},
    name::{Name, NameSelf},
};

use self::structural::Structure;

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

impl NameSelf for Implementation {
    fn name(&self) -> &Name {
        &self.name
    }
}
