use crate::{
    common::vhdl_name::VhdlName, object::object_type::severity::SeverityLevel,
    statement::label::Label,
};

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
pub enum TestStatementKind {
    Assert(Assert),
    Report(Report),
    AssertReport(AssertReport),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TestStatement {
    label: Option<VhdlName>,
    kind: TestStatementKind,
}

impl Label for TestStatement {
    fn set_label(&mut self, label: impl Into<VhdlName>) {
        self.label = Some(label.into())
    }

    fn label(&self) -> Option<&VhdlName> {
        self.label.as_ref()
    }
}
