use tydi_common::error::Result;
use tydi_intern::Id;

use crate::{
    assignment::FieldSelection,
    object::{object_type::ObjectType, Object},
};

use super::interner::Interner;

#[salsa::query_group(ObjectStorage)]
pub trait ObjectQueries: Interner {
    fn assignable_types(&self, left: Id<ObjectType>, right: Id<ObjectType>) -> Result<()>;

    fn get_object(&self, key: ObjectKey) -> Result<Object>;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectKey {
    obj: Id<Object>,
    selection: Vec<FieldSelection>,
}

impl ObjectKey {
    pub fn new(obj: Id<Object>, selection: Vec<FieldSelection>) -> Self {
        ObjectKey { obj, selection }
    }

    pub fn obj(&self) -> Id<Object> {
        self.obj
    }

    pub fn selection(&self) -> &Vec<FieldSelection> {
        &self.selection
    }
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
