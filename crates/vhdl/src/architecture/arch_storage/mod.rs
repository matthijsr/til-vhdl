use std::sync::Arc;

use tydi_common::name::Name;
use tydi_intern::Id;

use crate::{
    declaration::{ArchitectureDeclaration, ObjectDeclaration},
    object::ObjectType,
    statement::Statement,
};

pub mod db;

#[salsa::query_group(ArchStorage)]
pub trait Arch {
    #[salsa::interned]
    fn intern_architecture_declaration(
        &self,
        arch_decl: ArchitectureDeclaration,
    ) -> Id<ArchitectureDeclaration>;

    #[salsa::interned]
    fn intern_object_declaration(&self, obj_decl: ObjectDeclaration) -> Id<ObjectDeclaration>;

    // #[salsa::interned]
    // fn intern_statement(&self, stat: Statement) -> Id<Statement>;
}
