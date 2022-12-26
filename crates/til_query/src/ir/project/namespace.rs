use std::{collections::BTreeMap, sync::Arc};

use tydi_common::{
    error::{Error, Result, TryOptional, TryResult},
    name::{Name, PathName, PathNameSelf},
    traits::Identify,
};
use tydi_intern::Id;

use crate::{
    common::logical::logicaltype::{stream::Stream, LogicalType},
    ir::{
        generics::{param_value::GenericParamValue, GenericParameter},
        implementation::Implementation,
        streamlet::Streamlet,
        traits::{GetSelf, InternArc, InternSelf, MoveDb, TryIntern},
        Ir,
    },
};

use super::{interface::Interface, type_declaration::TypeDeclaration};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Namespace {
    /// The name of the Namespace within its parent project
    name: PathName,
    /// The types declared within the namespace.
    /// Names are purely for tracking, and do not affect type equivalence.
    types: BTreeMap<Name, TypeDeclaration>,
    /// The streamlets declared within the namespace.
    streamlets: BTreeMap<Name, Id<Arc<Streamlet>>>,
    /// The implementations declared within the namespace.
    implementations: BTreeMap<Name, Id<Implementation>>,
    /// Interface declarations.
    /// As implementations and streamlets both contain an Interface,
    /// they are also declared as interfaces.
    /// This means that streamlet and implementation names cannot overlap.
    interfaces: BTreeMap<Name, Id<Arc<Interface>>>,
}

impl Namespace {
    pub fn new(name: impl TryResult<PathName>) -> Result<Self> {
        Ok(Namespace {
            name: name.try_result()?,
            types: BTreeMap::new(),
            streamlets: BTreeMap::new(),
            implementations: BTreeMap::new(),
            interfaces: BTreeMap::new(),
        })
    }

    pub fn type_decls(&self) -> &BTreeMap<Name, TypeDeclaration> {
        &self.types
    }

    // pub fn types(&self, db: &dyn Ir) -> BTreeMap<Name, LogicalType> {
    //     self.type_decls()
    //         .iter()
    //         .map(|(name, id)| (name.clone(), id.get(db)))
    //         .collect()
    // }

    pub fn streamlet_ids(&self) -> &BTreeMap<Name, Id<Arc<Streamlet>>> {
        &self.streamlets
    }

    pub fn streamlets(&self, db: &dyn Ir) -> BTreeMap<Name, Arc<Streamlet>> {
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

    pub fn interface_ids(&self) -> &BTreeMap<Name, Id<Arc<Interface>>> {
        &self.interfaces
    }

    pub fn interfaces(&self, db: &dyn Ir) -> BTreeMap<Name, Arc<Interface>> {
        self.interface_ids()
            .iter()
            .map(|(name, id)| (name.clone(), id.get(db)))
            .collect()
    }

    pub fn import_type(
        &mut self,
        name: impl TryResult<Name>,
        type_decl: TypeDeclaration,
    ) -> Result<()> {
        let name = name.try_result()?;
        match self.types.insert(name.clone(), type_decl) {
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
        streamlet_id: Id<Arc<Streamlet>>,
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

    pub fn import_interface(
        &mut self,
        name: impl TryResult<Name>,
        interface_id: Id<Arc<Interface>>,
    ) -> Result<()> {
        let name = name.try_result()?;
        match self.interfaces.insert(name.clone(), interface_id) {
            None => Ok(()),
            Some(_) => Err(Error::InvalidArgument(format!(
                "An interface with name {} already exists in namespace {}.",
                name,
                self.path_name()
            ))),
        }
    }

    pub fn define_type_no_params(
        &mut self,
        db: &dyn Ir,
        name: impl TryResult<Name>,
        typ: impl TryResult<LogicalType>,
    ) -> Result<()> {
        let name = name.try_result()?;
        let type_id = typ.try_result()?.intern(db);
        let type_decl =
            TypeDeclaration::try_new_no_params(db, self.path_name().with_child(&name), type_id)?;
        self.import_type(name, type_decl)?;
        Ok(())
    }

    pub fn define_type(
        &mut self,
        db: &dyn Ir,
        name: impl TryResult<Name>,
        typ: impl TryResult<LogicalType>,
        parameters: impl IntoIterator<Item = impl TryResult<GenericParameter>>,
    ) -> Result<()> {
        let name = name.try_result()?;
        let type_id = typ.try_result()?.intern(db);
        let type_decl =
            TypeDeclaration::try_new(db, self.path_name().with_child(&name), type_id, parameters)?;
        self.import_type(name, type_decl)?;
        Ok(())
    }

    pub fn define_streamlet(
        &mut self,
        db: &dyn Ir,
        name: impl TryResult<Name>,
        streamlet: impl TryResult<Streamlet>,
    ) -> Result<Id<Arc<Streamlet>>> {
        let name = name.try_result()?;
        let streamlet = streamlet
            .try_result()?
            .with_name(self.path_name().with_child(&name));
        let streamlet_id = streamlet.intern_arc(db);
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
            .with_name(self.path_name().with_child(&name));
        let implementation_id = implementation.intern(db);
        self.import_implementation(name, implementation_id)?;
        Ok(implementation_id)
    }

    pub fn define_interface(
        &mut self,
        db: &dyn Ir,
        name: impl TryResult<Name>,
        interface: impl TryIntern<Arc<Interface>>,
    ) -> Result<Id<Arc<Interface>>> {
        let name = name.try_result()?;
        let interface_id = interface.try_intern(db)?;
        self.import_interface(name, interface_id)?;
        Ok(interface_id)
    }

    pub fn get_type_id_no_assignments(
        &self,
        db: &dyn Ir,
        name: impl TryResult<Name>,
    ) -> Result<Id<LogicalType>> {
        let name = name.try_result()?;
        self.type_decls()
            .get(&name)
            .cloned()
            .ok_or(Error::InvalidArgument(format!(
                "A type with name {} does not exist in namespace {}",
                name,
                self.path_name()
            )))?
            .type_id(db)
    }

    pub fn get_type_id(
        &self,
        db: &dyn Ir,
        name: impl TryResult<Name>,
        parameter_assignments: impl IntoIterator<
            Item = (impl TryOptional<Name>, impl TryResult<GenericParamValue>),
        >,
    ) -> Result<Id<LogicalType>> {
        let name = name.try_result()?;
        self.type_decls()
            .get(&name)
            .cloned()
            .ok_or(Error::InvalidArgument(format!(
                "A type with name {} does not exist in namespace {}",
                name,
                self.path_name()
            )))?
            .with_assignments(parameter_assignments)?
            .type_id(db)
    }

    pub fn get_type_no_assignments(
        &self,
        db: &dyn Ir,
        name: impl TryResult<Name>,
    ) -> Result<LogicalType> {
        Ok(self.get_type_id_no_assignments(db, name)?.get(db))
    }

    pub fn get_type(
        &self,
        db: &dyn Ir,
        name: impl TryResult<Name>,
        parameter_assignments: impl IntoIterator<
            Item = (impl TryOptional<Name>, impl TryResult<GenericParamValue>),
        >,
    ) -> Result<LogicalType> {
        Ok(self.get_type_id(db, name, parameter_assignments)?.get(db))
    }

    pub fn get_streamlet_id(&self, name: impl TryResult<Name>) -> Result<Id<Arc<Streamlet>>> {
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

    pub fn get_streamlet(&self, db: &dyn Ir, name: impl TryResult<Name>) -> Result<Arc<Streamlet>> {
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

    pub fn get_stream_id_no_assignments(
        &self,
        db: &dyn Ir,
        name: impl TryResult<Name>,
    ) -> Result<Id<Stream>> {
        let name = name.try_result()?;
        let typ = self.get_type_no_assignments(db, &name)?;
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

    pub fn get_stream_id(
        &self,
        db: &dyn Ir,
        name: impl TryResult<Name>,
        parameter_assignments: impl IntoIterator<
            Item = (impl TryOptional<Name>, impl TryResult<GenericParamValue>),
        >,
    ) -> Result<Id<Stream>> {
        let name = name.try_result()?;
        let typ = self.get_type(db, &name, parameter_assignments)?;
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

    pub fn get_interface_id(&self, name: impl TryResult<Name>) -> Result<Id<Arc<Interface>>> {
        let name = name.try_result()?;
        self.interface_ids()
            .get(&name)
            .cloned()
            .ok_or(Error::InvalidArgument(format!(
                "An interface with name {} does not exist in namespace {}",
                name,
                self.path_name()
            )))
    }

    pub fn get_interface(&self, db: &dyn Ir, name: impl TryResult<Name>) -> Result<Arc<Interface>> {
        Ok(self.get_interface_id(name)?.get(db))
    }

    pub fn get_stream(&self, db: &dyn Ir, name: impl TryResult<Name>) -> Result<Stream> {
        Ok(self.get_stream_id_no_assignments(db, name)?.get(db))
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
            .type_decls()
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
        let interfaces = self
            .interface_ids()
            .iter()
            .map(|(k, v)| Ok((k.clone(), v.move_db(original_db, target_db, prefix)?)))
            .collect::<Result<_>>()?;
        Ok(Namespace {
            name: self.name.clone(),
            types,
            streamlets,
            implementations,
            interfaces,
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
        namespace.define_type_no_params(db, "typ", stream_id)?;
        assert_eq!(
            stream_id,
            namespace.get_stream_id_no_assignments(db, "typ")?
        );
        Ok(())
    }
}
