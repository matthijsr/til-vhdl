pub mod statement;

use itertools::Itertools;
use textwrap::indent;
use tydi_common::{
    error::{Result, TryResult},
    map::InsertionOrderedMap,
};
use tydi_intern::Id;

use crate::{
    architecture::arch_storage::Arch,
    common::vhdl_name::VhdlName,
    declaration::{DeclareWithIndent, ObjectDeclaration},
    statement::label::Label,
    usings::{ListUsings, Usings, ListUsingsDb},
};

use self::statement::SequentialStatement;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Process {
    /// The label of this process.
    ///
    /// While not required in VHDL, this significantly improves the ability to
    /// debug issues in code generation (and in VHDL simulation itself), and is
    /// thus required by this implementation.
    label: VhdlName,
    /// The sensitivity list of this process, indexed by the names of the
    /// objects.
    sensitivity_list: InsertionOrderedMap<VhdlName, Id<ObjectDeclaration>>,
    /// Variable declarations on this process. Indexed by their names.
    ///
    /// While a process's declarative part can technically contain different
    /// declarations. For our purposes, only variables are relevant at this
    /// time.
    variable_declarations: InsertionOrderedMap<VhdlName, Id<ObjectDeclaration>>,
    /// The process's statements.
    statements: Vec<SequentialStatement>,
    /// Any usings accumulated from declarations
    usings: Usings,
}

impl Process {
    /// The sensitivity list of this process, indexed by the names of the
    /// objects.
    #[must_use]
    pub fn sensitivity_list(&self) -> &InsertionOrderedMap<VhdlName, Id<ObjectDeclaration>> {
        &self.sensitivity_list
    }

    /// Get a reference to the process's variable declarations.
    #[must_use]
    pub fn variable_declarations(&self) -> &InsertionOrderedMap<VhdlName, Id<ObjectDeclaration>> {
        &self.variable_declarations
    }

    /// Get a reference to the process's statements.
    #[must_use]
    pub fn statements(&self) -> &[SequentialStatement] {
        self.statements.as_ref()
    }

    pub fn try_new(label: impl TryResult<VhdlName>) -> Result<Self> {
        Ok(Self::new(label.try_result()?))
    }

    pub fn new(label: impl Into<VhdlName>) -> Self {
        Self {
            label: label.into(),
            sensitivity_list: InsertionOrderedMap::new(),
            variable_declarations: InsertionOrderedMap::new(),
            statements: vec![],
            usings: Usings::new_empty(),
        }
    }

    pub fn add_statement(&mut self, db: &dyn Arch, statement: impl TryResult<SequentialStatement>) -> Result<()> {
        let statement = statement.try_result()?;
        self.usings.combine(&statement.list_usings_db(db)?);
        self.statements.push(statement);
        Ok(())
    }
}

impl Label for Process {
    fn set_label(&mut self, label: impl Into<VhdlName>) {
        self.label = label.into()
    }

    fn label(&self) -> Option<&VhdlName> {
        Some(&self.label)
    }
}

impl ListUsings for Process {
    fn list_usings(&self) -> Result<Usings> {
        todo!()
    }
}

impl DeclareWithIndent for Process {
    fn declare_with_indent(&self, db: &dyn Arch, indent_style: &str) -> Result<String> {
        let mut result = if self.sensitivity_list().len() > 0 {
            format!("process({})\n", self.sensitivity_list().keys().join(", "))
        } else {
            "process\n".to_string()
        };

        let mut declarations = String::new();
        for declaration in self.variable_declarations().values() {
            declarations.push_str(&format!(
                "{};\n",
                declaration.declare_with_indent(db, indent_style)?
            ));
        }
        result.push_str(&indent(&declarations, indent_style));

        result.push_str("begin\n");

        let mut statements = String::new();
        for statement in self.statements() {
            statements.push_str(&format!(
                "{};\n",
                statement.declare_with_indent(db, indent_style)?
            ));
        }
        result.push_str(&indent(&statements, indent_style));

        result.push_str(&format!("end process {};", &self.label));

        Ok(result)
    }
}
