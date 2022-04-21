use std::sync::Arc;

use til_query::{
    common::{
        signals::{PhysicalSignals, PhysicalStreamDirection},
        stream_direction::StreamDirection,
        transfer::{
            element_type::ElementType,
            physical_transfer::{LastMode, StrobeMode},
        },
    },
    ir::physical_properties::InterfaceDirection,
};
use tydi_common::{
    error::Result,
    name::{PathName, PathNameSelf},
    numbers::NonNegative,
    traits::{Identify, Reversed},
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

impl PhysicalSignals for PhysicalStreamProcess {
    fn direction(&self) -> PhysicalStreamDirection {
        let interface_dir = match self.stream_object().interface_direction() {
            InterfaceDirection::Out => PhysicalStreamDirection::Source,
            InterfaceDirection::In => PhysicalStreamDirection::Sink,
        };
        match self.stream_object().stream_direction() {
            StreamDirection::Forward => interface_dir,
            StreamDirection::Reverse => interface_dir.reversed(),
        }
    }

    fn comment(&mut self, _comment: &str) {
        // TODO: Add support for arbitrary comments in Processes
        ()
    }

    fn act_data_default(&mut self) -> Result<()> {
        todo!()
    }

    fn assert_data_default(&mut self, message: &str) -> Result<()> {
        todo!()
    }

    fn act_data(&mut self, element_lane: NonNegative, data: &ElementType) -> Result<()> {
        todo!()
    }

    fn assert_data(
        &mut self,
        element_lane: NonNegative,
        data: &ElementType,
        message: &str,
    ) -> Result<()> {
        todo!()
    }

    fn act_user_default(&mut self) -> Result<()> {
        todo!()
    }

    fn assert_user_default(&mut self, message: &str) -> Result<()> {
        todo!()
    }

    fn act_user(&mut self, user: &ElementType) -> Result<()> {
        todo!()
    }

    fn assert_user(&mut self, user: &ElementType, message: &str) -> Result<()> {
        todo!()
    }

    fn act_stai(&mut self, stai: NonNegative) -> Result<()> {
        todo!()
    }

    fn assert_stai(&mut self, stai: NonNegative, message: &str) -> Result<()> {
        todo!()
    }

    fn act_endi(&mut self, endi: NonNegative) -> Result<()> {
        todo!()
    }

    fn assert_endi(&mut self, endi: NonNegative, message: &str) -> Result<()> {
        todo!()
    }

    fn act_strb(&mut self, strb: StrobeMode) -> Result<()> {
        todo!()
    }

    fn assert_strb(&mut self, strb: StrobeMode, message: &str) -> Result<()> {
        todo!()
    }

    fn act_last(&mut self, last: LastMode) -> Result<()> {
        todo!()
    }

    fn assert_last(&mut self, last: LastMode, message: &str) -> Result<()> {
        todo!()
    }

    fn handshake(&mut self) -> Result<()> {
        todo!()
    }

    fn handshake_continue(&mut self, message: &str) -> Result<()> {
        todo!()
    }

    fn handshake_start(&mut self) -> Result<()> {
        todo!()
    }

    fn handshake_end(&mut self) -> Result<()> {
        todo!()
    }
}
