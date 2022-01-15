use tydi_intern::Id;

use crate::declaration::{ArchitectureDeclaration, ObjectDeclaration};

use super::{Arch, InternSelf};

impl InternSelf for ArchitectureDeclaration {
    fn intern(self, db: &dyn Arch) -> Id<Self> {
        db.intern_architecture_declaration(self)
    }
}

impl InternSelf for ObjectDeclaration {
    fn intern(self, db: &dyn Arch) -> Id<Self> {
        db.intern_object_declaration(self)
    }
}
