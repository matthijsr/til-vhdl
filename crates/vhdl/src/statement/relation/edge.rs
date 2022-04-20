use core::fmt;

use tydi_common::error::{Error, Result, TryResult};

use crate::{
    architecture::arch_storage::Arch, assignment::ObjectAssignment, declaration::DeclareWithIndent,
    object::object_type::ObjectType,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeKind {
    Rising,
    Falling,
}

impl fmt::Display for EdgeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EdgeKind::Rising => write!(f, "rising_edge"),
            EdgeKind::Falling => write!(f, "falling_edge"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Edge {
    subject: ObjectAssignment,
    kind: EdgeKind,
}

impl Edge {
    /// Get the edge's subject.
    #[must_use]
    pub fn subject(&self) -> &ObjectAssignment {
        &self.subject
    }

    /// Get the edge's kind.
    #[must_use]
    pub fn kind(&self) -> EdgeKind {
        self.kind
    }

    fn try_new(
        db: &dyn Arch,
        subject: impl TryResult<ObjectAssignment>,
        kind: EdgeKind,
    ) -> Result<Self> {
        let subject = subject.try_result()?;
        let typ = db.get_object_type(subject.as_object_key(db))?;
        if let ObjectType::Bit = typ.as_ref() {
            Ok(Self { subject, kind })
        } else {
            Err(Error::InvalidArgument(format!(
                "{} function expects a Bit (std_logic), cannot operate on a {}.",
                kind, typ
            )))
        }
    }

    pub fn rising_edge(db: &dyn Arch, subject: impl TryResult<ObjectAssignment>) -> Result<Self> {
        Self::try_new(db, subject, EdgeKind::Rising)
    }

    pub fn falling_edge(db: &dyn Arch, subject: impl TryResult<ObjectAssignment>) -> Result<Self> {
        Self::try_new(db, subject, EdgeKind::Falling)
    }
}

impl DeclareWithIndent for Edge {
    fn declare_with_indent(&self, db: &dyn Arch, indent_style: &str) -> Result<String> {
        Ok(format!(
            "{}({})",
            self.kind(),
            self.subject().declare_with_indent(db, indent_style)?
        ))
    }
}
