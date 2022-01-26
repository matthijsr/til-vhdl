use std::collections::BTreeMap;

use tydi_common::{
    error::{Result, TryResult},
    name::{Name, PathName, PathNameSelf},
    traits::Identify,
};
use tydi_intern::Id;

use crate::{
    common::logical::logicaltype::LogicalType,
    ir::{implementation::Implementation, streamlet::Streamlet, InternSelf, Ir, MoveDb},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Namespace {
    /// The name of the Namespace within its parent project
    name: PathName,
    /// The types declared within the namespace.
    /// Names are purely for tracking, and do not affect type equivalence.
    types: BTreeMap<Name, Id<LogicalType>>,
    /// The streamlets declared within the namespace.
    streamlets: BTreeMap<Name, Id<Streamlet>>,
    /// The implementations declared within the namespace.
    implementations: BTreeMap<Name, Id<Implementation>>,
}

impl Namespace {
    pub fn new(name: impl TryResult<PathName>) -> Result<Self> {
        Ok(Namespace {
            name: name.try_result()?,
            types: BTreeMap::new(),
            streamlets: BTreeMap::new(),
            implementations: BTreeMap::new(),
        })
    }

    pub fn type_ids(&self) -> &BTreeMap<Name, Id<LogicalType>> {
        &self.types
    }

    pub fn streamlet_ids(&self) -> &BTreeMap<Name, Id<Streamlet>> {
        &self.streamlets
    }

    pub fn implementation_ids(&self) -> &BTreeMap<Name, Id<Implementation>> {
        &self.implementations
    }
}

impl Identify for Namespace {
    fn identifier(&self) -> String {
        self.path_name().to_string()
    }
}

impl PathNameSelf for Namespace {
    fn path_name(&self) -> &PathName {
        &self.name
    }
}

impl MoveDb<Id<Namespace>> for Namespace {
    fn move_db(
        &self,
        original_db: &dyn Ir,
        target_db: &dyn Ir,
        prefix: &Option<Name>,
    ) -> Result<Id<Namespace>> {
        let types = self
            .type_ids()
            .iter()
            .map(|(k, v)| Ok((k.clone(), v.move_db(original_db, target_db, prefix)?)))
            .collect::<Result<_>>()?;
        let streamlets = self
            .streamlet_ids()
            .iter()
            .map(|(k, v)| Ok((k.clone(), v.move_db(original_db, target_db, prefix)?)))
            .collect::<Result<_>>()?;
        let implementations = self
            .implementation_ids()
            .iter()
            .map(|(k, v)| Ok((k.clone(), v.move_db(original_db, target_db, prefix)?)))
            .collect::<Result<_>>()?;
        Ok(Namespace {
            name: self.name.clone(),
            types,
            streamlets,
            implementations,
        }
        .intern(target_db))
    }
}
