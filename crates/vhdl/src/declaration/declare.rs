use tydi_common::error::{Error, Result};
use tydi_intern::Id;

use crate::architecture::{arch_storage::Arch, ArchitectureDeclare};

use super::{ArchitectureDeclaration, ObjectDeclaration, ObjectKind, ObjectMode, ObjectState};

impl ArchitectureDeclare for ArchitectureDeclaration {
    fn declare(&self, db: &dyn Arch, pre: &str, post: &str) -> Result<String> {
        match self {
            ArchitectureDeclaration::Type(_) => todo!(),
            ArchitectureDeclaration::SubType(_) => todo!(),
            ArchitectureDeclaration::Procedure(_) => todo!(),
            ArchitectureDeclaration::Function(_) => todo!(),
            ArchitectureDeclaration::Object(object) => object.declare(db, pre, post),
            ArchitectureDeclaration::Alias(_) => todo!(),
            ArchitectureDeclaration::Component(_) => todo!(),
            ArchitectureDeclaration::Custom(_) => todo!(),
        }
    }
}

impl ArchitectureDeclare for ObjectDeclaration {
    fn declare(&self, db: &dyn Arch, pre: &str, post: &str) -> Result<String> {
        if self.kind() == ObjectKind::EntityPort {
            // Entity ports are part of the architecture, but aren't declared in the declaration part
            return Ok("".to_string());
        }
        let mut result = pre.to_string();
        result.push_str(match self.kind() {
            ObjectKind::Signal => "signal ",
            ObjectKind::Variable => "variable ",
            ObjectKind::Constant => "constant ",
            ObjectKind::EntityPort => "", // Should be unreachable
            ObjectKind::ComponentPort => "",
        });
        result.push_str(&self.identifier());
        result.push_str(" : ");
        if self.kind() == ObjectKind::ComponentPort {
            match self.mode().state() {
                ObjectState::Assigned => result.push_str("out "),
                ObjectState::Unassigned => result.push_str("in "),
            };
        }
        result.push_str(self.typ().type_name().as_str());
        if let Some(default) = self.default() {
            result.push_str(" := ");
            result.push_str(&default.declare_for(db, self.identifier(), pre, post)?);
        }
        result.push_str(post);
        Ok(result)
    }
}

impl ArchitectureDeclare for Id<ObjectDeclaration> {
    fn declare(&self, db: &dyn Arch, pre: &str, post: &str) -> Result<String> {
        db.lookup_intern_object_declaration(*self)
            .declare(db, pre, post)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        architecture::arch_storage::db::Database, assignment::StdLogicValue, object::ObjectType,
    };

    use super::*;

    #[test]
    fn test_declarations() -> Result<()> {
        let mut db = Database::default();
        assert_eq!(
            "signal TestSignal : std_logic;\n",
            ObjectDeclaration::signal(&mut db, "TestSignal", ObjectType::Bit, None)
                .declare(&db, "", ";\n")?
        );
        assert_eq!(
            "variable TestVariable : std_logic;\n",
            ObjectDeclaration::variable(&mut db, "TestVariable", ObjectType::Bit, None)
                .declare(&db, "", ";\n")?
        );
        assert_eq!(
            "signal SignalWithDefault : std_logic := 'U';\n",
            ObjectDeclaration::signal(
                &mut db,
                "SignalWithDefault",
                ObjectType::Bit,
                Some(StdLogicValue::U.into())
            )
            .declare(&mut db, "", ";\n")?
        );
        assert_eq!(
            "  constant TestConstant : std_logic := 'U';\n",
            ObjectDeclaration::constant(&mut db, "TestConstant", ObjectType::Bit, StdLogicValue::U)
                .declare(&db, "  ", ";\n")?
        );
        Ok(())
    }
}
