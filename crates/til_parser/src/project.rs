use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use serde::Deserialize;
use til_query::ir::{db::Database, project::Project, Ir};
use tydi_common::error::{Error, Result, TryResult, WrapError};

#[derive(Deserialize)]
pub struct ProjectFile {
    name: String,
    files: Vec<String>,
    output_path: String,
}

impl ProjectFile {
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn files(&self) -> &[String] {
        self.files.as_ref()
    }

    pub fn output_path(&self) -> &str {
        self.output_path.as_ref()
    }
}

pub fn from_path(proj_file_path: impl Into<String>) -> tydi_common::error::Result<Database> {
    todo!()
}

pub fn into_query_storage(
    src: impl Into<String>,
    location: impl TryResult<PathBuf>,
) -> tydi_common::error::Result<Database> {
    let src = src.into();
    let project_info: ProjectFile = toml::from_str(&src)
        .map_err(|err| Error::ProjectError(format!("Unable to parse the project file: {}", err)))?;
    let mut db = Database::default();
    db.set_project(Arc::new(Mutex::new(Project::new(
        project_info.name(),
        location,
        Some(project_info.output_path()),
    )?)));

    todo!();
}
