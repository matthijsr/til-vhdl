use crate::common::physical::Fields;

use super::LogicalType;

#[derive(Debug, Clone, PartialEq)]
pub struct Signals(LogicalType);
impl Signals {
    /// Returns the LogicalType of this element.
    pub fn logical_type(&self) -> &LogicalType {
        &self.0
    }
    /// Returns all fields in these async signals.
    pub fn fields(&self) -> Fields {
        self.0.fields()
    }
}
