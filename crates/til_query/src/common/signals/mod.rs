use bitvec::prelude::BitVec;
use tydi_common::{error::Result, numbers::NonNegative};

use super::transfer::physical_transfer::{LastMode, StrobeMode};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PhysicalStreamDirection {
    Source,
    Sink,
}

/// Act on the signals of a physical stream, or assert that they have certain
/// values.
///
/// Whether to act or assert should be automatically determined for transfers
/// based on the direction of the physical stream.
///
/// Assertions can use a `message` string for additional context.
pub trait PhysicalSignals {
    /// Returns the direction of the physical stream.
    fn direction(&self) -> PhysicalStreamDirection;

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

    /// Drive the `stai` signal to the given value.
    fn act_stai(&mut self, stai: NonNegative) -> Result<()>;

    /// Assert that the `stai` signal contains the given value.
    fn assert_stai(&mut self, stai: NonNegative, message: &str) -> Result<()>;

    /// Drive the `endi` signal to the given value.
    fn act_endi(&mut self, endi: NonNegative) -> Result<()>;

    /// Assert that the `endi` signal contains the given value.
    fn assert_endi(&mut self, endi: NonNegative, message: &str) -> Result<()>;

    /// Drive the corresponding `strb` bit(s) high or low.
    fn act_strb(&mut self, strb: StrobeMode) -> Result<()>;

    /// Assert that the corresponding `strb` bit(s) are driven high or low.
    fn assert_strb(&mut self, strb: StrobeMode, message: &str) -> Result<()>;

    /// Drive the corresponding `last` bits for the given range(s) high or low.
    fn act_last(&mut self, last: LastMode) -> Result<()>;

    /// Assert that the corresponding `last` bits for the given range(s) are
    /// driven high or low.
    fn assert_last(&mut self, last: LastMode, message: &str) -> Result<()>;

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

    /// Close the transfer.
    ///
    /// Drive `valid` or `ready` low, then wait for a cycle.
    fn close(&mut self) -> Result<()>;
}

// NOTE: It may be worthwile to set a limit (or allow users to test) for the
// number of cycles `ready` and/or `valid` are low. (To verify whether a Stream
// isn't being stalled indefinitely.)
