use tydi_common::{
    error::{Error, Result, TryResult},
    numbers::NonNegative,
    traits::Reversed,
};

use crate::common::transfer::element_type::ElementType;

use super::transfer::physical_transfer::{IndexMode, LastMode, PhysicalTransfer, StrobeMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PhysicalStreamDirection {
    Source,
    Sink,
}

impl Reversed for PhysicalStreamDirection {
    fn reversed(&self) -> Self {
        match self {
            PhysicalStreamDirection::Source => PhysicalStreamDirection::Sink,
            PhysicalStreamDirection::Sink => PhysicalStreamDirection::Source,
        }
    }
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

    /// Drive the `data` signal starting at the given element lane
    fn act_data(&mut self, element_lane: NonNegative, data: &ElementType) -> Result<()>;

    /// Assert the value of the `data` signal starting at the given element lane
    fn assert_data(
        &mut self,
        element_lane: NonNegative,
        data: &ElementType,
        message: &str,
    ) -> Result<()>;

    fn auto_data(
        &mut self,
        element_lane: NonNegative,
        data: &ElementType,
        message: &str,
    ) -> Result<()> {
        match self.direction() {
            PhysicalStreamDirection::Source => self.assert_data(element_lane, data, message),
            PhysicalStreamDirection::Sink => self.act_data(element_lane, data),
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

    /// Drive the `user` signal
    fn act_user(&mut self, user: &ElementType) -> Result<()>;

    /// Assert the value of the `user` signal
    fn assert_user(&mut self, user: &ElementType, message: &str) -> Result<()>;

    fn auto_user(&mut self, user: &ElementType, message: &str) -> Result<()> {
        match self.direction() {
            PhysicalStreamDirection::Source => self.assert_user(user, message),
            PhysicalStreamDirection::Sink => self.act_user(user),
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
    fn act_last(&mut self, last: &LastMode) -> Result<()>;

    /// Assert that the corresponding `last` bits for the given range(s) are
    /// driven high or low.
    fn assert_last(&mut self, last: &LastMode, message: &str) -> Result<()>;

    fn auto_last(&mut self, last: &LastMode, message: &str) -> Result<()> {
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
    /// Wait for `valid` to be high and an active clock edge,
    /// or drive `valid` high.
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
    ///
    /// `context` refers to the context of the physical stream, such as its
    /// interface name.
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

        self.comment(message);

        if let Some(data) = transfer.data() {
            for (lane, data) in data.iter().enumerate() {
                let lane = NonNegative::try_from(lane)
                    .map_err(|err| Error::BackEndError(err.to_string()))?;
                match data {
                    Some(data) => {
                        self.auto_data(lane, data, message)?;
                    }
                    None => self.comment(&format!("Lane {} inactive", lane)),
                }
            }
        } else {
            self.auto_data_default(message)?;
        }

        self.auto_last(transfer.last(), message)?;

        self.auto_strb(transfer.strobe().clone(), message)?;

        if let IndexMode::Index(Some(stai)) = transfer.start_index() {
            self.auto_stai(*stai, message)?;
        }

        if let IndexMode::Index(Some(endi)) = transfer.end_index() {
            self.auto_endi(*endi, message)?;
        }

        match transfer.user() {
            Some(Some(user)) => self.auto_user(user, message)?,
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

#[cfg(test)]
mod tests {
    use tydi_common::numbers::Positive;

    use crate::common::physical::complexity::Complexity;

    use super::*;

    pub struct TestSignaller {
        dir: PhysicalStreamDirection,
        result: String,
    }

    impl TestSignaller {
        pub fn sink() -> Self {
            Self {
                dir: PhysicalStreamDirection::Sink,
                result: String::new(),
            }
        }

        pub fn source() -> Self {
            Self {
                dir: PhysicalStreamDirection::Source,
                result: String::new(),
            }
        }

        pub fn result(&self) -> &str {
            &self.result
        }
    }

    impl PhysicalSignals for TestSignaller {
        fn direction(&self) -> PhysicalStreamDirection {
            self.dir
        }

        fn comment(&mut self, comment: &str) {
            self.result.push_str(&format!("// {}\n", comment))
        }

        fn act_data_default(&mut self) -> Result<()> {
            self.result.push_str("act_data_default\n");
            Ok(())
        }

        fn assert_data_default(&mut self, message: &str) -> Result<()> {
            self.result
                .push_str(&format!("assert_data_default: {}\n", message));
            Ok(())
        }

        fn act_data(&mut self, element_lane: NonNegative, data: &ElementType) -> Result<()> {
            self.result
                .push_str(&format!("act_data({}, {})\n", element_lane, data));
            Ok(())
        }

        fn assert_data(
            &mut self,
            element_lane: NonNegative,
            data: &ElementType,
            message: &str,
        ) -> Result<()> {
            self.result.push_str(&format!(
                "assert_data({}, {}): {}\n",
                element_lane, data, message
            ));
            Ok(())
        }

        fn act_user_default(&mut self) -> Result<()> {
            self.result.push_str("act_user_default\n");
            Ok(())
        }

        fn assert_user_default(&mut self, message: &str) -> Result<()> {
            self.result
                .push_str(&format!("assert_user_default: {}\n", message));
            Ok(())
        }

        fn act_user(&mut self, user: &ElementType) -> Result<()> {
            self.result.push_str(&format!("act_user({})\n", user));
            Ok(())
        }

        fn assert_user(&mut self, user: &ElementType, message: &str) -> Result<()> {
            self.result
                .push_str(&format!("assert_user({}): {}\n", user, message));
            Ok(())
        }

        fn act_stai(&mut self, stai: NonNegative) -> Result<()> {
            self.result.push_str(&format!("act_stai({})\n", stai));
            Ok(())
        }

        fn assert_stai(&mut self, stai: NonNegative, message: &str) -> Result<()> {
            self.result
                .push_str(&format!("assert_stai({}): {}\n", stai, message));
            Ok(())
        }

        fn act_endi(&mut self, endi: NonNegative) -> Result<()> {
            self.result.push_str(&format!("act_endi({})\n", endi));
            Ok(())
        }

        fn assert_endi(&mut self, endi: NonNegative, message: &str) -> Result<()> {
            self.result
                .push_str(&format!("assert_endi({}): {}\n", endi, message));
            Ok(())
        }

        fn act_strb(&mut self, strb: StrobeMode) -> Result<()> {
            self.result.push_str(&format!("act_strb({})\n", strb));
            Ok(())
        }

        fn assert_strb(&mut self, strb: StrobeMode, message: &str) -> Result<()> {
            self.result
                .push_str(&format!("assert_strb({}): {}\n", strb, message));
            Ok(())
        }

        fn act_last(&mut self, last: &LastMode) -> Result<()> {
            self.result.push_str(&format!("act_last({})\n", last));
            Ok(())
        }

        fn assert_last(&mut self, last: &LastMode, message: &str) -> Result<()> {
            self.result
                .push_str(&format!("assert_last({}): {}\n", last, message));
            Ok(())
        }

        fn handshake(&mut self) -> Result<()> {
            self.result.push_str("handshake\n");
            Ok(())
        }

        fn handshake_continue(&mut self, message: &str) -> Result<()> {
            self.result
                .push_str(&format!("handshake_continue: {}\n", message));
            Ok(())
        }

        fn handshake_start(&mut self) -> Result<()> {
            self.result.push_str("handshake_start\n");
            Ok(())
        }

        fn handshake_end(&mut self) -> Result<()> {
            self.result.push_str("handshake_end\n");
            Ok(())
        }
    }

    #[test]
    fn test_transfer() -> Result<()> {
        let transfer =
            PhysicalTransfer::new(Complexity::new_major(8), Positive::new(3).unwrap(), 2, 3, 3)
                .with_logical_transfer(([Some("11"), None, Some("11")], "101"))?;

        let mut sink = TestSignaller::sink();
        sink.open_transfer()?;
        sink.transfer(transfer.clone(), false, "test message")?;
        sink.close_transfer()?;

        assert_eq!(
            r#"handshake_start
// test message
act_data(0, Bits([1, 1]))
// Lane 1 inactive
act_data(2, Bits([1, 1]))
act_last(Lane(None, None, None))
act_strb(Lane("101"))
act_user(Bits([1, 0, 1]))
handshake_continue: test message
handshake_end
"#,
            sink.result()
        );

        let mut source = TestSignaller::source();
        source.open_transfer()?;
        source.transfer(transfer.clone(), false, "test message")?;
        source.close_transfer()?;

        assert_eq!(
            r#"handshake_start
// test message
assert_data(0, Bits([1, 1])): test message
// Lane 1 inactive
assert_data(2, Bits([1, 1])): test message
assert_last(Lane(None, None, None)): test message
assert_strb(Lane("101")): test message
assert_user(Bits([1, 0, 1])): test message
handshake
handshake_end
"#,
            source.result()
        );

        Ok(())
    }
}
