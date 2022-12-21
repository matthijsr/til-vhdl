use crate::{assignment::*, common::vhdl_name::VhdlName};
use tydi_common::error::{Error, Result, TryResult};

/// Quick way to get the minimum number of binary values required for an unsigned integer
pub fn min_length_unsigned(value: u32) -> u32 {
    if value == 0 {
        1
    } else {
        32 - value.leading_zeros()
    }
}

/// Quick way to get the minimum number of binary values required for a signed integer
pub fn min_length_signed(value: i32) -> u32 {
    if value == 0 {
        1
    } else if value < 0 {
        33 - value.leading_ones()
    } else {
        33 - value.leading_zeros()
    }
}

/// Source of the width for a conversion between a number and an std_logic_vector
///
/// The predetermined values will always succeed, but may produce invalid VHDL.
///
/// The Auto option is more likely to fail earlier, but only works when assigning
/// a value to an object.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WidthSource {
    /// A specific object's length
    Object(VhdlName),
    /// A constant value
    Constant(u32),
    /// Determine automatically at declaration (this may fail)
    Auto,
}

/// A struct for describing value assigned to a bit vector
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BitVecValue {
    /// Value assigned as (others => value)
    Others(StdLogicValue),
    /// A full, specific range of std_logic values is assigned
    ///
    /// Result (example): "01-XULH"
    Full(Vec<StdLogicValue>),
    /// A value is assigned from an unsigned integer
    ///
    /// Result: std_logic_vector(to_unsigned([value], [name]'length))
    ///
    /// Or: std_logic_vector(to_unsigned([value], [range length]))
    Unsigned(u32, WidthSource),
    /// A value is assigned from a signed integer
    ///
    /// Result: std_logic_vector(to_signed([value], [name]'length))
    ///
    /// Or: std_logic_vector(to_signed([value], [range length]))
    Signed(i32, WidthSource),
}

impl BitVecValue {
    /// Create a bit vector value from a string
    pub fn from_str(value: &str) -> Result<BitVecValue> {
        let logicvals = value
            .chars()
            .map(StdLogicValue::from_char)
            .collect::<Result<Vec<StdLogicValue>>>()?;
        Ok(BitVecValue::Full(logicvals))
    }

    pub fn validate_width(&self, width: u32) -> Result<()> {
        match self {
            BitVecValue::Others(_) => Ok(()),
            BitVecValue::Full(full) => {
                if full.len() == width.try_into().unwrap() {
                    Ok(())
                } else {
                    Err(Error::InvalidArgument(format!(
                        "Value with length {} cannot be assigned to bit vector with length {}",
                        full.len(),
                        width
                    )))
                }
            }
            BitVecValue::Unsigned(value, _) => {
                if min_length_unsigned(*value) > width {
                    Err(Error::InvalidArgument(format!(
                        "Cannot assign unsigned integer {} to range with width {}",
                        value, width
                    )))
                } else {
                    Ok(())
                }
            }
            BitVecValue::Signed(value, _) => {
                if min_length_signed(*value) > width {
                    Err(Error::InvalidArgument(format!(
                        "Cannot assign signed integer {} to range with width {}",
                        value, width
                    )))
                } else {
                    Ok(())
                }
            }
        }
    }

    pub fn declare(&self) -> Result<String> {
        match self {
            BitVecValue::Others(value) => Ok(format!("(others => '{}')", value)),
            BitVecValue::Full(bitvec) => {
                let mut result = String::new();
                for value in bitvec {
                    result.push_str(value.to_string().as_str());
                }
                Ok(format!("\"{}\"", result))
            }
            BitVecValue::Unsigned(value, WidthSource::Object(name)) => Ok(format!(
                "std_logic_vector(to_unsigned({}, {}'length))",
                value,
                name
            )),
            BitVecValue::Signed(value, WidthSource::Object(name)) => Ok(format!(
                "std_logic_vector(to_signed({}, {}'length))",
                value,
                name
            )),
            BitVecValue::Unsigned(value, WidthSource::Constant(width)) => Ok(format!(
                "std_logic_vector(to_unsigned({}, {}))",
                value,
                width
            )),
            BitVecValue::Signed(value, WidthSource::Constant(width)) => Ok(format!(
                "std_logic_vector(to_signed({}, {}))",
                value,
                width
            )),
            BitVecValue::Unsigned(_, WidthSource::Auto) | BitVecValue::Signed(_, WidthSource::Auto) => Err(Error::InvalidTarget("Unable to declare bit vector value, signed and unsigned values require a width or object identifier.".to_string())),
        }
    }

    /// Declares the value assigned for the object being assigned to (identifier required in case Range is empty)
    pub fn declare_for(&self, object_identifier: impl TryResult<VhdlName>) -> Result<String> {
        Ok(match self {
            BitVecValue::Others(_) | BitVecValue::Full(_) => self.declare().unwrap(),
            BitVecValue::Unsigned(value, _) => format!(
                "std_logic_vector(to_unsigned({}, {}'length))",
                value,
                object_identifier.try_result()?
            ),
            BitVecValue::Signed(value, _) => format!(
                "std_logic_vector(to_signed({}, {}'length))",
                value,
                object_identifier.try_result()?
            ),
        })
    }

    /// Declares the value assigned for the range being assigned to
    pub fn declare_for_range(&self, range: &RangeConstraint) -> Result<String> {
        match self {
            BitVecValue::Others(_) | BitVecValue::Full(_) => self.declare(),
            BitVecValue::Unsigned(value, _) => match range.width()? {
                Some(Width::Scalar) => Err(Error::InvalidTarget(
                    "Cannot assign an std_logic_vector(unsigned) to indexed std_logic".to_string(),
                )),
                Some(Width::Vector(width)) => {
                    self.validate_width(width)?;
                    Ok(format!(
                        "std_logic_vector(to_unsigned({}, {}))",
                        value, width
                    ))
                }
                _ => todo!(),
            },
            BitVecValue::Signed(value, _) => match range.width()? {
                Some(Width::Scalar) => Err(Error::InvalidTarget(
                    "Cannot assign an std_logic_vector(signed) to indexed std_logic".to_string(),
                )),
                Some(Width::Vector(width)) => {
                    self.validate_width(width)?;
                    Ok(format!("std_logic_vector(to_signed({}, {}))", value, width))
                }
                _ => todo!(),
            },
        }
    }

    pub fn matching_bitvec(&self, other: &BitVecValue) -> bool {
        match self {
            BitVecValue::Others(_) => match other {
                BitVecValue::Others(_) => true,
                _ => false,
            },
            BitVecValue::Full(f) => match other {
                BitVecValue::Full(o_f) => f.len() == o_f.len(),
                _ => false,
            },
            BitVecValue::Unsigned(_, _) => match other {
                BitVecValue::Unsigned(_, _) | BitVecValue::Signed(_, _) => true,
                _ => false,
            },
            BitVecValue::Signed(_, _) => match other {
                BitVecValue::Unsigned(_, _) | BitVecValue::Signed(_, _) => true,
                _ => false,
            },
        }
    }
}

impl<T: IntoIterator<Item = bool>> From<T> for BitVecValue {
    fn from(it: T) -> Self {
        let mut res_vec = vec![];
        for val in it {
            res_vec.push(StdLogicValue::Logic(val));
        }
        BitVecValue::Full(res_vec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn min_length_signed_test() {
        assert_eq!(1, min_length_signed(0));
        assert_eq!(1, min_length_signed(-1));
        assert_eq!(2, min_length_signed(1));
        assert_eq!(32, min_length_signed(i32::MIN));
        assert_eq!(32, min_length_signed(i32::MAX));
    }

    #[test]
    fn min_length_unsigned_test() {
        assert_eq!(1, min_length_unsigned(0));
        assert_eq!(1, min_length_unsigned(1));
        assert_eq!(2, min_length_unsigned(2));
        assert_eq!(32, min_length_unsigned(u32::MAX));
    }
}
