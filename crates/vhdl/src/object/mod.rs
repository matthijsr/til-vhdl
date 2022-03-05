use std::{convert::TryInto, fmt};

use array::ArrayObject;
use record::RecordObject;
use tydi_common::{
    error::{Error, Result, TryResult},
    numbers::BitCount,
};

use crate::{
    architecture::arch_storage::Arch,
    assignment::{
        array_assignment::ArrayAssignment, Assignment, AssignmentKind, DirectAssignment,
        FieldSelection, RangeConstraint, ValueAssignment,
    },
    common::vhdl_name::VhdlName,
    declaration::{Declare, DeclareWithIndent},
    properties::Analyze,
};

pub mod array;
pub mod object_from;
pub mod record;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub struct Assignable {
    /// Can be assigned to
    pub to: bool,
    /// Can be assigned from
    pub from: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Object {
    typ: ObjectType,
    assignable: Assignable,
}

/// Types of VHDL objects, possibly referring to fields
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ObjectType {
    /// A bit object, can not contain further fields
    Bit,
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
                    record.type_name(),
                    fields
                )
            }
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
                                    array.type_name(),
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
                        "Cannot assign {} to Array",
                        typ
                    )))
                }
            }
            ObjectType::Record(to_record) => {
                if let ObjectType::Record(from_record) = typ {
                    if from_record.type_name() == to_record.type_name() {
                        Ok(())
                    } else {
                        Err(Error::InvalidTarget(format!(
                            "Cannot assign record type {} to record type {}",
                            from_record.type_name(),
                            to_record.type_name(),
                        )))
                    }
                } else {
                    Err(Error::InvalidTarget(format!(
                        "Cannot assign {} to {}",
                        typ, self
                    )))
                }
            }
        }
    }

    pub fn can_assign(&self, db: &dyn Arch, assignment: &Assignment) -> Result<()> {
        let mut to_object = self.clone();
        for field in assignment.to_field() {
            to_object = to_object.get_field(field)?;
        }
        match assignment.kind() {
            AssignmentKind::Object(object) => to_object.can_assign_type(&object.typ(db)?),
            AssignmentKind::Direct(direct) => match direct {
                DirectAssignment::Value(value) => match value {
                    ValueAssignment::Bit(_) => match to_object {
                        ObjectType::Bit => Ok(()),
                        ObjectType::Array(_) | ObjectType::Record(_) => Err(Error::InvalidTarget(
                            format!("Cannot assign Bit to {}", to_object),
                        )),
                    },
                    ValueAssignment::BitVec(bitvec) => match to_object {
                        ObjectType::Array(array) if array.is_bitvector() => {
                            bitvec.validate_width(array.width())
                        }
                        _ => Err(Error::InvalidTarget(format!(
                            "Cannot assign Bit Vector to {}",
                            to_object
                        ))),
                    },
                },
                DirectAssignment::FullRecord(record) => {
                    if let ObjectType::Record(to_record) = &to_object {
                        if to_record.fields().len() == record.len() {
                            for ra in record {
                                let to_field = to_object
                                    .get_field(&FieldSelection::name(ra.field().clone()))?;
                                to_field
                                    .can_assign(db, &Assignment::from(ra.assignment().clone()))?;
                            }
                            Ok(())
                        } else {
                            Err(Error::InvalidArgument(format!("Attempted full record assignment. Number of fields do not match. Record has {} fields, assignment has {} fields", to_record.fields().len(), record.len())))
                        }
                    } else {
                        Err(Error::InvalidTarget(format!(
                            "Cannot perform full Record assignment to {}",
                            to_object
                        )))
                    }
                }
                DirectAssignment::FullArray(array) => {
                    if let ObjectType::Array(to_array) = &to_object {
                        match array {
                            ArrayAssignment::Direct(direct) => {
                                if to_array.width() == direct.len().try_into().unwrap() {
                                    for value in direct {
                                        to_array
                                            .typ()
                                            .can_assign(db, &Assignment::from(value.clone()))?;
                                    }
                                    Ok(())
                                } else {
                                    Err(Error::InvalidArgument(format!("Attempted full array assignment. Number of fields do not match. Array has {} fields, assignment has {} fields", to_array.width(), direct.len())))
                                }
                            }
                            ArrayAssignment::Sliced { direct, others } => {
                                let mut ranges_assigned: Vec<&RangeConstraint> = vec![];
                                for ra in direct {
                                    let range = ra.constraint();
                                    if !range.is_between(to_array.high(), to_array.low())? {
                                        return Err(Error::InvalidArgument(format!(
                                            "{} is not between {} and {}",
                                            range,
                                            to_array.high(),
                                            to_array.low()
                                        )));
                                    }
                                    if ranges_assigned.iter().any(|x| x.overlaps(range)) {
                                        return Err(Error::InvalidArgument(format!("Sliced array assignment: {} overlaps with a range which was already assigned.", range)));
                                    }
                                    to_array.typ().can_assign(
                                        db,
                                        &Assignment::from(ra.assignment().clone()),
                                    )?;
                                    ranges_assigned.push(range);
                                }
                                let total_assigned: u32 =
                                    ranges_assigned.iter().map(|x| x.width_u32()).sum();
                                if total_assigned == to_array.width() {
                                    if let Some(_) = others {
                                        return Err(Error::InvalidArgument("Sliced array assignment contains an 'others' field, but already assigns all fields directly.".to_string()));
                                    } else {
                                        Ok(())
                                    }
                                } else {
                                    if let Some(value) = others {
                                        to_array.typ().can_assign(
                                            db,
                                            &Assignment::from(value.as_ref().clone()),
                                        )
                                    } else {
                                        Err(Error::InvalidArgument("Sliced array assignment does not assign all values directly, but does not contain an 'others' field.".to_string()))
                                    }
                                }
                            }
                            ArrayAssignment::Others(others) => to_array
                                .typ()
                                .can_assign(db, &Assignment::from(others.as_ref().clone())),
                        }
                    } else {
                        Err(Error::InvalidTarget(format!(
                            "Cannot perform full Array assignment to {}",
                            to_object
                        )))
                    }
                }
            },
        }
    }

    pub fn type_name(&self) -> String {
        match self {
            ObjectType::Bit => "std_logic".to_string(),
            ObjectType::Array(array) => array.type_name(),
            ObjectType::Record(record) => record.type_name(),
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
    use super::*;

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
