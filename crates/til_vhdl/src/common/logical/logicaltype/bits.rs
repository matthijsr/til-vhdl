use std::convert::TryFrom;

use tydi_common::{
    error::{Error, Result},
    numbers::NonNegative,
};

use crate::ir::LogicalType;

impl TryFrom<NonNegative> for LogicalType {
    type Error = Error;

    fn try_from(value: NonNegative) -> Result<Self> {
        LogicalType::try_new_bits(value)
    }
}
