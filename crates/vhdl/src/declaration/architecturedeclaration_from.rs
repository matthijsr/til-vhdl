use tydi_intern::Id;

use super::{AliasDeclaration, ArchitectureDeclaration, ObjectDeclaration};

impl From<Id<ObjectDeclaration>> for ArchitectureDeclaration {
    fn from(object: Id<ObjectDeclaration>) -> Self {
        ArchitectureDeclaration::Object(object)
    }
}

impl From<AliasDeclaration> for ArchitectureDeclaration {
    fn from(alias: AliasDeclaration) -> Self {
        ArchitectureDeclaration::Alias(alias)
    }
}
