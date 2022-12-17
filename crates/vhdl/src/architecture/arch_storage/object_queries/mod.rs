use std::sync::Arc;

use tydi_common::error::Result;
use tydi_intern::Id;

use crate::{
    declaration::{ObjectDeclaration, ObjectKind},
    object::{object_type::ObjectType, Object},
};

use self::object_key::ObjectKey;

use super::interner::Interner;

pub mod object_key;

#[salsa::query_group(ObjectStorage)]
pub trait ObjectQueries: Interner {
    /// Verify whether two object types can be assigned to one another
    fn assignable_types(&self, left: Id<ObjectType>, right: Id<ObjectType>) -> Result<()>;

    fn get_object_kind(&self, key: Id<ObjectDeclaration>) -> Arc<ObjectKind>;

    fn get_object_key(&self, key: Id<ObjectDeclaration>) -> ObjectKey;
}

fn assignable_types(
    db: &dyn ObjectQueries,
    left: Id<ObjectType>,
    right: Id<ObjectType>,
) -> Result<()> {
    if left == right {
        Ok(())
    } else {
        let left = db.lookup_intern_object_type(left);
        let right = db.lookup_intern_object_type(right);
        left.can_assign_type(&right)
    }
}

fn get_object_kind(db: &dyn ObjectQueries, key: Id<ObjectDeclaration>) -> Arc<ObjectKind> {
    let obj = db.lookup_intern_object_declaration(key);
    Arc::new(obj.kind().clone())
}

fn get_object_key(db: &dyn ObjectQueries, key: Id<ObjectDeclaration>) -> ObjectKey {
    let obj = db.lookup_intern_object_declaration(key);
    obj.object_key().clone()
}
