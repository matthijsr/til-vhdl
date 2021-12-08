use crate::common::{NonNegative, Positive, traits::Identify};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Origin {
    Source,
    Sink,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Width {
    /// Non-vectorized single bit.
    Scalar,
    /// Vectorized multiple bits.
    Vector(NonNegative),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Signal {
    name: String,
    origin: Origin,
    width: Width,
}

impl Identify for Signal {
    fn identifier(&self) -> &str {
        self.name.as_str()
    }
}

impl Signal {
    /// Returns a vector-style signal if the input width is Some(NonNegative)
    pub fn opt_vec(
        name: impl Into<String>,
        origin: Origin,
        width: Option<NonNegative>,
    ) -> Option<Signal> {
        width.map(|w| Signal {
            name: name.into(),
            origin,
            width: Width::Vector(w),
        })
    }

    /// Returns a vector-style signal.
    pub fn vec(name: impl Into<String>, origin: Origin, width: Positive) -> Signal {
        Signal {
            name: name.into(),
            origin,
            width: Width::Vector(width.get()),
        }
    }

    /// Returns a single bit non-vector style signal.
    pub fn bit(name: impl Into<String>, origin: Origin) -> Signal {
        Signal {
            name: name.into(),
            origin,
            width: Width::Scalar,
        }
    }

    /// Returns whether the signal is reversed w.r.t. the source
    pub fn reversed(&self) -> bool {
        self.origin == Origin::Sink
    }

    pub fn origin(&self) -> Origin {
        self.origin
    }

    pub fn width(&self) -> Width {
        self.width
    }

    pub fn with_name(&self, name: String) -> Signal {
        Signal {
            name,
            origin: self.origin,
            width: self.width,
        }
    }
}

/// Signal list for the signals in a physical stream.
///
/// A signal list can be constructed from a [`PhysicalStream`] using the
/// [`signal_list`] method or using the `From`/`Into` trait implementation.
///
/// [Reference]
///
/// [`PhysicalStream`]: ./struct.PhysicalStream.html
/// [`signal_list`]: ./struct.PhysicalStream.html#method.signal_list
/// [Reference]: https://abs-tudelft.github.io/tydi/specification/physical.html#signals
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SignalList {
    data: Option<NonNegative>,
    last: Option<NonNegative>,
    stai: Option<NonNegative>,
    endi: Option<NonNegative>,
    strb: Option<NonNegative>,
    user: Option<NonNegative>,
}

impl SignalList {
    /// Returns the valid signal.
    pub fn valid(&self) -> Signal {
        Signal {
            name: "valid".to_string(),
            origin: Origin::Source,
            width: Width::Scalar,
        }
    }

    /// Returns the ready signal.
    pub fn ready(&self) -> Signal {
        Signal {
            name: "ready".to_string(),
            origin: Origin::Sink,
            width: Width::Scalar,
        }
    }

    /// Returns the `data` signal, if applicable for this PhysicalStream.
    pub fn data(&self) -> Option<Signal> {
        Signal::opt_vec("data", Origin::Source, self.data)
    }

    /// Returns the `last` signal, if applicable for this PhysicalStream.
    pub fn last(&self) -> Option<Signal> {
        Signal::opt_vec("last", Origin::Source, self.last)
    }

    /// Returns the `stai` signal, if applicable for this PhysicalStream.
    pub fn stai(&self) -> Option<Signal> {
        Signal::opt_vec("stai", Origin::Source, self.stai)
    }

    /// Returns the `endi` signal, if applicable for this PhysicalStream.
    pub fn endi(&self) -> Option<Signal> {
        Signal::opt_vec("endi", Origin::Source, self.endi)
    }

    /// Returns the `strb` signal, if applicable for this PhysicalStream.
    pub fn strb(&self) -> Option<Signal> {
        Signal::opt_vec("strb", Origin::Source, self.strb)
    }

    /// Returns the `user` signal, if applicable for this PhysicalStream.
    pub fn user(&self) -> Option<Signal> {
        Signal::opt_vec("user", Origin::Source, self.user)
    }

    /// Returns the bit count of all combined signals in this map.
    pub fn opt_bit_count(&self) -> Option<NonNegative> {
        match self.data.unwrap_or(0)
            + self.last.unwrap_or(0)
            + self.stai.unwrap_or(0)
            + self.endi.unwrap_or(0)
            + self.strb.unwrap_or(0)
            + self.user.unwrap_or(0)
        {
            0 => None,
            x => Some(x),
        }
    }

    /// Returns the bit count of all combined signals in this map.
    pub fn bit_count(&self) -> NonNegative {
        self.opt_bit_count().unwrap_or(0)
    }
}

impl<'a> IntoIterator for &'a SignalList {
    type Item = Signal;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        [
            Some(self.valid()),
            Some(self.ready()),
            self.data(),
            self.last(),
            self.stai(),
            self.endi(),
            self.strb(),
            self.user(),
        ]
        .iter()
        .filter(|o| o.is_some())
        .map(|s| s.clone().unwrap())
        .collect::<Vec<_>>()
        .into_iter()
    }
}
