use tydi_common::error::Result;
use tydi_intern::Id;

use crate::{
    assignment::FieldSelection,
    object::{object_type::ObjectType, Object},
};

use super::interner::Interner;

#[salsa::query_group(ObjectStorage)]
pub trait ObjectQueries: Interner {
    /// Verify whether two object types can be assigned to one another
    fn assignable_types(&self, left: Id<ObjectType>, right: Id<ObjectType>) -> Result<()>;

    /// Verify whether `from` can be assigned to `to`
    fn assignable_objects(&self, to: ObjectKey, from: ObjectKey) -> Result<()>;

    /// Get an object based on its key
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

    pub fn with_selection(mut self, selection: FieldSelection) -> Self {
        self.selection.push(selection);
        self
    }

    pub fn with_nested(mut self, mut selection: Vec<FieldSelection>) -> Self {
        self.selection.append(&mut selection);
        self
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

fn assignable_objects(db: &dyn ObjectQueries, to: ObjectKey, from: ObjectKey) -> Result<()> {
    let to = db.get_object(to)?;
    to.assignable.to_or_err()?;
    let from = db.get_object(from)?;
    from.assignable.from_or_err()?;

    db.assignable_types(to.typ, from.typ)
}
