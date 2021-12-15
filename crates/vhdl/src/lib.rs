//! VHDL Properties
//!
//! The goal of this crate is to describe VHDL in Rust, for the purposes of design generation

pub mod architecture;
pub mod assignment;
pub mod component;
pub mod declaration;
pub mod entity;
pub mod object;
pub mod package;
pub mod port;
pub mod properties;
pub mod statement;
pub mod usings;

#[cfg(test)]
mod tests {
    // use crate::generator::common::test::{records, test_comp};

    // use super::*;

    // pub fn test_entity() -> Entity {
    //     Entity::from(test_comp())
    // }
}
