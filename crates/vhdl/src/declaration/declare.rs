use tydi_common::{error::Result, traits::Identify};
use tydi_intern::Id;

use crate::{architecture::arch_storage::Arch, object::object_type::DeclarationTypeName};

use super::{ArchitectureDeclaration, DeclareWithIndent, ObjectDeclaration, ObjectKind};

impl DeclareWithIndent for ArchitectureDeclaration {
    fn declare_with_indent(&self, db: &dyn Arch, indent_style: &str) -> Result<String> {
        match self {
            ArchitectureDeclaration::Type(_) => todo!(),
            ArchitectureDeclaration::SubType(_) => todo!(),
            ArchitectureDeclaration::Procedure(_) => todo!(),
            ArchitectureDeclaration::Function(_) => todo!(),
            ArchitectureDeclaration::Object(object) => object.declare_with_indent(db, indent_style),
            ArchitectureDeclaration::Component(_) => todo!(),
            ArchitectureDeclaration::Custom(_) => todo!(),
        }
    }
}

impl DeclareWithIndent for ObjectDeclaration {
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
                self.object(db)?.typ(db).declaration_type_name(db)?,
                default_string
            ),
            ObjectKind::Variable => format!(
                "variable {} : {}{}",
                self.identifier(),
                self.object(db)?.typ(db).declaration_type_name(db)?,
                default_string
            ),
            ObjectKind::Constant => format!(
                "constant {} : {}{}",
                self.identifier(),
                self.object(db)?.typ(db).declaration_type_name(db)?,
                default_string
            ),
            ObjectKind::EntityPort(_) => "".to_string(), // Entity ports are part of the architecture, but aren't declared in the declaration part
            ObjectKind::ComponentPort(mode) => format!(
                "{} : {} {}{}",
                self.identifier(),
                mode,
                self.object(db)?.typ(db).declaration_type_name(db)?,
                default_string
            ),
            ObjectKind::Alias(obj, _) => {
                let field_selection_string = self
                    .object_key()
                    .selection()
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join("");
                format!(
                    "alias {} : {} is {}{}",
                    self.identifier(),
                    self.object(db)?.typ(db).declaration_type_name(db)?,
                    obj,
                    field_selection_string
                )
            }
        })
    }
}

impl DeclareWithIndent for Id<ObjectDeclaration> {
    fn declare_with_indent(&self, db: &dyn Arch, indent_style: &str) -> Result<String> {
        db.lookup_intern_object_declaration(*self)
            .declare_with_indent(db, indent_style)
    }
}

#[cfg(test)]
mod tests {
    use crate::declaration::Declare;
    use crate::object::object_type::ObjectType;
    use crate::{architecture::arch_storage::db::Database, assignment::StdLogicValue};

    use super::*;

    #[test]
    fn test_declarations() -> Result<()> {
        let _db = Database::default();
        let db = &_db;
        let test_signal = ObjectDeclaration::signal(db, "TestSignal", ObjectType::Bit, None)?;
        assert_eq!("signal TestSignal : std_logic", test_signal.declare(db)?);
        assert_eq!(
            "variable TestVariable : std_logic",
            ObjectDeclaration::variable(db, "TestVariable", ObjectType::Bit, None)?.declare(db)?
        );
        assert_eq!(
            "signal SignalWithDefault : std_logic := 'U'",
            ObjectDeclaration::signal(
                db,
                "SignalWithDefault",
                ObjectType::Bit,
                Some(StdLogicValue::U.into())
            )?
            .declare(db)?
        );
        assert_eq!(
            "constant TestConstant : std_logic := 'U'",
            ObjectDeclaration::constant(db, "TestConstant", ObjectType::Bit, StdLogicValue::U)?
                .declare(db)?
        );
        assert_eq!(
            "alias TestAlias : std_logic is TestSignal",
            ObjectDeclaration::alias(db, "TestAlias", test_signal, vec![])?.declare(db)?
        );
        Ok(())
    }
}
