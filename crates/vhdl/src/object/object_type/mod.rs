pub mod severity;
pub mod time;

use std::convert::TryInto;
use std::fmt;

use tydi_common::error::Result;
use tydi_common::error::{Error, TryResult};
use tydi_common::numbers::BitCount;
use tydi_common::traits::Identify;

use crate::architecture::arch_storage::Arch;
use crate::assignment::{FieldSelection, RangeConstraint};
use crate::common::vhdl_name::{VhdlName, VhdlNameSelf};
use crate::declaration::{Declare, DeclareWithIndent};
use crate::object::array::ArrayObject;
use crate::object::record::RecordObject;
use crate::properties::Analyze;

pub trait DeclarationTypeName {
    /// Returns the type name for use in object declaration
    fn declaration_type_name(&self) -> String;
}

/// Basic signed number type defined in the std package, typically 32 bits,
/// but this depends on the implementation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IntegerType {
    /// Supports both negative and positive values
    Integer,
    /// Supports values 0 and up, subset of Integer
    Natural,
    /// Supports values 1 and up, subset of Integer
    Positive,
}

impl fmt::Display for IntegerType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IntegerType::Integer => write!(f, "Integer"),
            IntegerType::Natural => write!(f, "Natural"),
            IntegerType::Positive => write!(f, "Positive"),
        }
    }
}

impl DeclarationTypeName for IntegerType {
    fn declaration_type_name(&self) -> String {
        match self {
            IntegerType::Integer => "integer".to_string(),
            IntegerType::Natural => "natural".to_string(),
            IntegerType::Positive => "positive".to_string(),
        }
    }
}

/// Types of VHDL objects, possibly referring to fields
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ObjectType {
    // TODO: Add Time, Delay_length, Boolean, Real, Character, String, Severity (Possibly combine into "Abstract")
    // Note that Natural (>=0) and Positive (>=1) are just subtypes of Integer, Delay is a subtype (>=0) of Time, String is an array of Characters
    // File-related types are currently not necessary.
    //
    // Strictly speaking, all types can be categorized in four primitive types (scalar, composite, access and file)
    // "file" is currently irrelevant, and "access" (pointer-esque data manipulation) is unlikely to ever be relevant.
    // "composite" covers Array and Record
    // "scalar" is further split into discrete, floating point and physical.
    // physical refers to Time (and by extension, Delay_length)
    // floating point refers to Real
    // discrete is further split into integer and enumeration
    // integer refers to Integer (and by extension, Natural and Positive)
    // enumeration is any type (a, b, c), which covers bits, characters, booleans, severity level, file status, etc.
    //
    // I'm not sure adhering to these strict categories will really make things easier, however.
    // They might make sense for a "Custom" type, instead.
    /// A boolean object
    Boolean,
    /// A time object
    Time,
    /// An std_logic bit object, can not contain further fields
    Bit,
    /// Basic signed number type defined in the std package, typically 32 bits,
    /// but this depends on the implementation
    Integer(IntegerType),
    /// An array of fields, covers both conventional arrays, as well as bit vectors
    Array(ArrayObject),
    /// A record object, consisting of named fields
    Record(RecordObject),
}

impl fmt::Display for ObjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObjectType::Bit => write!(f, "Bit"),
            ObjectType::Array(array) => write!(
                f,
                "Array ({} to {}) containing {}",
                array.low(),
                array.high(),
                array.typ()
            ),
            ObjectType::Record(record) => {
                let mut fields = String::new();
                for (name, typ) in record.fields() {
                    fields.push_str(format!("{}: {} ", name, typ).as_str());
                }
                write!(
                    f,
                    "Record (type name: {}) with fields: ( {})",
                    record.declaration_type_name(),
                    fields
                )
            }
            ObjectType::Time => write!(f, "Time"),
            ObjectType::Boolean => write!(f, "Boolean"),
            ObjectType::Integer(int_typ) => write!(f, "Integer({})", int_typ),
        }
    }
}

impl ObjectType {
    pub fn get_field(&self, field: &FieldSelection) -> Result<ObjectType> {
        match self {
            ObjectType::Bit => Err(Error::InvalidTarget(
                "Cannot select a field on a Bit".to_string(),
            )),
            ObjectType::Array(array) => match field {
                FieldSelection::Range(range) => {
                    if let RangeConstraint::Index(index) = range {
                        if *index <= array.high() && *index >= array.low() {
                            Ok(array.typ().clone())
                        } else {
                            Err(Error::InvalidArgument(format!(
                                "Cannot select index {} on array with high: {}, low: {}",
                                index,
                                array.high(),
                                array.low()
                            )))
                        }
                    } else {
                        if range.is_between(array.high(), array.low())? {
                            if array.is_std_logic_vector() {
                                ObjectType::bit_vector(range.high(), range.low())
                            } else {
                                ObjectType::array(
                                    range.high(),
                                    range.low(),
                                    array.typ().clone(),
                                    array.vhdl_name().clone(),
                                )
                            }
                        } else {
                            Err(Error::InvalidArgument(format!(
                                "Cannot select {} on array with high: {}, low: {}",
                                range,
                                array.high(),
                                array.low()
                            )))
                        }
                    }
                }
                FieldSelection::Name(_) => Err(Error::InvalidTarget(
                    "Cannot select a named field on an array".to_string(),
                )),
            },
            ObjectType::Record(record) => match field {
                FieldSelection::Range(_) => Err(Error::InvalidTarget(
                    "Cannot select a range on a record".to_string(),
                )),
                FieldSelection::Name(name) => Ok(record.get_field(name)?.clone()),
            },
            ObjectType::Time => Err(Error::InvalidTarget(
                "Cannot select a field on a Time".to_string(),
            )),
            ObjectType::Boolean => Err(Error::InvalidTarget(
                "Cannot select a field on a Boolean".to_string(),
            )),
            ObjectType::Integer(_) => Err(Error::InvalidTarget(
                "Cannot select a field on an Integer".to_string(),
            )),
        }
    }

    pub fn get_nested(&self, nested: &Vec<FieldSelection>) -> Result<ObjectType> {
        let mut result = self.clone();
        for field in nested {
            result = result.get_field(field)?;
        }
        Ok(result)
    }

    /// Create an array of a specific field type
    pub fn array(
        high: i32,
        low: i32,
        object: ObjectType,
        type_name: impl TryResult<VhdlName>,
    ) -> Result<ObjectType> {
        Ok(ObjectType::Array(ArrayObject::array(
            high, low, object, type_name,
        )?))
    }

    /// Create a bit vector object
    pub fn bit_vector(high: i32, low: i32) -> Result<ObjectType> {
        Ok(ArrayObject::bit_vector(high, low)?.into())
    }

    /// Test whether two `ObjectType`s can be assigned to one another
    pub fn can_assign_type(&self, typ: &ObjectType) -> Result<()> {
        match self {
            ObjectType::Bit => {
                if let ObjectType::Bit = typ {
                    Ok(())
                } else {
                    Err(Error::InvalidTarget(format!(
                        "Cannot assign {} to Bit",
                        typ
                    )))
                }
            }
            ObjectType::Array(to_array) => {
                if let ObjectType::Array(from_array) = typ {
                    if from_array.identifier() == to_array.identifier() {
                        if from_array.width() == to_array.width() {
                            to_array.typ().can_assign_type(from_array.typ())
                        } else {
                            Err(Error::InvalidTarget(format!(
                                "Cannot assign array with width {} to array with width {}",
                                from_array.width(),
                                to_array.width(),
                            )))
                        }
                    } else {
                        Err(Error::InvalidTarget(format!(
                            "Cannot assign array type {} to array type {}",
                            from_array.identifier(),
                            to_array.identifier(),
                        )))
                    }
                } else {
                    Err(Error::InvalidTarget(format!(
                        "Cannot assign {} to Array",
                        typ
                    )))
                }
            }
            ObjectType::Record(to_record) => {
                if let ObjectType::Record(from_record) = typ {
                    if from_record.identifier() == to_record.identifier() {
                        Ok(())
                    } else {
                        Err(Error::InvalidTarget(format!(
                            "Cannot assign record type {} to record type {}",
                            from_record.identifier(),
                            to_record.identifier(),
                        )))
                    }
                } else {
                    Err(Error::InvalidTarget(format!(
                        "Cannot assign {} to {}",
                        typ, self
                    )))
                }
            }
            ObjectType::Time => {
                if let ObjectType::Time = typ {
                    Ok(())
                } else {
                    Err(Error::InvalidTarget(format!(
                        "Cannot assign {} to Time",
                        typ
                    )))
                }
            }
            ObjectType::Boolean => {
                if let ObjectType::Boolean = typ {
                    Ok(())
                } else {
                    Err(Error::InvalidTarget(format!(
                        "Cannot assign {} to Boolean",
                        typ
                    )))
                }
            }
            ObjectType::Integer(_) => {
                // All subsets of Integer can be assigned to one another, though
                // this may result in unexpected behavior. The assumption being
                // that people know what they're doing.
                // TODO: May want to add some sort of "warn" here
                if let ObjectType::Integer(_) = typ {
                    Ok(())
                } else {
                    Err(Error::InvalidTarget(format!(
                        "Cannot assign {} to Integer",
                        typ
                    )))
                }
            }
        }
    }

    /// Returns true if the object is a Bit or Bit Vector
    pub fn is_flat(&self) -> bool {
        match self {
            ObjectType::Bit => true,
            ObjectType::Array(arr) if arr.is_bitvector() => true,
            _ => false,
        }
    }
}

impl DeclarationTypeName for ObjectType {
    fn declaration_type_name(&self) -> String {
        match self {
            ObjectType::Bit => "std_logic".to_string(),
            ObjectType::Array(array) => array.declaration_type_name(),
            ObjectType::Record(record) => record.declaration_type_name(),
            ObjectType::Time => "time".to_string(),
            ObjectType::Boolean => "boolean".to_string(),
            ObjectType::Integer(int_typ) => int_typ.declaration_type_name(),
        }
    }
}

impl Analyze for ObjectType {
    fn list_nested_types(&self) -> Vec<ObjectType> {
        match self {
            ObjectType::Bit => vec![],
            ObjectType::Array(array_object) => {
                if array_object.is_std_logic_vector() {
                    vec![]
                } else {
                    let mut result = array_object.typ().list_nested_types();
                    result.push(self.clone());
                    result
                }
            }
            ObjectType::Record(record_object) => {
                let mut result = vec![];
                for (_, typ) in record_object.fields() {
                    result.append(&mut typ.list_nested_types())
                }
                result.push(self.clone());
                result
            }
            ObjectType::Time => vec![],
            ObjectType::Boolean => vec![],
            ObjectType::Integer(_) => vec![],
        }
    }
}

impl DeclareWithIndent for ObjectType {
    fn declare_with_indent(&self, db: &dyn Arch, _indent_style: &str) -> Result<String> {
        match self {
            ObjectType::Bit => Err(Error::BackEndError(
                "Invalid type, Bit (std_logic) cannot be declared.".to_string(),
            )),
            ObjectType::Array(array_object) => array_object.declare(db),
            ObjectType::Record(_) => todo!(),
            ObjectType::Time |
            ObjectType::Boolean |
            ObjectType::Integer(_) => Err(Error::BackEndError(format!(
                "Invalid type, {} ({}) cannot be declared.",
                self,
                self.declaration_type_name(),
            ))),
        }
    }
}

impl From<BitCount> for ObjectType {
    fn from(bits: BitCount) -> Self {
        if bits == BitCount::new(1).unwrap() {
            ObjectType::Bit
        } else {
            ObjectType::bit_vector((bits.get() - 1).try_into().unwrap(), 0).unwrap()
        }
    }
}

impl<T> From<std::ops::Range<T>> for ObjectType
where
    T: Into<i32>,
{
    fn from(range: std::ops::Range<T>) -> Self {
        let start = range.start.into();
        let end = range.end.into();
        let (high, low) = if start > end {
            (start, end)
        } else {
            (end, start)
        };
        ObjectType::bit_vector(high, low).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::object::*;

    #[test]
    fn bit_vector_from_range() {
        assert_eq!(
            ObjectType::from(0..2),
            ObjectType::bit_vector(2, 0).unwrap()
        );
        assert_eq!(
            ObjectType::from(2..0),
            ObjectType::bit_vector(2, 0).unwrap()
        );
    }
}
