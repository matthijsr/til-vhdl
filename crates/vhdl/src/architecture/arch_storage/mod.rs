use std::sync::Arc;

use tydi_common::error::{Error, Result};
use tydi_intern::Id;

use crate::{
    assignment::AssignmentKind,
    declaration::{ArchitectureDeclaration, ObjectDeclaration, ObjectMode},
    object::ObjectType,
    statement::Statement,
};

use super::Architecture;

pub mod db;

#[salsa::query_group(ArchStorage)]
pub trait Arch {
    #[salsa::input]
    fn architecture(&self) -> Arc<Architecture>;

    #[salsa::input]
    fn object_mode(&self, key: Id<ObjectDeclaration>) -> ObjectMode;

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
