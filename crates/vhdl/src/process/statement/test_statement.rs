use crate::object::object_type::severity::SeverityLevel;

use super::condition::Condition;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Assert {
    condition: Condition,
    /// Severity level, default severity is usually implicitly assumed to be `note`.
    severity: Option<SeverityLevel>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Report {
    message: String,
    /// Severity level, default severity is usually implicitly assumed to be `note`.
    severity: Option<SeverityLevel>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AssertReport {
    condition: Condition,
    message: String,
    /// Severity level, default severity is usually implicitly assumed to be `note`.
    severity: Option<SeverityLevel>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TestStatement {
    Assert(Assert),
    Report(Report),
    AssertReport(AssertReport),
}
