use tydi_common::error::Result;

use crate::{
    architecture::arch_storage::Arch,
    common::vhdl_name::VhdlName,
    declaration::DeclareWithIndent,
    object::object_type::severity::{HasSeverity, SetSeverity, SeverityLevel},
    statement::label::Label,
};

use super::condition::Condition;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Assert {
    condition: Condition,
    /// Severity level, default severity is usually implicitly assumed to be `note`.
    severity: Option<SeverityLevel>,
}

impl Assert {
    /// Get a reference to the assert's condition.
    #[must_use]
    pub fn condition(&self) -> &Condition {
        &self.condition
    }
}

impl HasSeverity for Assert {
    fn severity(&self) -> Option<&SeverityLevel> {
        self.severity.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Report {
    message: String,
    /// Severity level, default severity is usually implicitly assumed to be `note`.
    severity: Option<SeverityLevel>,
}

impl Report {
    /// Get a reference to the report's message.
    #[must_use]
    pub fn message(&self) -> &str {
        self.message.as_ref()
    }
}

impl HasSeverity for Report {
    fn severity(&self) -> Option<&SeverityLevel> {
        self.severity.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AssertReport {
    condition: Condition,
    message: String,
    /// Severity level, default severity is usually implicitly assumed to be `note`.
    severity: Option<SeverityLevel>,
}

impl AssertReport {
    /// Get a reference to the assert report's condition.
    #[must_use]
    pub fn condition(&self) -> &Condition {
        &self.condition
    }

    /// Get a reference to the assert report's message.
    #[must_use]
    pub fn message(&self) -> &str {
        self.message.as_ref()
    }
}

impl HasSeverity for AssertReport {
    fn severity(&self) -> Option<&SeverityLevel> {
        self.severity.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TestStatementKind {
    Assert(Assert),
    Report(Report),
    AssertReport(AssertReport),
}

impl HasSeverity for TestStatementKind {
    fn severity(&self) -> Option<&SeverityLevel> {
        match self {
            TestStatementKind::Assert(a) => a.severity(),
            TestStatementKind::Report(r) => r.severity(),
            TestStatementKind::AssertReport(ar) => ar.severity(),
        }
    }
}

impl SetSeverity for TestStatementKind {
    fn set_severity(&mut self, severity: impl Into<SeverityLevel>) {
        match self {
            TestStatementKind::Assert(a) => a.severity = Some(severity.into()),
            TestStatementKind::Report(r) => r.severity = Some(severity.into()),
            TestStatementKind::AssertReport(ar) => ar.severity = Some(severity.into()),
        }
    }
}

impl DeclareWithIndent for TestStatementKind {
    fn declare_with_indent(&self, db: &dyn Arch, indent_style: &str) -> Result<String> {
        let mut result = match self {
            TestStatementKind::Assert(a) => format!(
                "assert {}",
                a.condition().declare_with_indent(db, indent_style)?
            ),
            TestStatementKind::Report(r) => {
                format!("report \"{}\"", r.message())
            }
            TestStatementKind::AssertReport(ar) => format!(
                "assert {} report \"{}\"",
                ar.condition().declare_with_indent(db, indent_style)?,
                ar.message(),
            ),
        };

        if let Some(severity) = self.severity() {
            result.push_str(&format!(" severity {}", severity))
        }

        Ok(result)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TestStatement {
    label: Option<VhdlName>,
    kind: TestStatementKind,
}

impl TestStatement {
    /// Get a reference to the test statement's kind.
    #[must_use]
    pub fn kind(&self) -> &TestStatementKind {
        &self.kind
    }

    pub fn report(message: impl Into<String>) -> Self {
        Self {
            label: None,
            kind: TestStatementKind::Report(Report {
                message: message.into(),
                severity: None,
            }),
        }
    }

    pub fn assert(condition: impl Into<Condition>) -> Self {
        Self {
            label: None,
            kind: TestStatementKind::Assert(Assert {
                condition: condition.into(),
                severity: None,
            }),
        }
    }

    pub fn assert_report(condition: impl Into<Condition>, message: impl Into<String>) -> Self {
        Self {
            label: None,
            kind: TestStatementKind::AssertReport(AssertReport {
                condition: condition.into(),
                message: message.into(),
                severity: None,
            }),
        }
    }
}

impl HasSeverity for TestStatement {
    fn severity(&self) -> Option<&SeverityLevel> {
        self.kind().severity()
    }
}

impl SetSeverity for TestStatement {
    fn set_severity(&mut self, severity: impl Into<SeverityLevel>) {
        self.kind.set_severity(severity)
    }
}

impl DeclareWithIndent for TestStatement {
    fn declare_with_indent(&self, db: &dyn Arch, indent_style: &str) -> Result<String> {
        self.kind().declare_with_indent(db, indent_style)
    }
}

impl Label for TestStatement {
    fn set_label(&mut self, label: impl Into<VhdlName>) {
        self.label = Some(label.into())
    }

    fn label(&self) -> Option<&VhdlName> {
        self.label.as_ref()
    }
}
