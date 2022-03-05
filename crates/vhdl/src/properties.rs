use tydi_common::numbers::NonNegative;

use crate::object::object_type::ObjectType;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Width {
    /// Non-vectorized single bit.
    Scalar,
    /// Vectorized multiple bits.
    Vector(NonNegative),
}

/// Analyze trait for VHDL objects.
pub trait Analyze {
    /// List all nested types used.
    fn list_nested_types(&self) -> Vec<ObjectType>;
}
