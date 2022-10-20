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

use super::{
    traits::{InternSelf, MoveDb},
    Ir,
};

pub mod interface;
pub mod namespace;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Project {
    /// The project name
    name: Name,
    /// The root folder of the project.
    /// Relevant for links to behavioural implementations, and for determining the output folder.
    location: PathBuf,
    /// The expected output directory
    output_path: Option<PathBuf>,
    /// Namespaces within the project
    namespaces: BTreeMap<PathName, Id<Namespace>>,
    /// External dependencies
    imports: BTreeMap<Name, Project>,
}

impl Project {
    pub fn new(
        name: impl TryResult<Name>,
        location: impl TryResult<PathBuf>,
        output_path: Option<impl TryResult<PathBuf>>,
    ) -> Result<Self> {
        Ok(Project {
            name: name.try_result()?,
            location: location.try_result()?,
            output_path: output_path.map(|x| x.try_result()).transpose()?,
            namespaces: BTreeMap::new(),
            imports: BTreeMap::new(),
        })
    }

    pub fn location(&self) -> &Path {
        self.location.as_path()
    }

    pub fn output_path(&self) -> &Option<PathBuf> {
        &self.output_path
    }

    pub fn namespaces(&self) -> &BTreeMap<PathName, Id<Namespace>> {
        &self.namespaces
    }

    pub fn imports(&self) -> &BTreeMap<Name, Project> {
        &self.imports
    }

    fn import_project_recursive(
        &mut self,
        db: &dyn Ir,
        project: &Project,
        proj_db: &dyn Ir,
        alias_name: Name,
        is_root: bool,
    ) -> Result<()> {
        for (import_name, import_project) in project.imports() {
            if let Err(err) = self.import_project_recursive(
                db,
                import_project,
                proj_db,
                import_name.clone(),
                false,
            ) {
                return Err(Error::ProjectError(format!(
                    "Unable to import project {}, due to a problem importing its dependency {}: {}",
                    project.name(),
                    import_name,
                    err
                )));
            }
        }

        let prefix = if is_root {
            Some(self.name().clone())
        } else {
            None
        };

        let namespaces = project
            .namespaces()
            .iter()
            .map(|(k, v)| Ok((k.clone(), v.move_db(proj_db, db, &prefix)?)))
            .collect::<Result<_>>()?;
        self.imports.insert(
            alias_name,
            Project {
                name: project.name.clone(),
                location: project.location.clone(),
                output_path: project.output_path.clone(),
                namespaces,
                imports: BTreeMap::new(),
            },
        );

        Ok(())
    }

    /// Import another project using an alias
    pub fn import_project_as(
        &mut self,
        db: &dyn Ir,
        project: &Project,
        proj_db: &dyn Ir,
        alias_name: impl TryResult<Name>,
    ) -> Result<()> {
        let alias_name = alias_name.try_result()?;
        if self.imports().contains_key(&alias_name) {
            Err(Error::InvalidArgument(format!(
                "Project already has an import with name {}",
                &alias_name
            )))
        } else if self
            .namespaces()
            .keys()
            .filter_map(|path| path.first())
            .any(|first| first == &alias_name)
        {
            Err(Error::InvalidArgument(format!(
                "Importing project {} would overlap with existing namespace",
                &alias_name
            )))
        } else {
            self.import_project_recursive(db, project, proj_db, alias_name, true)
        }
    }

    /// Import another project
    pub fn import_project(
        &mut self,
        db: &dyn Ir,
        project: &Project,
        proj_db: &dyn Ir,
    ) -> Result<()> {
        self.import_project_as(db, project, proj_db, project.name())
    }

    pub fn add_namespace(&mut self, db: &dyn Ir, namespace: Namespace) -> Result<()> {
        let namespace_path = namespace.path_name().clone();
        if let Some(name) = namespace_path.first() {
            if self.imports().contains_key(name) {
                return Err(Error::InvalidArgument(format!("Cannot add namespace with root {}, as this overlaps with an existing project import.", name)));
            }
        }

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
