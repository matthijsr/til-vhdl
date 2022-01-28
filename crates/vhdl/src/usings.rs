use std::collections::HashSet;

use indexmap::IndexMap;
use tydi_common::error::{Result, TryResult};

use crate::common::vhdl_name::{VhdlName, VhdlPathName};

/// A list of VHDL usings, indexed by library
#[derive(Debug, Clone)]
pub struct Usings(IndexMap<VhdlName, HashSet<VhdlPathName>>);

impl Usings {
    pub fn new_empty() -> Usings {
        Usings(IndexMap::new())
    }

    /// If the set did not have this value present, `true` is returned.
    ///
    /// If the set did have this value present, `false` is returned.
    pub fn add_using(
        &mut self,
        library: impl TryResult<VhdlName>,
        using: impl TryResult<VhdlPathName>,
    ) -> Result<bool> {
        Ok(self
            .0
            .entry(library.try_result()?)
            .or_insert(HashSet::new())
            .insert(using.try_result()?))
    }

    pub fn usings(&self) -> &IndexMap<VhdlName, HashSet<VhdlPathName>> {
        &self.0
    }

    /// Combine two usings
    pub fn combine(&mut self, other: &Usings) {
        for (library, using) in other.usings() {
            self.0.insert(library.clone(), using.clone());
        }
    }
}

pub trait ListUsings {
    fn list_usings(&self) -> Result<Usings>;
}

pub trait DeclareUsings {
    fn declare_usings(&self) -> Result<String>;
}

/// Generate supertrait for VHDL with usings declarations. (E.g. use ieee.std_logic_1164.all;)
impl<T: ListUsings> DeclareUsings for T {
    fn declare_usings(&self) -> Result<String> {
        let mut result = String::new();

        for (lib, usings) in self.list_usings()?.0 {
            result.push_str(format!("library {};\n", lib).as_str());
            for using in usings {
                result.push_str(format!("use {}.{};\n", lib, using).as_str());
            }
            result.push_str("\n");
        }

        Ok(result)
    }
}
