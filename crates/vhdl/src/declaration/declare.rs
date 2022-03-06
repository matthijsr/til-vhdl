use tydi_common::{error::Result, traits::Identify};
use tydi_intern::Id;

use crate::{
    architecture::{arch_storage::Arch, ArchitectureDeclare},
    object::object_type::DeclarationTypeName,
};

use super::{ArchitectureDeclaration, ObjectDeclaration, ObjectKind};

impl ArchitectureDeclare for ArchitectureDeclaration {
    fn declare_with_indent(&self, db: &dyn Arch, indent_style: &str) -> Result<String> {
        match self {
            ArchitectureDeclaration::Type(_) => todo!(),
            ArchitectureDeclaration::SubType(_) => todo!(),
            ArchitectureDeclaration::Procedure(_) => todo!(),
            ArchitectureDeclaration::Function(_) => todo!(),
            ArchitectureDeclaration::Object(object) => object.declare_with_indent(db, indent_style),
            ArchitectureDeclaration::Alias(_) => todo!(),
            ArchitectureDeclaration::Component(_) => todo!(),
            ArchitectureDeclaration::Custom(_) => todo!(),
        }
    }
}

impl ArchitectureDeclare for ObjectDeclaration {
    fn declare_with_indent(&self, db: &dyn Arch, indent_style: &str) -> Result<String> {
        let default_string = if let Some(default) = self.default() {
            format!(
                " := {}",
                default.declare_for(db, self.identifier(), indent_style)?
            )
        } else {
            "".to_string()
        };
        Ok(match self.kind() {
            ObjectKind::Signal => format!(
                "signal {} : {}{}",
                self.identifier(),
                self.typ().declaration_type_name(),
                default_string
            ),
            ObjectKind::Variable => format!(
                "variable {} : {}{}",
                self.identifier(),
                self.typ().declaration_type_name(),
                default_string
            ),
            ObjectKind::Constant => format!(
                "constant {} : {}{}",
                self.identifier(),
                self.typ().declaration_type_name(),
                default_string
            ),
            ObjectKind::EntityPort(_) => "".to_string(), // Entity ports are part of the architecture, but aren't declared in the declaration part
            ObjectKind::ComponentPort(mode) => format!(
                "{} : {} {}{}",
                self.identifier(),
                mode,
                self.typ().declaration_type_name(),
                default_string
            ),
        })
    }
}

impl ArchitectureDeclare for Id<ObjectDeclaration> {
    fn declare_with_indent(&self, db: &dyn Arch, indent_style: &str) -> Result<String> {
        db.lookup_intern_object_declaration(*self)
            .declare_with_indent(db, indent_style)
    }
}

#[cfg(test)]
mod tests {
    use crate::object::object_type::ObjectType;
    use crate::{architecture::arch_storage::db::Database, assignment::StdLogicValue};

    use super::*;

    #[test]
    fn test_declarations() -> Result<()> {
        let mut db = Database::default();
        assert_eq!(
            "signal TestSignal : std_logic",
            ObjectDeclaration::signal(&mut db, "TestSignal", ObjectType::Bit, None)?
                .declare(&db)?
        );
        assert_eq!(
            "variable TestVariable : std_logic",
            ObjectDeclaration::variable(&mut db, "TestVariable", ObjectType::Bit, None)?
                .declare(&db)?
        );
        assert_eq!(
            "signal SignalWithDefault : std_logic := 'U'",
            ObjectDeclaration::signal(
                &mut db,
                "SignalWithDefault",
                ObjectType::Bit,
                Some(StdLogicValue::U.into())
            )?
            .declare(&mut db)?
        );
        assert_eq!(
            "constant TestConstant : std_logic := 'U'",
            ObjectDeclaration::constant(
                &mut db,
                "TestConstant",
                ObjectType::Bit,
                StdLogicValue::U
            )?
            .declare(&db)?
        );
        Ok(())
    }
}
