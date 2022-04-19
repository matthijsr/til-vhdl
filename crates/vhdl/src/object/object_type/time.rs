use core::fmt;
use tydi_common::error::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeUnit {
    Femto,
    Pico,
    Nano,
    Micro,
    Milli,
    Second,
    Minute,
    Hour,
}

impl fmt::Display for TimeUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimeUnit::Femto => write!(f, "fs"),
            TimeUnit::Pico => write!(f, "ps"),
            TimeUnit::Nano => write!(f, "ns"),
            TimeUnit::Micro => write!(f, "us"),
            TimeUnit::Milli => write!(f, "ms"),
            TimeUnit::Second => write!(f, "sec"),
            TimeUnit::Minute => write!(f, "min"),
            TimeUnit::Hour => write!(f, "hr"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TimeValue {
    value: i32,
    unit: TimeUnit,
    represent_as: Option<TimeUnit>,
}

impl TimeValue {
    pub fn try_string(&self) -> Result<String> {
        if let Some(represent_as) = self.represent_as() {
            todo!()
        } else {
            Ok(format!("{} {}", self.value(), self.unit()))
        }
    }

    /// Get the time value's value.
    #[must_use]
    pub fn value(&self) -> i32 {
        self.value
    }

    /// Get the time value's unit.
    #[must_use]
    pub fn unit(&self) -> TimeUnit {
        self.unit
    }

    /// Get the time value's represent as.
    #[must_use]
    pub fn represent_as(&self) -> Option<TimeUnit> {
        self.represent_as
    }
}
