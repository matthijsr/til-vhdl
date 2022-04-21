use std::sync::Arc;

use tydi_common::{
    name::{PathName, PathNameSelf},
    traits::Identify,
};
use tydi_vhdl::process::Process;

use crate::ir::streamlet::PhysicalStreamObject;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PhysicalStreamProcess {
    name: PathName,
    process: Process,
    stream_object: Arc<PhysicalStreamObject>,
}

impl PhysicalStreamProcess {
    /// Get a reference to the physical stream process's process.
    #[must_use]
    pub fn process(&self) -> &Process {
        &self.process
    }

    /// Get a reference to the physical stream process's stream object.
    #[must_use]
    pub fn stream_object(&self) -> &PhysicalStreamObject {
        self.stream_object.as_ref()
    }
}

impl PathNameSelf for PhysicalStreamProcess {
    fn path_name(&self) -> &PathName {
        &self.name
    }
}

impl Identify for PhysicalStreamProcess {
    fn identifier(&self) -> String {
        self.path_name().to_string()
    }
}

impl<T: Into<Arc<PhysicalStreamObject>>> From<T> for PhysicalStreamProcess {
    fn from(val: T) -> Self {
        let stream_object = val.into();
        let process = Process::new(stream_object.path_name());
        Self {
            name: stream_object.path_name().clone(),
            process,
            stream_object,
        }
    }
}
