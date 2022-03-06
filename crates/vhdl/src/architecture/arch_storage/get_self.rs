use tydi_intern::Id;

use crate::architecture::arch_storage::interner::GetSelf;
use crate::{
    declaration::{ArchitectureDeclaration, ObjectDeclaration},
    object::object_type::ObjectType,
};

use super::Arch;

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

impl GetSelf<ObjectType> for Id<ObjectType> {
    fn get(&self, db: &dyn Arch) -> ObjectType {
        db.lookup_intern_object_type(*self)
    }
}
