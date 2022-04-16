use std::sync::Arc;

use tydi_intern::Id;

use crate::{common::vhdl_name::VhdlName, declaration::ObjectDeclaration};

use super::interner::GetName;

impl GetName<Arc<VhdlName>> for Id<ObjectDeclaration> {
    fn get_name(&self, db: &dyn super::Arch) -> Arc<VhdlName> {
        db.get_object_declaration_name(*self)
    }
}
