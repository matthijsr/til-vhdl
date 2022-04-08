use core::fmt;
use std::str::FromStr;

use tydi_common::error::{Error, Result};

/// Direction of a stream.
///
/// [Reference]
///
/// [Reference]: https://abs-tudelft.github.io/tydi/specification/logical.html#stream
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum StreamDirection {
    /// Forward indicates that the child stream flows in the same direction as
    /// its parent, complementing the data of its parent in some way.
    Forward,
    /// Reverse indicates that the child stream acts as a response channel for
    /// the parent stream. If there is no parent stream, Forward indicates that
    /// the stream flows in the natural source to sink direction of the logical
    /// stream, while Reverse indicates a control channel in the opposite
    /// direction. The latter may occur for instance when doing random read
    /// access to a memory; the first stream carrying the read commands then
    /// flows in the sink to source direction.
    Reverse,
}

impl Default for StreamDirection {
    fn default() -> Self {
        StreamDirection::Forward
    }
}

impl FromStr for StreamDirection {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self> {
        match input {
            "Forward" => Ok(StreamDirection::Forward),
            "Reverse" => Ok(StreamDirection::Reverse),
            _ => Err(Error::InvalidArgument(format!(
                "{} is not a valid Direction",
                input
            ))),
        }
    }
}

impl fmt::Display for StreamDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StreamDirection::Forward => write!(f, "Forward"),
            StreamDirection::Reverse => write!(f, "Reverse"),
        }
    }
}
