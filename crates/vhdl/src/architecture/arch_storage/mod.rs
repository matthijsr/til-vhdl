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

    fn get_object_declaration(&self, key: Id<ObjectDeclaration>) -> Result<ObjectDeclaration>;

    // #[salsa::interned]
    // fn intern_statement(&self, stat: Statement) -> Id<Statement>;
}

fn get_object_declaration(db: &dyn Arch, key: Id<ObjectDeclaration>) -> Result<ObjectDeclaration> {
    let mut obj = db.lookup_intern_object_declaration(key);
    let arch = db.architecture();
    for statement in arch.statements() {
        match statement {
            Statement::Assignment(a) => {
                if a.object() == key {
                    obj.set_mode(a.resulting_mode(db))?;
                }
            }
            Statement::PortMapping(pm) => {
                if let Some(a) = pm.assignment_for(obj.identifier(), key) {
                    obj.set_mode(a.resulting_mode(db))?;
                }
            }
        }
    }
    Ok(obj)
}
