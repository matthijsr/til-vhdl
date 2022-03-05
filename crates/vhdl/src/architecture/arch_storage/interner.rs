use tydi_intern::Id;

use crate::{
    declaration::{ArchitectureDeclaration, ObjectDeclaration},
    object::Object,
};
use crate::object::object_type::ObjectType;

#[salsa::query_group(InternerStorage)]
pub trait Interner {
    #[salsa::interned]
    fn intern_architecture_declaration(
        &self,
        arch_decl: ArchitectureDeclaration,
    ) -> Id<ArchitectureDeclaration>;

    #[salsa::interned]
    fn intern_object_declaration(&self, obj_decl: ObjectDeclaration) -> Id<ObjectDeclaration>;

    #[salsa::interned]
    fn intern_object(&self, obj: Object) -> Id<Object>;

    #[salsa::interned]
    fn intern_object_type(&self, object_type: ObjectType) -> Id<ObjectType>;
}
