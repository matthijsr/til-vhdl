use std::path::{Path, PathBuf};

use tydi_common::{
    error::{Error, Result, TryResult},
    name::{PathName, PathNameSelf},
    traits::Identify,
};
use tydi_intern::Id;

use crate::ir::project::interface::InterfaceCollection;

/// This node represents a link to a behavioural `Implementation`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Link {
    /// The interface associated with this link
    ///
    /// It's possible for back-ends to use this information to select a specific file
    interface: Id<InterfaceCollection>,
    /// The name of the component/entity or equivalent
    ///
    /// This can be used in two ways:
    /// 1. To lock the name of a streamlet, ensuring that its declaration is consistent with that of the definition in the path
    /// 2. To provide the name of the streamlet to a back-end, and modify the name used in the definition
    streamlet_name: PathName,
    /// The path to the implementation
    ///
    /// This can refer to a file directly, or to a directory .Its usage is entirely dependent on the back-end.
    path: PathBuf,
}

impl Link {
    pub fn try_get(
        interface: Id<InterfaceCollection>,
        streamlet_name: impl TryResult<PathName>,
        path: impl Into<PathBuf>,
    ) -> Result<Self> {
        let path = path.into();
        let pathr = path.as_path();
        let streamlet_name = streamlet_name.try_result()?;
        if pathr.exists() {
            Ok(Link {
                interface,
                streamlet_name,
                path,
            })
        } else {
            Err(Error::FileIOError(format!(
                "Path {} does not exist",
                pathr.display()
            )))
        }
    }

    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    pub fn interface_id(&self) -> Id<InterfaceCollection> {
        self.interface
    }
}

impl Identify for Link {
    fn identifier(&self) -> String {
        self.path_name().to_string()
    }
}

impl PathNameSelf for Link {
    fn path_name(&self) -> &PathName {
        &self.streamlet_name
    }
}
