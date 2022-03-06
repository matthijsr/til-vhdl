use std::iter::FromIterator;

use indexmap::IndexMap;
use textwrap::indent;
use tydi_common::error::{Error, Result};
use tydi_common::traits::Identify;

use crate::architecture::arch_storage::Arch;
use crate::common::vhdl_name::{VhdlName, VhdlNameSelf};
use crate::declaration::DeclareWithIndent;
use crate::object::object_type::ObjectType;

use super::object_type::DeclarationTypeName;

/// A record object
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecordObject {
    type_name: VhdlName,
    fields: Vec<RecordField>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecordField {
    field: VhdlName,
    typ: ObjectType,
}

impl RecordField {
    pub fn new(field: VhdlName, typ: ObjectType) -> Self {
        RecordField { field, typ }
    }

    pub fn field(&self) -> &VhdlName {
        &self.field
    }

    pub fn typ(&self) -> &ObjectType {
        &self.typ
    }
}

impl RecordObject {
    pub fn new(
        type_name: impl Into<VhdlName>,
        fields: IndexMap<VhdlName, ObjectType>,
    ) -> RecordObject {
        RecordObject {
            type_name: type_name.into(),
            fields: fields
                .into_iter()
                .map(|(field, typ)| RecordField::new(field, typ))
                .collect(),
        }
    }

    pub fn fields(&self) -> IndexMap<VhdlName, &ObjectType> {
        IndexMap::from_iter(self.fields.iter().map(|x| (x.field().clone(), x.typ())))
    }

    pub fn get_field(&self, field_name: &VhdlName) -> Result<&ObjectType> {
        match self.fields().get(field_name) {
            Some(field) => Ok(field),
            None => Err(Error::InvalidArgument(format!(
                "Field {} does not exist on record with type {}",
                field_name.to_string(),
                self.vhdl_name()
            ))),
        }
    }
}

impl DeclareWithIndent for RecordObject {
    fn declare_with_indent(&self, _db: &dyn Arch, indent_style: &str) -> Result<String> {
        let mut this = format!("type {} is record\n", self.vhdl_name());
        let mut fields = String::new();
        for (name, typ) in self.fields() {
            fields.push_str(format!("{} : {};\n", name, typ.declaration_type_name()).as_str());
        }
        this.push_str(&indent(&fields, indent_style));
        this.push_str("end record;");
        Ok(this)
    }
}

impl VhdlNameSelf for RecordObject {
    fn vhdl_name(&self) -> &VhdlName {
        &self.type_name
    }
}

impl Identify for RecordObject {
    fn identifier(&self) -> String {
        self.vhdl_name().to_string()
    }
}

impl DeclarationTypeName for RecordObject {
    fn declaration_type_name(&self) -> String {
        self.identifier()
    }
}
