use tydi_intern::Id;

use crate::architecture::arch_storage::interner::InternSelf;
use crate::{
    declaration::{ArchitectureDeclaration, ObjectDeclaration},
    object::object_type::ObjectType,
};

use super::Arch;

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

impl InternSelf for ObjectType {
    fn intern(self, db: &dyn Arch) -> Id<Self> {
        db.intern_object_type(self)
    }
}
