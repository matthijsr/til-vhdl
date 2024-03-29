use std::path::PathBuf;

use serde::Deserialize;
use til_query::ir::{db::Database, project::Project, Ir};
use tydi_common::error::{Error, Result, TryResult, WrapError};

use crate::query::file_to_project;

#[derive(Deserialize)]
pub struct ProjectFile {
    name: String,
    files: Vec<String>,
    output_path: String,
    config: Option<ConfigKeys>,
}

#[derive(Deserialize)]
pub struct ConfigKeys {
    link_relative_to_file: bool,
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

    pub fn config(&self) -> &Option<ConfigKeys> {
        &self.config
    }

    pub fn config_link_relative_to_file(&self) -> bool {
        match self.config() {
            Some(config) => config.link_relative_to_file,
            None => false,
        }
    }
}

pub fn from_path(proj_file_path: impl TryResult<PathBuf>) -> Result<Database> {
    let mut proj_file_path = proj_file_path.try_result()?;
    let src = std::fs::read_to_string(&proj_file_path)
        .map_err(|err| Error::FileIOError(format!("Unable to read project file: {}", err)))?;
    proj_file_path.pop();
    into_query_storage(src, proj_file_path)
}

pub fn into_query_storage(
    src: impl Into<String>,
    location: impl TryResult<PathBuf>,
) -> Result<Database> {
    let src = src.into();
    let project_info: ProjectFile = toml::from_str(&src)
        .map_err(|err| Error::ProjectError(format!("Unable to parse the project file: {}", err)))?;
    let mut db = Database::default();
    let location: PathBuf = location.try_result()?;

    db.set_project(Project::new(
        project_info.name(),
        location.clone(),
        Some(project_info.output_path()),
    )?);

    for file in project_info.files() {
        let mut file_location = location.clone();
        file_location.push(file);
        let file_src = std::fs::read_to_string(&file_location).map_err(|err| {
            Error::FileIOError(format!(
                "Unable to read file from project: {}",
                err.to_string()
            ))
        })?;
        let link_root = match project_info.config_link_relative_to_file() {
            true => {
                let mut root = file_location.clone();
                root.pop();
                root
            }
            false => location.clone(),
        };
        file_to_project(file_src, &mut db, link_root)
            .wrap_err(Error::ProjectError(format!("Error in file \"{}\"", file)))?;
    }

    Ok(db)
}
