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
    numbers::{usize_to_u32, NonNegative},
    traits::{Identify, Reversed},
};
use tydi_intern::Id;
use tydi_vhdl::{
    architecture::arch_storage::db::Database,
    assignment::{bitvec::BitVecValue, Assign, FieldSelection, ObjectSelection, StdLogicValue},
    declaration::ObjectDeclaration,
    process::{
        statement::{
            condition::Condition, test_statement::TestStatement, wait::Wait, SequentialStatement,
        },
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

    fn stream_object(&self) -> &PhysicalStreamObject {
        self.process.stream_object()
    }

    /// A helper function, since handshakes often call for a `rising_edge(clk)`
    fn rising_edge_clk(&self) -> Result<Edge> {
        Edge::rising_edge(self.db, self.process.stream_object().clock())
    }

    /// A helper function, since handshakes often call for a
    /// `wait until rising_edge(clk)`
    fn wait_until_rising_edge_clk(&mut self) -> Result<()> {
        self.add_statement(Wait::wait().until_relation(self.db, self.rising_edge_clk()?)?)
    }

    fn wait_until_rising_edge_clk_and_high(&mut self, sig: Id<ObjectDeclaration>) -> Result<()> {
        self.add_statement(Wait::wait().until_relation(
            self.db,
            self.rising_edge_clk()?.and(self.db, self.is_high(sig)?)?,
        )?)
    }

    fn signal_list(&self) -> &SignalList<Id<ObjectDeclaration>> {
        self.stream_object().signal_list()
    }

    fn add_statement(&mut self, statement: impl TryResult<SequentialStatement>) -> Result<()> {
        self.process.process.add_statement(self.db, statement)
    }

    fn is_high(&self, sig: Id<ObjectDeclaration>) -> Result<Relation> {
        Ok(sig.r_eq(self.db, StdLogicValue::Logic(true))?.into())
    }

    fn set_high(&mut self, sig: Id<ObjectDeclaration>) -> Result<()> {
        self.add_statement(sig.assign(self.db, &StdLogicValue::Logic(true))?)
    }

    fn set_low(&mut self, sig: Id<ObjectDeclaration>) -> Result<()> {
        self.add_statement(sig.assign(self.db, &StdLogicValue::Logic(false))?)
    }

    fn assert_eq_report(
        &mut self,
        left: impl TryResult<Relation>,
        right: impl TryResult<Relation>,
        message: &str,
    ) -> Result<()> {
        self.add_statement(TestStatement::assert_report(
            Condition::relation(self.db, left.r_eq(self.db, right)?)?,
            message,
        ))
    }

    fn last_for_lane(
        &self,
        lane: u32,
        last: &Option<std::ops::Range<u32>>,
    ) -> Result<(ObjectSelection, BitVecValue)> {
        let left = self.stream_object().get_last(lane)?;
        let right = if let Some(transfer_last) = last {
            let mut assign_vec = vec![];
            for dim in (0..self.stream_object().dimensionality()).rev() {
                if dim >= transfer_last.end && dim <= transfer_last.start {
                    assign_vec.push(StdLogicValue::Logic(true));
                } else {
                    assign_vec.push(StdLogicValue::Logic(false));
                }
            }
            BitVecValue::Full(assign_vec)
        } else {
            BitVecValue::Others(StdLogicValue::Logic(false))
        };
        Ok((left, right))
    }

    fn last_and_val(&self, last: &LastMode) -> Result<Vec<(ObjectSelection, BitVecValue)>> {
        match last {
            LastMode::None => Ok(vec![]),
            LastMode::Transfer(transfer_last) => Ok(vec![self.last_for_lane(0, transfer_last)?]),
            LastMode::Lane(last_lanes) => last_lanes
                .iter()
                .enumerate()
                .map(|(lane, last)| Ok(self.last_for_lane(usize_to_u32(lane)?, last)?))
                .collect(),
        }
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

    fn act_last(&mut self, last: &LastMode) -> Result<()> {
        for (sig, val) in self.last_and_val(last)? {
            self.add_statement(sig.assign(self.db, &val)?)?;
        }
        Ok(())
    }

    fn assert_last(&mut self, last: &LastMode, message: &str) -> Result<()> {
        for (sig, val) in self.last_and_val(last)? {
            self.assert_eq_report(sig, val, message)?
        }
        Ok(())
    }

    fn handshake(&mut self) -> Result<()> {
        if let (Some(ready), Some(valid)) =
            (*self.signal_list().ready(), *self.signal_list().valid())
        {
            match self.direction() {
                PhysicalStreamDirection::Source => {
                    self.set_high(ready)?;
                    self.wait_until_rising_edge_clk_and_high(valid)?;
                }
                PhysicalStreamDirection::Sink => {
                    self.set_high(valid)?;
                    self.wait_until_rising_edge_clk_and_high(ready)?;
                }
            }
        }
        Ok(())
    }

    fn handshake_continue(&mut self, message: &str) -> Result<()> {
        if let Some(ready) = *self.signal_list().ready() {
            match self.direction() {
                PhysicalStreamDirection::Source => {
                    if let Some(valid) = *self.signal_list().valid() {
                        self.set_high(ready)?;
                        self.wait_until_rising_edge_clk()?;
                        self.assert_eq_report(valid, StdLogicValue::Logic(true), message)?;
                    }
                }
                PhysicalStreamDirection::Sink => {
                    self.wait_until_rising_edge_clk_and_high(ready)?;
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
                    self.wait_until_rising_edge_clk_and_high(valid)?;
                }
                PhysicalStreamDirection::Sink => {
                    self.set_high(valid)?;
                }
            }
        }
        Ok(())
    }

    fn handshake_end(&mut self) -> Result<()> {
        match self.direction() {
            PhysicalStreamDirection::Source => {
                if let Some(ready) = *self.signal_list().ready() {
                    self.set_low(ready)?;
                    self.wait_until_rising_edge_clk()?;
                }
            }
            PhysicalStreamDirection::Sink => {
                if let Some(valid) = *self.signal_list().valid() {
                    self.set_low(valid)?;
                    self.wait_until_rising_edge_clk()?;
                }
            }
        }
        Ok(())
    }
}
