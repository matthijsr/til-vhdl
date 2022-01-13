use indexmap::IndexMap;
use tydi_common::name::Name;
use tydi_common::{error::Result, traits::Identify};
use tydi_intern::Id;

use crate::{
    declaration::{ArchitectureDeclaration, ObjectDeclaration},
    entity::Entity,
    package::Package,
    statement::Statement,
    usings::{ListUsings, Usings},
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
    declarations: Vec<Id<ArchitectureDeclaration>>,
    /// The statement part of the architecture
    statements: Vec<Statement>,
}

impl ArchitectureBody {
    pub fn new(declarations: Vec<Id<ArchitectureDeclaration>>, statements: Vec<Statement>) -> Self {
        ArchitectureBody {
            declarations,
            statements,
        }
    }

    pub fn declaration_ids(&self) -> &Vec<Id<ArchitectureDeclaration>> {
        &self.declarations
    }

    pub fn declarations(&self, db: &dyn Arch) -> Vec<ArchitectureDeclaration> {
        self.declaration_ids()
            .iter()
            .map(|x| db.lookup_intern_architecture_declaration(*x))
            .collect()
    }

    pub fn statements(&self) -> &Vec<Statement> {
        &self.statements
    }
}

/// An architecture
#[derive(Debug, Clone)]
pub struct Architecture {
    /// Name of the architecture
    identifier: Name,
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

pub trait ArchitectureDeclare {
    /// Returns a string for the declaration, pre can be used for indentation, post is used for closing characters (','/';')
    fn declare(&self, db: &dyn Arch, pre: &str, post: &str) -> Result<String>;
}

impl Architecture {
    /// Create the architecture based on a component contained within a package, assuming the library (project) is "work" and the architecture's identifier is "Behavioral"
    pub fn new_default(package: &Package, component_id: impl Into<String>) -> Result<Architecture> {
        Architecture::new(
            Name::try_new("work")?,
            Name::try_new("Behavioral")?,
            package,
            component_id,
        )
    }

    /// Create the architecture based on a component contained within a package, specify the library (project) in which the package is contained
    pub fn new(
        library_id: Name,
        identifier: Name,
        package: &Package,
        component_id: impl Into<String>,
    ) -> Result<Architecture> {
        // let mut usings = Usings::new_empty();
        // usings.add_using(Name::try_new("ieee")?, "std_logic_1164.all".to_string());
        let mut usings = package.list_usings()?;
        usings.add_using(library_id, format!("{}.all", package.identifier()));
        Ok(Architecture {
            identifier,
            entity: Entity::from(package.get_component(component_id)?),
            usings: usings,
            doc: None,
            declaration: vec![],
            statement: vec![],
        })
    }

    /// Add additional usings which weren't already part of the package
    pub fn add_using(&mut self, library: Name, using: String) -> bool {
        self.usings.add_using(library, using)
    }

    /// Return this architecture with documentation added.
    pub fn with_doc(mut self, doc: impl Into<String>) -> Self {
        self.doc = Some(doc.into());
        self
    }

    /// Set the documentation of this architecture.
    pub fn set_doc(&mut self, doc: impl Into<String>) {
        self.doc = Some(doc.into())
    }

    pub fn add_declaration(
        &mut self,
        db: &dyn Arch,
        declaration: impl Into<ArchitectureDeclaration>,
    ) -> Result<Id<ArchitectureDeclaration>> {
        let declaration = declaration.into();
        match &declaration {
            ArchitectureDeclaration::Object(object) => {
                self.usings.combine(&object.list_usings()?);
            }
            ArchitectureDeclaration::Alias(alias) => {
                self.usings.combine(
                    &db.lookup_intern_object_declaration(alias.object())
                        .list_usings()?,
                );
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
        match &statement {
            Statement::Assignment(assignment) => self.usings.combine(&assignment.list_usings()?),
            Statement::PortMapping(pm) => {
                for (_, object) in pm.ports() {
                    self.usings
                        .combine(&db.lookup_intern_object_declaration(*object).list_usings()?);
                }
            }
        }
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
    ) -> Result<IndexMap<String, Id<ObjectDeclaration>>> {
        let mut result = IndexMap::new();
        for port in self.entity.ports() {
            let obj = ObjectDeclaration::from_port(db, port, true);
            result.insert(port.identifier().to_string(), obj);
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    // TODO

    //     use crate::generator::{common::convert::Packify, vhdl::Declare};

    use crate::{architecture::arch_storage::db::Database, declaration::Declare, test_tools::*};

    use super::*;

    pub(crate) fn test_package() -> Result<Package> {
        Ok(Package::new(
            Name::try_new("pak")?,
            &vec![empty_component(), component_with_nested_types()?],
            &vec![],
        ))
    }

    #[test]
    fn test_architecture() -> Result<()> {
        let db = Database::default();
        let package = test_package()?;
        let architecture =
            Architecture::new_default(&package, Name::try_new("component_with_nested_types")?)?;
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
