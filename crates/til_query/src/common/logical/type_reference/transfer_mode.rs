use tydi_common::traits::Reversed;

use crate::{
    common::logical::logicaltype::stream::Direction, ir::physical_properties::InterfaceDirection,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransferMode {
    /// This Stream acts as a Source.
    ///
    /// It defines the state of all but its `ready` signal. (Comprising
    /// `valid`, `data`, etc.)
    Source,
    /// This Stream acts as a Sink.
    ///
    /// It defines the state of its `ready` signal.
    Sink,
}

impl TransferMode {
    /// The transfer mode of a Stream, assuming the interface is addressed by
    /// another, external component.
    pub fn new(interface_direction: InterfaceDirection) -> Self {
        match interface_direction {
            InterfaceDirection::Out => Self::Source,
            InterfaceDirection::In => Self::Sink,
        }
    }

    /// If this Stream is addressed within the Streamlet, its transfer mode is
    /// effectively reversed.
    pub fn internal_mode(&self) -> Self {
        self.reversed()
    }

    /// The transfer mode is reversed relative to its interface or parent
    /// depending on its `direction` property.
    pub fn for_direction(&self, direction: Direction) -> Self {
        if direction == Direction::Reverse {
            self.reversed()
        } else {
            *self
        }
    }
}

impl From<InterfaceDirection> for TransferMode {
    fn from(interface_direction: InterfaceDirection) -> Self {
        Self::new(interface_direction)
    }
}

impl From<&InterfaceDirection> for TransferMode {
    fn from(interface_direction: &InterfaceDirection) -> Self {
        Self::new(*interface_direction)
    }
}

impl Reversed for TransferMode {
    fn reversed(&self) -> Self {
        match self {
            TransferMode::Source => Self::Sink,
            TransferMode::Sink => Self::Source,
        }
    }
}
