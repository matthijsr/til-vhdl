use std::collections::BTreeMap;

use tydi_common::{
    error::{Error, Result, TryResult},
    name::{Name, PathName, PathNameSelf},
    traits::Identify,
};
use tydi_intern::Id;

use crate::{
    common::logical::logicaltype::{stream::Stream, LogicalType},
    ir::{
        implementation::Implementation,
        streamlet::Streamlet,
        traits::{GetSelf, InternSelf, MoveDb},
        Ir,
    },
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

    pub fn types(&self, db: &dyn Ir) -> BTreeMap<Name, LogicalType> {
        self.type_ids()
            .iter()
            .map(|(name, id)| (name.clone(), id.get(db)))
            .collect()
    }

    pub fn streamlet_ids(&self) -> &BTreeMap<Name, Id<Streamlet>> {
        &self.streamlets
    }

    pub fn streamlets(&self, db: &dyn Ir) -> BTreeMap<Name, Streamlet> {
        self.streamlet_ids()
            .iter()
            .map(|(name, id)| (name.clone(), id.get(db)))
            .collect()
    }

    pub fn implementation_ids(&self) -> &BTreeMap<Name, Id<Implementation>> {
        &self.implementations
    }

    pub fn implementations(&self, db: &dyn Ir) -> BTreeMap<Name, Implementation> {
        self.implementation_ids()
            .iter()
            .map(|(name, id)| (name.clone(), id.get(db)))
            .collect()
    }

    pub fn import_type(
        &mut self,
        name: impl TryResult<Name>,
        type_id: Id<LogicalType>,
    ) -> Result<()> {
        let name = name.try_result()?;
        match self.types.insert(name.clone(), type_id) {
            None => Ok(()),
            Some(_) => Err(Error::InvalidArgument(format!(
                "A type with name {} already exists in namespace {}.",
                name,
                self.path_name()
            ))),
        }
    }

    pub fn import_streamlet(
        &mut self,
        name: impl TryResult<Name>,
        streamlet_id: Id<Streamlet>,
    ) -> Result<()> {
        let name = name.try_result()?;
        match self.streamlets.insert(name.clone(), streamlet_id) {
            None => Ok(()),
            Some(_) => Err(Error::InvalidArgument(format!(
                "A streamlet with name {} already exists in namespace {}.",
                name,
                self.path_name()
            ))),
        }
    }

    pub fn import_implementation(
        &mut self,
        name: impl TryResult<Name>,
        implementation_id: Id<Implementation>,
    ) -> Result<()> {
        let name = name.try_result()?;
        match self.implementations.insert(name.clone(), implementation_id) {
            None => Ok(()),
            Some(_) => Err(Error::InvalidArgument(format!(
                "An implementation with name {} already exists in namespace {}.",
                name,
                self.path_name()
            ))),
        }
    }

    pub fn define_type(
        &mut self,
        db: &dyn Ir,
        name: impl TryResult<Name>,
        typ: impl TryResult<LogicalType>,
    ) -> Result<Id<LogicalType>> {
        let name = name.try_result()?;
        let type_id = typ.try_result()?.intern(db);
        self.import_type(name, type_id)?;
        Ok(type_id)
    }

    pub fn define_streamlet(
        &mut self,
        db: &dyn Ir,
        name: impl TryResult<Name>,
        streamlet: impl TryResult<Streamlet>,
    ) -> Result<Id<Streamlet>> {
        let name = name.try_result()?;
        let streamlet = streamlet
            .try_result()?
            .with_name(self.path_name().with_child(&name))?;
        let streamlet_id = streamlet.intern(db);
        self.import_streamlet(name, streamlet_id)?;
        Ok(streamlet_id)
    }

    pub fn define_implementation(
        &mut self,
        db: &dyn Ir,
        name: impl TryResult<Name>,
        implementation: impl TryResult<Implementation>,
    ) -> Result<Id<Implementation>> {
        let name = name.try_result()?;
        let implementation = implementation
            .try_result()?
            .with_name(self.path_name().with_child(&name))?;
        let implementation_id = implementation.intern(db);
        self.import_implementation(name, implementation_id)?;
        Ok(implementation_id)
    }

    pub fn get_type_id(&self, name: impl TryResult<Name>) -> Result<Id<LogicalType>> {
        let name = name.try_result()?;
        self.type_ids()
            .get(&name)
            .cloned()
            .ok_or(Error::InvalidArgument(format!(
                "A type with name {} does not exist in namespace {}",
                name,
                self.path_name()
            )))
    }

    pub fn get_type(&self, db: &dyn Ir, name: impl TryResult<Name>) -> Result<LogicalType> {
        Ok(self.get_type_id(name)?.get(db))
    }

    pub fn get_streamlet_id(&self, name: impl TryResult<Name>) -> Result<Id<Streamlet>> {
        let name = name.try_result()?;
        self.streamlet_ids()
            .get(&name)
            .cloned()
            .ok_or(Error::InvalidArgument(format!(
                "A streamlet with name {} does not exist in namespace {}",
                name,
                self.path_name()
            )))
    }

    pub fn get_streamlet(&self, db: &dyn Ir, name: impl TryResult<Name>) -> Result<Streamlet> {
        Ok(self.get_streamlet_id(name)?.get(db))
    }

    pub fn get_implementation_id(&self, name: impl TryResult<Name>) -> Result<Id<Implementation>> {
        let name = name.try_result()?;
        self.implementation_ids()
            .get(&name)
            .cloned()
            .ok_or(Error::InvalidArgument(format!(
                "An implementation with name {} does not exist in namespace {}",
                name,
                self.path_name()
            )))
    }

    pub fn get_implementation(
        &self,
        db: &dyn Ir,
        name: impl TryResult<Name>,
    ) -> Result<Implementation> {
        Ok(self.get_implementation_id(name)?.get(db))
    }

    pub fn get_stream_id(&self, db: &dyn Ir, name: impl TryResult<Name>) -> Result<Id<Stream>> {
        let name = name.try_result()?;
        let typ = self.get_type(db, &name)?;
        match &typ {
            LogicalType::Stream(stream_id) => Ok(*stream_id),
            _ => Err(Error::InvalidArgument(format!(
                "Type {} in namespace {} is not a Stream, it is a {}",
                name,
                self.path_name(),
                typ
            ))),
        }
    }

    pub fn get_stream(&self, db: &dyn Ir, name: impl TryResult<Name>) -> Result<Stream> {
        Ok(self.get_stream_id(db, name)?.get(db))
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

#[cfg(test)]
mod tests {
    use crate::{ir::db::Database, test_utils::test_stream_id};

    use super::*;

    #[test]
    fn get_stream_from_type() -> Result<()> {
        let _db = Database::default();
        let db = &_db;

        let mut namespace = Namespace::new("namespace")?;
        let stream_id = test_stream_id(db, 4)?;
        namespace.define_type(db, "typ", stream_id)?;
        assert_eq!(stream_id, namespace.get_stream_id(db, "typ")?);
        Ok(())
    }
}
