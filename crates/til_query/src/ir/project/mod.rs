use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use tydi_common::{
    error::{Error, Result, TryResult},
    name::{Name, NameSelf, PathName, PathNameSelf},
    traits::Identify,
};
use tydi_intern::Id;

use self::namespace::Namespace;

use super::{InternSelf, Ir};

pub mod namespace;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Project {
    /// The project name
    name: Name,
    /// The root folder of the project.
    /// Relevant for links to behavioural implementations, and for determining the output folder.
    location: PathBuf,
    /// Namespaces within the project
    namespaces: BTreeMap<PathName, Id<Namespace>>,
    /// External dependencies
    imports: BTreeMap<Name, Project>,
}

impl Project {
    pub fn new(name: impl TryResult<Name>, location: impl TryResult<PathBuf>) -> Result<Self> {
        Ok(Project {
            name: name.try_result()?,
            location: location.try_result()?,
            namespaces: BTreeMap::new(),
            imports: BTreeMap::new(),
        })
    }

    pub fn location(&self) -> &Path {
        self.location.as_path()
    }

    pub fn namespaces(&self) -> &BTreeMap<PathName, Id<Namespace>> {
        &self.namespaces
    }

    pub fn imports(&self) -> &BTreeMap<Name, Project> {
        &self.imports
    }

    /// Validate whether the project's namespaces do not overlap
    pub fn validate_namespaces(&self) -> Result<()> {
        for namespace_name in self.namespaces().keys().map(|path| path.first()) {
            if let Some(namespace_root) = namespace_name {
                if self.imports().contains_key(namespace_root) {
                    return Err(Error::ProjectError(format!(
                        "Project has overlapping namespace and import {}",
                        namespace_root
                    )));
                }
            }
        }
        Ok(())
    }

    pub fn add_namespace(&mut self, db: &dyn Ir, namespace: Namespace) -> Result<()> {
        let namespace_path = namespace.path_name().clone();
        let namespace_id = namespace.intern(db);
        if self.namespaces.insert(namespace_path.clone(), namespace_id) == None {
            Ok(())
        } else {
            Err(Error::InvalidArgument(format!(
                "A namespace with name {} was already declared",
                namespace_path
            )))
        }
    }
}

impl Identify for Project {
    fn identifier(&self) -> String {
        self.name().to_string()
    }
}

impl NameSelf for Project {
    fn name(&self) -> &Name {
        &self.name
    }
}
