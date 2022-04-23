use std::sync::Arc;

use til_query::{
    common::{
        physical::signal_list::SignalList,
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
    error::{Result, TryResult},
    name::{PathName, PathNameSelf},
    numbers::NonNegative,
    traits::{Identify, Reversed},
};
use tydi_intern::Id;
use tydi_vhdl::{
    architecture::arch_storage::db::Database,
    assignment::{Assign, StdLogicValue},
    declaration::ObjectDeclaration,
    process::{
        statement::{wait::Wait, SequentialStatement},
        Process,
    },
    statement::relation::{edge::Edge, CombineRelation, CreateLogicalExpression, Relation},
};

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

    pub fn with_db<'a>(self, db: &'a Database) -> PhysicalStreamProcessWithDb<'a> {
        PhysicalStreamProcessWithDb { process: self, db }
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

pub struct PhysicalStreamProcessWithDb<'a> {
    process: PhysicalStreamProcess,
    db: &'a Database,
}

impl<'a> PhysicalStreamProcessWithDb<'a> {
    pub fn get(self) -> PhysicalStreamProcess {
        self.process
    }

    /// A helper function, since handshakes often call for a `rising_edge(clk)`
    fn rising_edge_clk(&self) -> Result<Edge> {
        Edge::rising_edge(self.db, self.process.stream_object().clock())
    }

    /// A helper function, since handshakes often call for a
    /// `wait until rising_edge(clk)`
    fn wait_until_rising_edge_clk(&self) -> Result<Wait> {
        Wait::wait().until_relation(self.db, self.rising_edge_clk()?)
    }

    fn signal_list(&self) -> &SignalList<Id<ObjectDeclaration>> {
        self.process.stream_object().signal_list()
    }

    fn add_statement(&mut self, statement: impl TryResult<SequentialStatement>) -> Result<()> {
        self.process.process.add_statement(self.db, statement)
    }

    fn is_high(&self, sig: Id<ObjectDeclaration>) -> Result<Relation> {
        Ok(sig.r_eq(self.db, StdLogicValue::Logic(true))?.into())
    }
}

impl<'a> PhysicalSignals for PhysicalStreamProcessWithDb<'a> {
    fn direction(&self) -> PhysicalStreamDirection {
        let interface_dir = match self.process.stream_object().interface_direction() {
            InterfaceDirection::Out => PhysicalStreamDirection::Source,
            InterfaceDirection::In => PhysicalStreamDirection::Sink,
        };
        match self.process.stream_object().stream_direction() {
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
        match self.direction() {
            PhysicalStreamDirection::Source => todo!(),
            PhysicalStreamDirection::Sink => {
                if let Some(ready) = *self.signal_list().ready() {
                    self.add_statement(Wait::wait().until_relation(
                        self.db,
                        self.is_high(ready)?.and(self.db, self.rising_edge_clk()?)?,
                    )?)?;
                }
            }
        }
        Ok(())
    }

    fn handshake_start(&mut self) -> Result<()> {
        // If there's no Valid signal, this stream is always valid
        if let Some(valid) = *self.signal_list().valid() {
            match self.direction() {
                PhysicalStreamDirection::Source => {
                    self.add_statement(
                        Wait::wait().until_relation(
                            self.db,
                            valid
                                .r_eq(self.db, StdLogicValue::Logic(true))?
                                .and(self.db, self.rising_edge_clk()?)?,
                        )?,
                    )?;
                }
                PhysicalStreamDirection::Sink => {
                    self.add_statement(valid.assign(self.db, &StdLogicValue::Logic(true))?)?;
                }
            }
        }
        Ok(())
    }

    fn handshake_end(&mut self) -> Result<()> {
        match self.direction() {
            PhysicalStreamDirection::Source => {
                if let Some(ready) = *self.signal_list().ready() {
                    self.add_statement(ready.assign(self.db, &StdLogicValue::Logic(false))?)?;
                    self.add_statement(self.wait_until_rising_edge_clk()?)?;
                }
            }
            PhysicalStreamDirection::Sink => {
                if let Some(valid) = *self.signal_list().valid() {
                    self.add_statement(valid.assign(self.db, &StdLogicValue::Logic(false))?)?;
                    self.add_statement(self.wait_until_rising_edge_clk()?)?;
                }
            }
        }
        Ok(())
    }
}
