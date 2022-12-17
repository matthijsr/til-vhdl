use std::convert::TryInto;

use textwrap::indent;
use tydi_common::{
    error::{Error, Result, TryResult},
    traits::Identify,
};

use crate::{
    architecture::arch_storage::Arch,
    common::vhdl_name::{VhdlName, VhdlNameSelf},
    statement::relation::Relation,
};
use crate::{declaration::DeclareWithIndent, object::object_type::ObjectType};

use super::object_type::DeclarationTypeName;

/// An array object, arrays contain a single type of object, but can contain nested objects
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArrayObject {
    high: Relation,
    low: Relation,
    typ: Box<ObjectType>,
    type_name: VhdlName,
    is_std_logic_vector: bool,
}

impl ArrayObject {
    /// Create a bit vector object
    pub fn bit_vector(high: i32, low: i32) -> Result<ArrayObject> {
        if low > high {
            Err(Error::InvalidArgument(format!(
                "{} > {}! Low must be lower than high",
                low, high
            )))
        } else {
            Ok(ArrayObject {
                high: high.into(),
                low: low.into(),
                typ: Box::new(ObjectType::Bit),
                type_name: VhdlName::try_new("std_logic_vector")?,
                is_std_logic_vector: true,
            })
        }
    }

    /// Create an array of a specific field type
    pub fn array(
        high: i32,
        low: i32,
        object: ObjectType,
        type_name: impl TryResult<VhdlName>,
    ) -> Result<ArrayObject> {
        if low > high {
            Err(Error::InvalidArgument(format!(
                "{} > {}! Low must be lower than high",
                low, high
            )))
        } else {
            Ok(ArrayObject {
                high: high.into(),
                low: low.into(),
                typ: Box::new(object),
                type_name: type_name.try_result()?,
                is_std_logic_vector: false,
            })
        }
    }

    /// Create a bit vector object
    pub fn relation_bit_vector(
        db: &dyn Arch,
        high: impl Into<Relation>,
        low: impl Into<Relation>,
    ) -> Result<ArrayObject> {
        let high = high.into();
        let low = low.into();
        high.is_integer(db)?;
        low.is_integer(db)?;
        Ok(ArrayObject {
            high,
            low,
            typ: Box::new(ObjectType::Bit),
            type_name: VhdlName::try_new("std_logic_vector")?,
            is_std_logic_vector: true,
        })
    }

    /// Create an array of a specific field type
    pub fn relation_array(
        db: &dyn Arch,
        high: impl Into<Relation>,
        low: impl Into<Relation>,
        object: ObjectType,
        type_name: impl TryResult<VhdlName>,
    ) -> Result<ArrayObject> {
        let high = high.into();
        let low = low.into();
        high.is_integer(db)?;
        low.is_integer(db)?;
        Ok(ArrayObject {
            high,
            low,
            typ: Box::new(object),
            type_name: type_name.try_result()?,
            is_std_logic_vector: false,
        })
    }

    pub fn typ(&self) -> &ObjectType {
        &self.typ
    }

    pub fn high(&self) -> &Relation {
        &self.high
    }

    pub fn low(&self) -> &Relation {
        &self.low
    }

    pub fn width(&self) -> u32 {
        todo!()
        // (1 + self.high - self.low).try_into().unwrap()
    }

    pub fn is_bitvector(&self) -> bool {
        match self.typ() {
            ObjectType::Bit => true,
            _ => false,
        }
    }

    pub fn is_std_logic_vector(&self) -> bool {
        self.is_std_logic_vector
    }
}

impl DeclareWithIndent for ArrayObject {
    fn declare_with_indent(&self, _db: &dyn Arch, indent_style: &str) -> Result<String> {
        if self.is_std_logic_vector() {
            Err(Error::BackEndError(
                "Invalid type, std_logic_vector cannot be declared.".to_string(),
            ))
        } else {
            Ok(indent(
                &format!(
                    "type {} is array ({} to {}) of ",
                    self.vhdl_name(),
                    self.low(),
                    self.high()
                ),
                indent_style,
            ))
        }
    }
}

impl Identify for ArrayObject {
    fn identifier(&self) -> String {
        self.vhdl_name().declare()
    }
}

impl VhdlNameSelf for ArrayObject {
    fn vhdl_name(&self) -> &VhdlName {
        &self.type_name
    }
}

impl DeclarationTypeName for ArrayObject {
    fn declaration_type_name(&self) -> String {
        if self.is_std_logic_vector() {
            format!("std_logic_vector({} downto {})", self.high(), self.low())
        } else {
            self.identifier()
        }
    }
}
