use arch_storage::interner::GetSelf;
use tydi_common::error::TryResult;
use tydi_common::map::InsertionOrderedMap;
use tydi_common::{error::Result, traits::Identify};
use tydi_intern::Id;

use crate::common::vhdl_name::{VhdlName, VhdlPathName};
use crate::{
    declaration::{ArchitectureDeclaration, ObjectDeclaration},
    entity::Entity,
    package::Package,
    statement::Statement,
    usings::{ListUsings, ListUsingsDb, Usings},
};

use self::arch_storage::Arch;

pub mod arch_storage;
pub mod impls;

// NOTE: One of the main things to consider is probably how to handle multiple element lanes. Probably as a check on the number of lanes,
// then wrapping in a generate statement. Need to consider indexes at that point.
// This'd be easier if I simply always made it an array, even when the number of lanes is 1, but that gets real ugly, real fast.

#[derive(Debug, Clone)]
pub struct ArchitectureBody {
    /// The declaration part of the architecture
    declarations: Vec<ArchitectureDeclaration>,
    /// The statement part of the architecture
    statements: Vec<Statement>,
}

impl ArchitectureBody {
    pub fn new(declarations: Vec<ArchitectureDeclaration>, statements: Vec<Statement>) -> Self {
        ArchitectureBody {
            declarations,
            statements,
        }
    }

    pub fn declarations(&self) -> &Vec<ArchitectureDeclaration> {
        &self.declarations
    }

    // pub fn declarations(&self, db: &dyn Arch) -> Vec<ArchitectureDeclaration> {
    //     self.declaration_ids()
    //         .iter()
    //         .map(|x| db.lookup_intern_architecture_declaration(*x))
    //         .collect()
    // }

    pub fn statements(&self) -> &Vec<Statement> {
        &self.statements
    }
}

/// An architecture
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Architecture {
    /// Name of the architecture
    identifier: VhdlName,
    /// Entity which this architecture is for
    entity: Entity,
    /// Additional usings beyond the Package and those within it
    usings: Usings,
    /// Documentation.
    doc: Option<String>,
    /// The declaration part of the architecture
    declaration: Vec<Id<ArchitectureDeclaration>>,
    /// The statement part of the architecture
    statement: Vec<Statement>,
}

impl Architecture {
    /// Create the architecture based on a component contained within a package, assuming the library (project) is "work" and the architecture's identifier is "Behavioral"
    pub fn new_default(
        package: &Package,
        component_id: impl TryResult<VhdlName>,
    ) -> Result<Architecture> {
        Architecture::new("work", "Behavioral", package, component_id)
    }

    /// Create the architecture based on a component contained within a package, specify the library (project) in which the package is contained
    pub fn new(
        library_id: impl TryResult<VhdlName>,
        identifier: impl TryResult<VhdlName>,
        package: &Package,
        component_id: impl TryResult<VhdlName>,
    ) -> Result<Architecture> {
        // let mut usings = Usings::new_empty();
        // usings.add_using(Name::try_new("ieee")?, "std_logic_1164.all".to_string());
        let library_id = library_id.try_result()?;
        let mut usings = package.list_usings()?;
        usings.add_using(library_id, format!("{}.all", package.identifier()))?;
        Ok(Architecture {
            identifier: identifier.try_result()?,
            entity: Entity::from(package.get_component(component_id)?),
            usings: usings,
            doc: None,
            declaration: vec![],
            statement: vec![],
        })
    }

    /// Create an architecture based on the default package and component defined in the Arch database
    pub fn from_database(db: &dyn Arch, identifier: impl TryResult<VhdlName>) -> Result<Self> {
        let package = db.default_package();
        let mut usings = package.list_usings()?;
        usings.add_using("work", format!("{}.all", package.identifier()))?;
        Ok(Architecture {
            identifier: identifier.try_result()?,
            entity: Entity::from(db.subject_component()?.as_ref()),
            usings: usings,
            doc: None,
            declaration: vec![],
            statement: vec![],
        })
    }

    /// Add additional usings which weren't already part of the package
    pub fn add_using(
        &mut self,
        library: impl TryResult<VhdlName>,
        using: impl TryResult<VhdlPathName>,
    ) -> Result<bool> {
        self.usings.add_using(library, using)
    }

    pub fn add_declaration(
        &mut self,
        db: &dyn Arch,
        declaration: impl Into<ArchitectureDeclaration>,
    ) -> Result<Id<ArchitectureDeclaration>> {
        let declaration = declaration.into();
        match &declaration {
            ArchitectureDeclaration::Object(object) => {
                self.usings.combine(&object.get(db).list_usings()?);
            }
            ArchitectureDeclaration::Type(_)
            | ArchitectureDeclaration::SubType(_)
            | ArchitectureDeclaration::Procedure(_)
            | ArchitectureDeclaration::Function(_)
            | ArchitectureDeclaration::Component(_)
            | ArchitectureDeclaration::Custom(_) => (),
        }
        let id = db.intern_architecture_declaration(declaration);
        self.declaration.push(id);
        Ok(id)
    }

    pub fn add_statement(&mut self, db: &dyn Arch, statement: impl Into<Statement>) -> Result<()> {
        let statement = statement.into();
        self.usings.combine(&statement.list_usings_db(db)?);
        self.statement.push(statement);
        Ok(())
    }

    pub fn statements(&self) -> &Vec<Statement> {
        &self.statement
    }

    pub fn declarations(&self) -> &Vec<Id<ArchitectureDeclaration>> {
        &self.declaration
    }

    pub fn entity_ports(
        &self,
        db: &mut dyn Arch,
    ) -> InsertionOrderedMap<VhdlName, Id<ObjectDeclaration>> {
        self.entity
            .ports()
            .clone()
            .map_convert(|p| ObjectDeclaration::from_port(db, &p, true))
    }

    pub fn entity_parameters(
        &self,
        db: &mut dyn Arch,
    ) -> Result<InsertionOrderedMap<VhdlName, Id<ObjectDeclaration>>> {
        self.entity
            .parameters()
            .clone()
            .try_map_convert(|p| ObjectDeclaration::from_parameter(db, &p))
    }

    pub fn add_body(&mut self, db: &dyn Arch, body: &ArchitectureBody) -> Result<()> {
        for declaration in body.declarations() {
            self.add_declaration(db, declaration.clone())?;
        }
        for statement in body.statements() {
            self.add_statement(db, statement.clone())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // TODO

    //     use crate::generator::{common::convert::Packify, vhdl::Declare};

    use std::sync::Arc;

    use crate::{architecture::arch_storage::db::Database, declaration::Declare, test_tools};

    use super::*;

    pub(crate) fn test_package() -> Result<Package> {
        Package::try_new(
            "pak",
            &vec![
                Arc::new(test_tools::empty_component()),
                Arc::new(test_tools::component_with_nested_types()?),
            ],
            &vec![],
        )
    }

    #[test]
    fn test_architecture() -> Result<()> {
        let db = Database::default();
        let package = test_package()?;
        let architecture = Architecture::new_default(&package, "component_with_nested_types")?;
        assert_eq!(
            r#"library ieee;
use ieee.std_logic_1164.all;

library work;
use work.pak.all;

entity component_with_nested_types is
  port (
    some_other_port : out record_type;
    clk : in std_logic
  );
end component_with_nested_types;

architecture Behavioral of component_with_nested_types is
begin
end Behavioral;"#,
            architecture.declare(&db)?
        );
        Ok(())
    }
}
