pub mod statement;

use itertools::Itertools;
use textwrap::indent;
use tydi_common::{
    error::{Error, Result, TryResult},
    map::InsertionOrderedMap,
};
use tydi_intern::Id;

use crate::{
    architecture::arch_storage::Arch,
    assignment::AssignmentKind,
    common::vhdl_name::{VhdlName, VhdlNameSelf},
    declaration::{DeclareWithIndent, ObjectDeclaration, ObjectKind},
    object::object_type::ObjectType,
    statement::label::Label,
    usings::{ListUsings, ListUsingsDb, Usings},
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

    pub fn usings(&self) -> &Usings {
        &self.usings
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

    pub fn add_sensitivity(&mut self, db: &dyn Arch, obj: Id<ObjectDeclaration>) -> Result<()> {
        self.sensitivity_list
            .try_insert(db.get_object_declaration_name(obj).as_ref().clone(), obj)
    }

    pub fn add_statement(
        &mut self,
        db: &dyn Arch,
        statement: impl TryResult<SequentialStatement>,
    ) -> Result<()> {
        let statement = statement.try_result()?;
        self.usings.combine(&statement.list_usings_db(db)?);
        self.statements.push(statement);
        Ok(())
    }

    /// Declare a variable directly
    pub fn declare_variable(
        &mut self,
        db: &dyn Arch,
        identifier: impl TryResult<VhdlName>,
        typ: impl TryResult<ObjectType>,
        default: Option<AssignmentKind>,
    ) -> Result<Id<ObjectDeclaration>> {
        let name = identifier.try_result()?;
        let var = ObjectDeclaration::variable(db, name.clone(), typ, default)?;
        self.variable_declarations.try_insert(name, var)?;
        Ok(var)
    }

    /// Add a variable from an ObjectDeclaration
    pub fn add_variable_declaration(
        &mut self,
        db: &dyn Arch,
        variable: impl TryResult<ObjectDeclaration>,
    ) -> Result<Id<ObjectDeclaration>> {
        let variable: ObjectDeclaration = variable.try_result()?;
        if let ObjectKind::Variable = variable.kind() {
            let var_name = variable.vhdl_name().clone();
            self.usings.combine(&variable.list_usings()?);
            let var_id = db.intern_object_declaration(variable);
            self.variable_declarations.try_insert(var_name, var_id)?;
            Ok(var_id)
        } else {
            Err(Error::InvalidArgument(format!(
                "Cannot declare a {} on a process.",
                variable.kind()
            )))
        }
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
        Ok(self.usings().clone())
    }
}

impl DeclareWithIndent for Process {
    fn declare_with_indent(&self, db: &dyn Arch, indent_style: &str) -> Result<String> {
        let mut result = if self.sensitivity_list().len() > 0 {
            format!(
                "process({}) is\n",
                self.sensitivity_list().keys().join(", ")
            )
        } else {
            "process is\n".to_string()
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

#[cfg(test)]
mod tests {
    use crate::{
        architecture::arch_storage::db::Database,
        assignment::{Assign, StdLogicValue},
        declaration::Declare,
        object::object_type::time::TimeValueFrom,
        statement::relation::CombineRelation,
    };

    use super::{
        statement::{test_statement::TestStatement, wait::Wait},
        *,
    };

    #[test]
    fn test_process_declare() -> Result<()> {
        let mut _db = Database::default();
        let db = &mut _db;
        let mut process = Process::try_new("test_proc")?;
        let clk = ObjectDeclaration::entity_clk(db);
        process.add_sensitivity(db, clk)?;
        let bool_var = process.declare_variable(db, "bool_var", ObjectType::Boolean, None)?;
        process.add_statement(
            db,
            bool_var.assign(db, clk.r_eq(db, StdLogicValue::Logic(true))?)?,
        )?;
        process.add_statement(db, Wait::wait().for_constant(1.us()))?;
        process.add_statement(db, TestStatement::assert_report(false, "end test"))?;
        assert_eq!(
            r#"process(clk) is
  variable bool_var : boolean;
begin
  bool_var := clk = '1';
  wait for 1 us;
  assert false report "end test";
end process test_proc;"#,
            process.declare(db)?
        );
        Ok(())
    }
}
