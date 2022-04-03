use bitvec::prelude::BitVec;
use tydi_common::{
    error::{Error, Result, TryResult},
    numbers::NonNegative,
};

use crate::common::transfer::element_type::ElementType;

use super::transfer::physical_transfer::{IndexMode, LastMode, PhysicalTransfer, StrobeMode};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PhysicalStreamDirection {
    Source,
    Sink,
}

/// Act on the signals of a physical stream, or assert that they have certain
/// values.
///
/// Assertions can use a `message` string for additional context.
///
/// The `auto` methods automatically select whether to `act` or `assert` based
/// on the stream's `direction`.
pub trait PhysicalSignals {
    /// Returns the direction of the physical stream.
    fn direction(&self) -> PhysicalStreamDirection;

    /// Insert an arbitrary comment, provided this functionality is implemented.
    fn comment(&mut self, comment: &str);

    /// Drive all bits of the `data` signal low.
    fn act_data_default(&mut self) -> Result<()>;

    /// Assert that all bits of the `data` signal are low.
    fn assert_data_default(&mut self, message: &str) -> Result<()>;

    fn auto_data_default(&mut self, message: &str) -> Result<()> {
        match self.direction() {
            PhysicalStreamDirection::Source => self.assert_data_default(message),
            PhysicalStreamDirection::Sink => self.act_data_default(),
        }
    }

    /// Drive the `data` signal starting at the given index (relative to the
    /// Least-Significant Bit).
    ///
    /// The `comment` can optionally be implemented by the backend to provide
    /// further context on what this (segment of) the `data` signal relates to.
    fn act_data(&mut self, lsb_index: NonNegative, data: BitVec, comment: &str) -> Result<()>;

    /// Assert the value of the `data` signal starting at the given index
    /// (relative to the Least-Significant Bit).
    ///
    /// The `comment` can optionally be implemented by the backend to provide
    /// further context on what this (segment of) the `data` signal relates to.
    fn assert_data(
        &mut self,
        lsb_index: NonNegative,
        data: BitVec,
        comment: &str,
        message: &str,
    ) -> Result<()>;

    fn auto_data(
        &mut self,
        lsb_index: NonNegative,
        data: BitVec,
        comment: &str,
        message: &str,
    ) -> Result<()> {
        match self.direction() {
            PhysicalStreamDirection::Source => self.assert_data(lsb_index, data, comment, message),
            PhysicalStreamDirection::Sink => self.act_data(lsb_index, data, comment),
        }
    }

    /// Drive all bits of the `user` signal low.
    fn act_user_default(&mut self) -> Result<()>;

    /// Assert that all bits of the `user` signal are low.
    fn assert_user_default(&mut self, message: &str) -> Result<()>;

    fn auto_user_default(&mut self, message: &str) -> Result<()> {
        match self.direction() {
            PhysicalStreamDirection::Source => self.assert_user_default(message),
            PhysicalStreamDirection::Sink => self.act_user_default(),
        }
    }

    /// Drive the `user` signal starting at the given index (relative to the
    /// Least-Significant Bit).
    ///
    /// The `comment` can optionally be implemented by the backend to provide
    /// further context on what this (segment of) the `user` signal relates to.
    fn act_user(&mut self, lsb_index: NonNegative, user: BitVec, comment: &str) -> Result<()>;

    /// Assert the value of the `user` signal starting at the given index
    /// (relative to the Least-Significant Bit).
    ///
    /// The `comment` can optionally be implemented by the backend to provide
    /// further context on what this (segment of) the `user` signal relates to.
    fn assert_user(
        &mut self,
        lsb_index: NonNegative,
        user: BitVec,
        comment: &str,
        message: &str,
    ) -> Result<()>;

    fn auto_user(
        &mut self,
        lsb_index: NonNegative,
        user: BitVec,
        comment: &str,
        message: &str,
    ) -> Result<()> {
        match self.direction() {
            PhysicalStreamDirection::Source => self.assert_user(lsb_index, user, comment, message),
            PhysicalStreamDirection::Sink => self.act_user(lsb_index, user, comment),
        }
    }

    /// Drive the `stai` signal to the given value.
    fn act_stai(&mut self, stai: NonNegative) -> Result<()>;

    /// Assert that the `stai` signal contains the given value.
    fn assert_stai(&mut self, stai: NonNegative, message: &str) -> Result<()>;

    fn auto_stai(&mut self, stai: NonNegative, message: &str) -> Result<()> {
        match self.direction() {
            PhysicalStreamDirection::Source => self.assert_stai(stai, message),
            PhysicalStreamDirection::Sink => self.act_stai(stai),
        }
    }

    /// Drive the `endi` signal to the given value.
    fn act_endi(&mut self, endi: NonNegative) -> Result<()>;

    /// Assert that the `endi` signal contains the given value.
    fn assert_endi(&mut self, endi: NonNegative, message: &str) -> Result<()>;

    fn auto_endi(&mut self, endi: NonNegative, message: &str) -> Result<()> {
        match self.direction() {
            PhysicalStreamDirection::Source => self.assert_endi(endi, message),
            PhysicalStreamDirection::Sink => self.act_endi(endi),
        }
    }

    /// Drive the corresponding `strb` bit(s) high or low.
    fn act_strb(&mut self, strb: StrobeMode) -> Result<()>;

    /// Assert that the corresponding `strb` bit(s) are driven high or low.
    fn assert_strb(&mut self, strb: StrobeMode, message: &str) -> Result<()>;

    fn auto_strb(&mut self, strb: StrobeMode, message: &str) -> Result<()> {
        match self.direction() {
            PhysicalStreamDirection::Source => self.assert_strb(strb, message),
            PhysicalStreamDirection::Sink => self.act_strb(strb),
        }
    }

    /// Drive the corresponding `last` bits for the given range(s) high or low.
    fn act_last(&mut self, last: LastMode) -> Result<()>;

    /// Assert that the corresponding `last` bits for the given range(s) are
    /// driven high or low.
    fn assert_last(&mut self, last: LastMode, message: &str) -> Result<()>;

    fn auto_last(&mut self, last: LastMode, message: &str) -> Result<()> {
        match self.direction() {
            PhysicalStreamDirection::Source => self.assert_last(last, message),
            PhysicalStreamDirection::Sink => self.act_last(last),
        }
    }

    /// "Handshake" a transfer, completing it.
    ///
    /// If this is acting on a Sink, drive `valid` high and wait for `ready`
    /// during the active clock edge. (Note: The transfer must be closed using
    /// the `close()` function.)
    ///
    /// If this is asserting the correct transfer from a Source, drive `ready`
    /// high and wait for `valid` to be high during the active clock edge.
    ///
    /// A transfer is "handshaked" when when both `valid` and `ready` are
    /// asserted during the active clock edge of the clock domain common to the
    /// source and the sink.
    fn handshake(&mut self) -> Result<()>;

    /// "Handshake" a transfer, assuming `valid` was held over consecutive
    /// cycles.
    ///
    /// If this is acting on a Sink, only wait for `ready` during the active
    /// clock edge, do not drive `valid`.
    ///
    /// If this is asserting the correct transfer from a Source, drive `ready`
    /// high and wait for a cycle without waiting for `valid`. Then assert that
    /// `valid` is high.
    ///
    /// A transfer is "handshaked" when when both `valid` and `ready` are
    /// asserted during the active clock edge of the clock domain common to the
    /// source and the sink.
    fn handshake_continue(&mut self, message: &str) -> Result<()>;

    /// Open the (sequence) transfer.
    ///
    /// Wait for `valid` to be high, or drive `valid` high.
    fn handshake_start(&mut self) -> Result<()>;

    /// Close the (sequence) transfer.
    ///
    /// Drive `valid` or `ready` low, then wait for a cycle.
    fn handshake_end(&mut self) -> Result<()>;
}

// NOTE: It may be worthwile to set a limit (or allow users to test) for the
// number of cycles `ready` and/or `valid` are low. (To verify whether a Stream
// isn't being stalled indefinitely.)

/// TODO: Doc
pub trait PhysicalTransfers {
    /// Open the (sequence) transfer.
    ///
    /// Wait for `valid` to be high, or drive `valid` high.
    fn open_transfer(&mut self) -> Result<()>;

    /// Close the (sequence) transfer.
    ///
    /// Drive `valid` or `ready` low, then wait for a cycle.
    fn close_transfer(&mut self) -> Result<()>;

    /// TODO: Doc
    ///
    /// `test_staggered` intentionally closes the transfer whenever possible.
    /// (Only applies when driving a Sink.)
    fn transfer(
        &mut self,
        transfer: impl TryResult<PhysicalTransfer>,
        test_staggered: bool,
        message: &str,
    ) -> Result<()>;
}

impl<T: PhysicalSignals> PhysicalTransfers for T {
    fn open_transfer(&mut self) -> Result<()> {
        self.handshake_start()
    }

    fn close_transfer(&mut self) -> Result<()> {
        self.handshake_end()
    }

    fn transfer(
        &mut self,
        transfer: impl TryResult<PhysicalTransfer>,
        test_staggered: bool,
        message: &str,
    ) -> Result<()> {
        let transfer: PhysicalTransfer = transfer.try_result()?;

        self.comment(&format!("Test: {}", message));

        // TODO: Use type knowledge to address (subsections of) data with
        // comments for additional context.

        if let Some(data) = transfer.data() {
            for (lane, data) in data.iter().enumerate() {
                let lane = NonNegative::try_from(lane)
                    .map_err(|err| Error::BackEndError(err.to_string()))?;
                if let Some(data) = data {
                    self.auto_data(
                        lane * transfer.element_size(),
                        data.flatten(),
                        &format!("Lane {}", lane),
                        message,
                    )?;
                } else {
                    self.comment(&format!("Lane {} inactive", lane));
                }
            }
        } else {
            self.auto_data_default(message)?;
        }

        self.auto_last(transfer.last().clone(), message)?;

        self.auto_strb(transfer.strobe().clone(), message)?;

        if let IndexMode::Index(Some(stai)) = transfer.start_index() {
            self.auto_stai(*stai, message)?;
        }

        if let IndexMode::Index(Some(endi)) = transfer.end_index() {
            self.auto_endi(*endi, message)?;
        }

        match transfer.user() {
            Some(Some(ElementType::Null)) => (),
            Some(Some(user)) => self.auto_user(0, user.flatten(), "", message)?,
            Some(None) => self.comment("User inactive"),
            None => self.auto_user_default(message)?,
        }

        match self.direction() {
            PhysicalStreamDirection::Source => {
                if transfer.holds_valid() {
                    self.handshake_continue(message)?;
                } else {
                    self.handshake()?;
                }
            }
            PhysicalStreamDirection::Sink => {
                if test_staggered && !transfer.holds_valid() {
                    self.handshake()?;
                    // When `test_staggered` is true, and the transfer does not
                    // require `valid` to be held high, close the transfer after
                    // a succesful handshake.
                    //
                    // This lets users test whether a sink actually supports
                    // transfers over non-consecutive cycles.
                    self.handshake_end()?;
                } else {
                    // When `test_staggered` is false, transfer a sequence
                    // over consecutive cycles, regardless of the physical
                    // stream's properties. This saves times during simulation
                    // and makes for more compact instructions.
                    self.handshake_continue(message)?;
                }
            }
        }

        Ok(())
    }
}
