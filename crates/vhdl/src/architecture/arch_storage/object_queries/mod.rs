use std::sync::Arc;

use tydi_common::error::Result;
use tydi_intern::Id;

use crate::{
    declaration::ObjectDeclaration,
    object::{object_type::ObjectType, Object},
};

use self::object_key::ObjectKey;

use super::interner::Interner;

pub mod object_key;

#[salsa::query_group(ObjectStorage)]
pub trait ObjectQueries: Interner {
    /// Verify whether two object types can be assigned to one another
    fn assignable_types(&self, left: Id<ObjectType>, right: Id<ObjectType>) -> Result<()>;

    /// Get an object based on its key
    fn get_object(&self, key: ObjectKey) -> Result<Object>;

    fn get_object_type(&self, key: ObjectKey) -> Result<Arc<ObjectType>>;

    fn get_object_declaration_type(&self, key: Id<ObjectDeclaration>) -> Result<Arc<ObjectType>>;
}

fn get_object_type(db: &dyn ObjectQueries, key: ObjectKey) -> Result<Arc<ObjectType>> {
    Ok(Arc::new(
        db.lookup_intern_object_type(db.get_object(key)?.typ_id()),
    ))
}

fn get_object_declaration_type(
    db: &dyn ObjectQueries,
    key: Id<ObjectDeclaration>,
) -> Result<Arc<ObjectType>> {
    db.get_object_type(
        db.lookup_intern_object_declaration(key)
            .object_key()
            .clone(),
    )
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

fn get_object(db: &dyn ObjectQueries, key: ObjectKey) -> Result<Object> {
    let obj = db.lookup_intern_object(key.obj());
    let typ = db
        .lookup_intern_object_type(obj.typ)
        .get_nested(key.selection())?;
    Ok(Object {
        typ: db.intern_object_type(typ),
        assignable: obj.assignable,
    })
}
