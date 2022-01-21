use std::convert::TryInto;

use tydi_common::{
    error::{Error, Result},
    name::Name,
};

use crate::{architecture::arch_storage::Arch, declaration::Declare, object::ObjectType};

/// An array object, arrays contain a single type of object, but can contain nested objects
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArrayObject {
    high: i32,
    low: i32,
    typ: Box<ObjectType>,
    type_name: String,
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
                high,
                low,
                typ: Box::new(ObjectType::Bit),
                type_name: format!("std_logic_vector({} downto {})", high, low),
                is_std_logic_vector: true,
            })
        }
    }

    /// Create an array of a specific field type
    pub fn array(
        high: i32,
        low: i32,
        object: ObjectType,
        type_name: impl Into<Name>,
    ) -> Result<ArrayObject> {
        if low > high {
            Err(Error::InvalidArgument(format!(
                "{} > {}! Low must be lower than high",
                low, high
            )))
        } else {
            Ok(ArrayObject {
                high,
                low,
                typ: Box::new(object),
                type_name: type_name.into().to_string(),
                is_std_logic_vector: false,
            })
        }
    }

    pub fn typ(&self) -> &ObjectType {
        &self.typ
    }

    pub fn high(&self) -> i32 {
        self.high
    }

    pub fn low(&self) -> i32 {
        self.low
    }

    pub fn width(&self) -> u32 {
        (1 + self.high - self.low).try_into().unwrap()
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

    pub fn type_name(&self) -> String {
        self.type_name.clone()
    }
}

impl Declare for ArrayObject {
    fn declare(&self, _db: &dyn Arch) -> Result<String> {
        if self.is_std_logic_vector() {
            Err(Error::BackEndError(
                "Invalid type, std_logic_vector cannot be declared.".to_string(),
            ))
        } else {
            Ok(format!(
                "type {} is array ({} to {}) of ",
                self.type_name(),
                self.low(),
                self.high()
            ))
        }
    }
}
