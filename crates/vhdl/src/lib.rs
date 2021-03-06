//! VHDL Properties
//!
//! The goal of this crate is to describe VHDL in Rust, for the purposes of design generation

pub mod architecture;
pub mod assignment;
pub mod common;
pub mod component;
pub mod declaration;
pub mod entity;
pub mod object;
pub mod package;
pub mod port;
pub mod process;
pub mod properties;
pub mod statement;
pub mod traits;
pub mod usings;

#[cfg(test)]
pub(crate) mod test_tools;

#[cfg(test)]
mod tests {}
