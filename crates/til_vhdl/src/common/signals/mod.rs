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
    error::{Error, Result, TryResult},
    name::{PathName, PathNameSelf},
    numbers::{i32_to_u32, usize_to_u32, NonNegative},
    traits::{Identify, Reversed},
    util::log2_ceil,
};
use tydi_intern::Id;
use tydi_vhdl::{
    architecture::arch_storage::{db::Database, Arch},
    assignment::{
        bitvec::{BitVecValue, WidthSource},
        Assign, ObjectSelection, StdLogicValue, ValueAssignment,
    },
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
        self.add_statement(sig.assign(self.db, StdLogicValue::Logic(true))?)
    }

    fn set_low(&mut self, sig: Id<ObjectDeclaration>) -> Result<()> {
        self.add_statement(sig.assign(self.db, StdLogicValue::Logic(false))?)
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
        db: &dyn Arch,
        lane: u32,
        last: &Option<std::ops::Range<u32>>,
    ) -> Result<(ObjectSelection, ValueAssignment)> {
        let left = self.stream_object().get_last(db, lane)?;
        if let Some(ValueAssignment::Integer(i)) =
            self.stream_object().dimensionality().try_eval()?
        {
            let right = if let Some(transfer_last) = last {
                let mut assign_vec = vec![];
                for dim in (0..i32_to_u32(i)?).rev() {
                    if dim <= transfer_last.end && dim >= transfer_last.start {
                        assign_vec.push(StdLogicValue::Logic(true));
                    } else {
                        assign_vec.push(StdLogicValue::Logic(false));
                    }
                }
                if assign_vec.len() == 1 {
                    assign_vec[0].into()
                } else {
                    BitVecValue::Full(assign_vec).into()
                }
            } else {
                if i == 1 {
                    StdLogicValue::Logic(false).into()
                } else {
                    BitVecValue::Others(StdLogicValue::Logic(false)).into()
                }
            };
            Ok((left, right))
        } else {
            todo!()
        }
    }

    fn last_and_val(
        &self,
        db: &dyn Arch,
        last: &LastMode,
    ) -> Result<Vec<(ObjectSelection, ValueAssignment)>> {
        match last {
            LastMode::None => Ok(vec![]),
            LastMode::Transfer(transfer_last) => {
                Ok(vec![self.last_for_lane(db, 0, transfer_last)?])
            }
            LastMode::Lane(last_lanes) => last_lanes
                .iter()
                .enumerate()
                .map(|(lane, last)| Ok(self.last_for_lane(db, usize_to_u32(lane)?, last)?))
                .collect(),
        }
    }

    fn fit_index(&self, idx: NonNegative) -> Result<ValueAssignment> {
        let lanes = self.stream_object().element_lanes().get();
        if lanes > 2 {
            if idx >= lanes {
                Err(Error::InvalidArgument(format!(
                    "Cannot assign start/end-index {} to {}, as it only has {} element lanes.",
                    idx,
                    self.process.path_name(),
                    lanes
                )))
            } else {
                Ok(BitVecValue::Unsigned(
                    idx,
                    WidthSource::Constant(log2_ceil(self.stream_object().element_lanes().clone())),
                )
                .into())
            }
        } else if lanes > 1 {
            if idx > 1 {
                Err(Error::InvalidArgument(format!(
                    "Cannot assign start/end-index {} to {}, as it only has two element lanes.",
                    idx,
                    self.process.path_name()
                )))
            } else if idx == 1 {
                Ok(StdLogicValue::Logic(true).into())
            } else {
                Ok(StdLogicValue::Logic(false).into())
            }
        } else {
            Err(Error::InvalidArgument(format!(
                "Cannot assign a start/end-index to {}, as it only has one element lane.",
                self.process.path_name()
            )))
        }
    }

    fn default_data(&self) -> Result<ValueAssignment> {
        if self.stream_object().data_element_size() == 0 {
            Err(Error::InvalidArgument(format!(
                "Cannot produce a default data signal assignment for {}, as it has no data signal.",
                self.process.path_name()
            )))
        } else if self.stream_object().data_element_size()
            * self.stream_object().element_lanes().get()
            == 1
        {
            Ok(StdLogicValue::Logic(false).into())
        } else {
            Ok(BitVecValue::Others(StdLogicValue::Logic(false)).into())
        }
    }

    fn default_user(&self) -> Result<ValueAssignment> {
        if self.stream_object().user_size() == 0 {
            Err(Error::InvalidArgument(format!(
                "Cannot produce a default user signal assignment for {}, as it has no user signal.",
                self.process.path_name()
            )))
        } else if self.stream_object().user_size() == 1 {
            Ok(StdLogicValue::Logic(false).into())
        } else {
            Ok(BitVecValue::Others(StdLogicValue::Logic(false)).into())
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
        if let Some(data_sig) = *self.signal_list().data() {
            self.add_statement(data_sig.assign(self.db, self.default_data()?)?)?;
        }
        Ok(())
    }

    fn assert_data_default(&mut self, message: &str) -> Result<()> {
        if let Some(data_sig) = *self.signal_list().data() {
            self.assert_eq_report(data_sig, self.default_data()?, message)?;
        }
        Ok(())
    }

    fn act_data(&mut self, element_lane: NonNegative, data: &ElementType) -> Result<()> {
        let (data_sig, el_data) = self
            .stream_object()
            .get_element_lane_for(element_lane, data)?;
        self.add_statement(data_sig.assign(self.db, el_data)?)
    }

    fn assert_data(
        &mut self,
        element_lane: NonNegative,
        data: &ElementType,
        message: &str,
    ) -> Result<()> {
        let (data_sig, el_data) = self
            .stream_object()
            .get_element_lane_for(element_lane, data)?;
        self.assert_eq_report(data_sig, el_data, message)
    }

    fn act_user_default(&mut self) -> Result<()> {
        if let Some(user_sig) = *self.signal_list().user() {
            self.add_statement(user_sig.assign(self.db, self.default_user()?)?)?;
        }
        Ok(())
    }

    fn assert_user_default(&mut self, message: &str) -> Result<()> {
        if let Some(user_sig) = *self.signal_list().user() {
            self.assert_eq_report(user_sig, self.default_user()?, message)?;
        }
        Ok(())
    }

    fn act_user(&mut self, user: &ElementType) -> Result<()> {
        let (user_sig, user_data) = self.stream_object().get_user_for(user)?;
        self.add_statement(user_sig.assign(self.db, user_data)?)
    }

    fn assert_user(&mut self, user: &ElementType, message: &str) -> Result<()> {
        let (user_sig, user_data) = self.stream_object().get_user_for(user)?;
        self.assert_eq_report(user_sig, user_data, message)
    }

    fn act_stai(&mut self, stai: NonNegative) -> Result<()> {
        if let Some(stai_sig) = *self.signal_list().stai() {
            self.add_statement(stai_sig.assign(self.db, self.fit_index(stai)?)?)
        } else {
            Err(Error::InvalidArgument(format!(
                "{} does not have an stai signal",
                self.process.path_name()
            )))
        }
    }

    fn assert_stai(&mut self, stai: NonNegative, message: &str) -> Result<()> {
        if let Some(stai_sig) = *self.signal_list().stai() {
            self.assert_eq_report(stai_sig, self.fit_index(stai)?, message)
        } else {
            Err(Error::InvalidArgument(format!(
                "{} does not have an stai signal",
                self.process.path_name()
            )))
        }
    }

    fn act_endi(&mut self, endi: NonNegative) -> Result<()> {
        if let Some(endi_sig) = *self.signal_list().endi() {
            self.add_statement(endi_sig.assign(self.db, self.fit_index(endi)?)?)
        } else {
            Err(Error::InvalidArgument(format!(
                "{} does not have an endi signal",
                self.process.path_name()
            )))
        }
    }

    fn assert_endi(&mut self, endi: NonNegative, message: &str) -> Result<()> {
        if let Some(endi_sig) = *self.signal_list().endi() {
            self.assert_eq_report(endi_sig, self.fit_index(endi)?, message)
        } else {
            Err(Error::InvalidArgument(format!(
                "{} does not have an endi signal",
                self.process.path_name()
            )))
        }
    }

    fn act_strb(&mut self, strb: &StrobeMode) -> Result<()> {
        if let Some(strb_sig) = *self.signal_list().strb() {
            match strb {
                StrobeMode::None => Err(Error::InvalidArgument(format!(
                    "Cannot assign {} to {}, as it does have a strb signal.",
                    strb,
                    self.process.path_name()
                ))),
                StrobeMode::Transfer(transfer_strb) => self.add_statement(strb_sig.assign(
                    self.db,
                    BitVecValue::Others(StdLogicValue::Logic(*transfer_strb)),
                )?),
                StrobeMode::Lane(lane_vals) => self.add_statement(
                    strb_sig.assign(self.db, BitVecValue::from(lane_vals.iter().map(|x| *x)))?,
                ),
            }
        } else {
            match strb {
                StrobeMode::None => Ok(()),
                _ => Err(Error::InvalidArgument(format!(
                    "Cannot assign {} to {}, as it does not have a strb signal.",
                    strb,
                    self.process.path_name()
                ))),
            }
        }
    }

    fn assert_strb(&mut self, strb: &StrobeMode, message: &str) -> Result<()> {
        if let Some(strb_sig) = *self.signal_list().strb() {
            match strb {
                StrobeMode::None => Err(Error::InvalidArgument(format!(
                    "Cannot assert {} for {}, as it does have a strb signal.",
                    strb,
                    self.process.path_name()
                ))),
                StrobeMode::Transfer(transfer_strb) => self.assert_eq_report(
                    strb_sig,
                    BitVecValue::Others(StdLogicValue::Logic(*transfer_strb)),
                    message,
                ),
                StrobeMode::Lane(lane_vals) => self.assert_eq_report(
                    strb_sig,
                    BitVecValue::from(lane_vals.iter().map(|x| *x)),
                    message,
                ),
            }
        } else {
            match strb {
                StrobeMode::None => Ok(()),
                _ => Err(Error::InvalidArgument(format!(
                    "Cannot assert {} for {}, as it does not have a strb signal.",
                    strb,
                    self.process.path_name()
                ))),
            }
        }
    }

    fn act_last(&mut self, last: &LastMode) -> Result<()> {
        for (sig, val) in self.last_and_val(self.db, last)? {
            self.add_statement(sig.assign(self.db, val)?)?;
        }
        Ok(())
    }

    fn assert_last(&mut self, last: &LastMode, message: &str) -> Result<()> {
        for (sig, val) in self.last_and_val(self.db, last)? {
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
