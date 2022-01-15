use tydi_intern::Id;

use crate::declaration::{ArchitectureDeclaration, ObjectDeclaration};

use super::{Arch, GetSelf};

impl GetSelf<ArchitectureDeclaration> for Id<ArchitectureDeclaration> {
    fn get(&self, db: &dyn Arch) -> ArchitectureDeclaration {
        db.lookup_intern_architecture_declaration(*self)
    }
}

impl GetSelf<ObjectDeclaration> for Id<ObjectDeclaration> {
    fn get(&self, db: &dyn Arch) -> ObjectDeclaration {
        db.lookup_intern_object_declaration(*self)
    }
}
