use indexmap::IndexMap;
use textwrap::indent;
use tydi_common::error::{Error, Result};
use tydi_common::name::Name;

use crate::declaration::Declare;
use crate::object::ObjectType;

/// A record object
#[derive(Debug, Clone, PartialEq)]
pub struct RecordObject {
    type_name: Name,
    fields: IndexMap<String, ObjectType>,
}

impl RecordObject {
    pub fn new(type_name: impl Into<Name>, fields: IndexMap<String, ObjectType>) -> RecordObject {
        RecordObject {
            type_name: type_name.into(),
            fields,
        }
    }

    pub fn type_name(&self) -> String {
        self.type_name.to_string()
    }

    pub fn fields(&self) -> &IndexMap<String, ObjectType> {
        &self.fields
    }

    pub fn get_field(&self, field_name: impl Into<String>) -> Result<&ObjectType> {
        let field_name = &field_name.into();
        self.fields()
            .get(field_name)
            .ok_or(Error::InvalidArgument(format!(
                "Field {} does not exist on record with type {}",
                field_name,
                self.type_name()
            )))
    }
}

impl Declare for RecordObject {
    fn declare(&self) -> Result<String> {
        let mut this = format!("type {} is record\n", self.type_name());
        let mut fields = String::new();
        for (name, typ) in self.fields() {
            fields.push_str(format!("{} : {};\n", name, typ.type_name()).as_str());
        }
        this.push_str(&indent(&fields, "  "));
        this.push_str("end record;");
        Ok(this)
    }
}
