use super::{AliasDeclaration, ArchitectureDeclaration, ObjectDeclaration};

impl From<ObjectDeclaration> for ArchitectureDeclaration {
    fn from(object: ObjectDeclaration) -> Self {
        ArchitectureDeclaration::Object(object)
    }
}

impl From<AliasDeclaration> for ArchitectureDeclaration {
    fn from(alias: AliasDeclaration) -> Self {
        ArchitectureDeclaration::Alias(alias)
    }
}
