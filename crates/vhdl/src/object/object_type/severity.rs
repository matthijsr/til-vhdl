use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Uses std.standard.severity_level
pub enum SeverityLevel {
    Note,
    Warning,
    Error,
    Failure,
}

impl fmt::Display for SeverityLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SeverityLevel::Note => write!(f, "note"),
            SeverityLevel::Warning => write!(f, "warning"),
            SeverityLevel::Error => write!(f, "error"),
            SeverityLevel::Failure => write!(f, "failure"),
        }
    }
}

pub trait HasSeverity {
    fn severity(&self) -> Option<&SeverityLevel>;
}

pub trait SetSeverity: Sized {
    fn with_severity(mut self, severity: impl Into<SeverityLevel>) -> Self {
        self.set_severity(severity);
        self
    }
    fn set_severity(&mut self, severity: impl Into<SeverityLevel>);
}
