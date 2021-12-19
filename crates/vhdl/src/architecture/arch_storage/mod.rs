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

    /// Object declarations are initially interned, representing their base declaration.
    ///
    /// Subsequently, it should be modified using the salsa::input derivation `object_declaration`/`set_object_declaration`
    #[salsa::interned]
    fn intern_object_declaration(&self, obj_decl: ObjectDeclaration) -> Id<ObjectDeclaration>;

    // TODO: This whole thing probably makes more sense as a query based on the initial object and after applying subsequent assignments.
    #[salsa::input]
    fn object_declaration(&self, key: Id<ObjectDeclaration>) -> Option<ObjectDeclaration>;

    fn get_object_declaration(&self, key: Id<ObjectDeclaration>) -> ObjectDeclaration;

    //fn set_object_declaration(&mut self, key: Id<ObjectDeclaration>, value: ObjectDeclaration);

    //fn set_object_declaration(&mut self, key: Id<ObjectDeclaration>, value: ObjectDeclaration) -> ();

    // #[salsa::interned]
    // fn intern_statement(&self, stat: Statement) -> Id<Statement>;
}

fn get_object_declaration(db: &dyn Arch, key: Id<ObjectDeclaration>) -> ObjectDeclaration {
    match db.object_declaration(key) {
        Some(object) => object,
        None => db.lookup_intern_object_declaration(key),
    }
}
