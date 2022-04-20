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
    pub fn declare(&self) -> Result<String> {
        if let Some(_represent_as) = self.represent_as() {
            todo!()
        } else {
            Ok(format!("{} {}", self.value(), self.unit()))
        }
    }

    pub fn new(value: impl Into<i32>, unit: TimeUnit, represent_as: Option<TimeUnit>) -> Self {
        Self {
            value: value.into(),
            unit,
            represent_as,
        }
    }

    pub fn as_unit(mut self, represent_as: TimeUnit) -> Self {
        self.represent_as = Some(represent_as);
        self
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

pub trait TimeValueFrom {
    fn fs(self) -> TimeValue;
    fn ps(self) -> TimeValue;
    fn ns(self) -> TimeValue;
    fn us(self) -> TimeValue;
    fn ms(self) -> TimeValue;
    fn sec(self) -> TimeValue;
    fn min(self) -> TimeValue;
    fn hr(self) -> TimeValue;
}

impl<T: Into<i32>> TimeValueFrom for T {
    fn fs(self) -> TimeValue {
        TimeValue::new(self, TimeUnit::Femto, None)
    }

    fn ps(self) -> TimeValue {
        TimeValue::new(self, TimeUnit::Pico, None)
    }

    fn ns(self) -> TimeValue {
        TimeValue::new(self, TimeUnit::Nano, None)
    }

    fn us(self) -> TimeValue {
        TimeValue::new(self, TimeUnit::Micro, None)
    }

    fn ms(self) -> TimeValue {
        TimeValue::new(self, TimeUnit::Milli, None)
    }

    fn sec(self) -> TimeValue {
        TimeValue::new(self, TimeUnit::Second, None)
    }

    fn min(self) -> TimeValue {
        TimeValue::new(self, TimeUnit::Minute, None)
    }

    fn hr(self) -> TimeValue {
        TimeValue::new(self, TimeUnit::Hour, None)
    }
}
