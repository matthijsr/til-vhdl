use tydi_intern::Id;

use crate::{assignment::FieldSelection, object::Object};

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

impl From<Id<Object>> for ObjectKey {
    fn from(id: Id<Object>) -> Self {
        ObjectKey {
            obj: id,
            selection: vec![],
        }
    }
}
