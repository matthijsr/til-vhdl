use std::collections::BTreeSet;

use tydi_common::{
    error::{Result, TryResult},
    map::InsertionOrderedMap,
};

use crate::common::vhdl_name::{VhdlName, VhdlPathName};

/// A list of VHDL usings, indexed by library
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Usings(InsertionOrderedMap<VhdlName, BTreeSet<VhdlPathName>>);

impl Usings {
    pub fn new_empty() -> Usings {
        Usings(InsertionOrderedMap::new())
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
            .get_or_insert(&library.try_result()?, BTreeSet::new())
            .insert(using.try_result()?))
    }

    pub fn usings(&self) -> &InsertionOrderedMap<VhdlName, BTreeSet<VhdlPathName>> {
        &self.0
    }

    /// Combine two usings
    pub fn combine(&mut self, other: &Usings) {
        for (library, using) in other.usings() {
            if let Some(existing) = self.0.insert_or_replace(library.clone(), using.clone()) {
                self.0.get_mut(library).unwrap().extend(existing);
            }
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
