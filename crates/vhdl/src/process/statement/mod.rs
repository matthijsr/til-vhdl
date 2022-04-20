pub mod case;
pub mod condition;
pub mod control_flow;
pub mod ifelse;
pub mod loop_statement;
pub mod test_statement;
pub mod wait;

use tydi_common::error::Result;

use crate::{
    architecture::arch_storage::Arch,
    assignment::AssignDeclaration,
    common::vhdl_name::VhdlName,
    declaration::DeclareWithIndent,
    statement::label::Label,
    usings::{ListUsingsDb, Usings},
};

use self::{control_flow::ControlFlow, test_statement::TestStatement};

pub type Block = Vec<SequentialStatement>;

// REFER TO: https://insights.sigasi.com/tech/vhdl2008.ebnf/

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SequentialStatement {
    Assignment(AssignDeclaration),
    Control(ControlFlow),
    Test(TestStatement),
}

impl From<AssignDeclaration> for SequentialStatement {
    fn from(assign: AssignDeclaration) -> Self {
        Self::Assignment(assign)
    }
}

impl<T: Into<ControlFlow>> From<T> for SequentialStatement {
    fn from(control: T) -> Self {
        Self::Control(control.into())
    }
}

impl From<TestStatement> for SequentialStatement {
    fn from(test: TestStatement) -> Self {
        Self::Test(test)
    }
}

impl Label for SequentialStatement {
    fn label(&self) -> Option<&VhdlName> {
        match self {
            SequentialStatement::Assignment(a) => a.label(),
            SequentialStatement::Control(c) => c.label(),
            SequentialStatement::Test(t) => t.label(),
        }
    }

    fn set_label(&mut self, label: impl Into<VhdlName>) {
        match self {
            SequentialStatement::Assignment(a) => a.set_label(label),
            SequentialStatement::Control(c) => c.set_label(label),
            SequentialStatement::Test(t) => t.set_label(label),
        }
    }
}

impl ListUsingsDb for SequentialStatement {
    fn list_usings_db(&self, db: &dyn Arch) -> Result<Usings> {
        match self {
            SequentialStatement::Assignment(a) => a.list_usings_db(db),
            SequentialStatement::Control(c) => c.list_usings_db(db),
            SequentialStatement::Test(_) => Ok(Usings::new_empty()),
        }
    }
}

impl DeclareWithIndent for SequentialStatement {
    fn declare_with_indent(&self, db: &dyn Arch, indent_style: &str) -> Result<String> {
        let result = match self {
            SequentialStatement::Assignment(assignment) => {
                assignment.declare_with_indent(db, indent_style)
            }
            SequentialStatement::Control(c) => c.declare_with_indent(db, indent_style),
            SequentialStatement::Test(t) => t.declare_with_indent(db, indent_style),
        };
        if let Some(label) = self.label() {
            Ok(format!("{}: {}", label, result?))
        } else {
            result
        }
    }
}
