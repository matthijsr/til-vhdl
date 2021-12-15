use std::collections::HashMap;

use tydi_common::{
    error::{Error, Result},
    traits::Identify,
};

use crate::{component::Component, declaration::Declare};

/// A library of components and types.
#[derive(Debug)]
pub struct Package {
    /// The identifier.
    pub identifier: String,
    /// The components declared within the library.66
    pub components: Vec<Component>,
}

impl Package {
    pub fn get_component(&self, identifier: impl Into<String>) -> Result<Component> {
        let identifier = identifier.into();
        match self
            .components
            .iter()
            .find(|x| x.identifier() == &identifier)
        {
            Some(component) => Ok(component.clone()),
            None => Err(Error::LibraryError(format!(
                "Component with identifier {} does not exist in package.",
                identifier
            ))),
        }
    }
}

// TODO

// impl Declare for Package {
//     fn declare(&self) -> Result<String> {
//         let mut result = String::new();
//         result.push_str(self.declare_usings()?.as_str());
//         result.push_str(format!("package {} is\n\n", self.identifier).as_str());

//         // Whatever generated the common representation is responsible to not to use the same
//         // identifiers for different types.
//         // Use a set to remember which type identifiers we've already used, so we don't declare
//         // them twice, and produce an error otherwise.
//         let mut type_ids = HashMap::<String, Type>::new();
//         for c in &self.components {
//             let comp_nested = c.list_nested_types();
//             for t in comp_nested.iter() {
//                 match type_ids.get(&t.vhdl_identifier()?) {
//                     None => {
//                         type_ids.insert(t.vhdl_identifier()?, t.clone());
//                         result.push_str(format!("{}\n\n", t.declare(true)?).as_str());
//                     }
//                     Some(already_defined_type) => {
//                         if t != already_defined_type {
//                             return Err(BackEndError(format!(
//                                 "Type name conflict: {}",
//                                 already_defined_type
//                                     .vhdl_identifier()
//                                     .unwrap_or_else(|_| "".to_string())
//                             )));
//                         }
//                     }
//                 }
//             }
//             result.push_str(format!("{}\n\n", c.declare()?).as_str());
//         }
//         result.push_str(format!("end {};", self.identifier).as_str());

//         Ok(result)
//     }
// }

// // NOTE: ListUsings is overkill for Packages as-is (could be simple constants, as they always use ieee.std_logic and nothing else), but serves as a decent example.
// impl ListUsings for Package {
//     fn list_usings(&self) -> Result<Usings> {
//         let mut usings = Usings::new_empty();
//         let mut types = self
//             .components
//             .iter()
//             .flat_map(|x| x.ports().iter().map(|p| p.typ()));
//         fn uses_std_logic(t: &Type) -> bool {
//             match t {
//                 Type::Bit => true,
//                 Type::BitVec { width: _ } => true,
//                 Type::Record(rec) => rec.fields().any(|field| uses_std_logic(field.typ())),
//                 Type::Union(rec) => rec.fields().any(|field| uses_std_logic(field.typ())),
//                 Type::Array(arr) => uses_std_logic(arr.typ()),
//             }
//         }

//         if types.any(|x| uses_std_logic(&x)) {
//             usings.add_using(Name::try_new("ieee")?, "std_logic_1164.all".to_string());
//         }

//         Ok(usings)
//     }
// }
