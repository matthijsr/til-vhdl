use tydi_intern::Id;

use super::{ArchitectureDeclaration, ObjectDeclaration};

impl From<Id<ObjectDeclaration>> for ArchitectureDeclaration {
    fn from(object: Id<ObjectDeclaration>) -> Self {
        ArchitectureDeclaration::Object(object)
    }
}
