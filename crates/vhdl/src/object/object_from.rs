use crate::object::array::ArrayObject;
use crate::object::record::RecordObject;

use super::ObjectType;

impl From<ArrayObject> for ObjectType {
    fn from(array: ArrayObject) -> Self {
        ObjectType::Array(array)
    }
}

impl From<RecordObject> for ObjectType {
    fn from(rec: RecordObject) -> Self {
        ObjectType::Record(rec)
    }
}
