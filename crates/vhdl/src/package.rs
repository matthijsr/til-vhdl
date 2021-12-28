use std::collections::{HashMap, HashSet};

use itertools::Itertools;
use textwrap::indent;
use tydi_common::{
    error::{Error, Result},
    name::Name,
    traits::Identify,
};

use crate::{
    architecture::arch_storage::Arch,
    component::Component,
    declaration::{Declare, DeclareWithIndent},
    object::ObjectType,
    properties::Analyze,
    usings::{DeclareUsings, ListUsings, Usings},
};

// TODO: Eventually functions as well.
/// A library of components and types.
#[derive(Debug)]
pub struct Package {
    /// The identifier.
    identifier: Name,
    /// The components declared within the library.
    components: Vec<Component>,
    /// The types declared within the library.
    types: Vec<ObjectType>,
}

impl Package {
    pub fn new(identifier: Name, components: &Vec<Component>, types: &Vec<ObjectType>) -> Self {
        let mut all_types: Vec<ObjectType> = types.clone();
        for component in components {
            all_types.append(&mut component.list_nested_types());
        }
        Package {
            identifier,
            components: components.clone(),
            types: all_types.into_iter().unique_by(|x| x.type_name()).collect(),
        }
    }

    /// Creates an empty "work" library
    pub fn new_default_empty() -> Self {
        Package::new(Name::try_new("work").unwrap(), &vec![], &vec![])
    }

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

    pub fn add_component(&mut self, component: Component) {
        self.components.push(component);
    }

    pub fn add_type(&mut self, typ: ObjectType) {
        self.types.push(typ);
    }

    pub fn components(&self) -> &Vec<Component> {
        &self.components
    }

    pub fn types(&self) -> &Vec<ObjectType> {
        &self.types
    }
}

impl DeclareWithIndent for Package {
    fn declare_with_indent(&self, db: &dyn Arch, pre: &str) -> Result<String> {
        let mut result = String::new();
        result.push_str(self.declare_usings()?.as_str());
        result.push_str(format!("package {} is\n\n", self.identifier).as_str());

        let mut body = String::new();
        for t in self.types() {
            body.push_str(format!("{}\n\n", t.declare_with_indent(db, pre)?).as_str());
        }
        for c in &self.components {
            body.push_str(format!("{}\n\n", c.declare(db)?).as_str());
        }
        result.push_str(&indent(&body, pre));
        result.push_str(format!("end {};", self.identifier).as_str());

        Ok(result)
    }
}

// NOTE: ListUsings is overkill for Packages as-is (could be simple constants, as they always use ieee.std_logic and nothing else), but serves as a decent example.
impl ListUsings for Package {
    fn list_usings(&self) -> Result<Usings> {
        let mut usings = Usings::new_empty();
        let mut types = self
            .components
            .iter()
            .flat_map(|x| x.ports().iter().map(|p| p.typ()));
        fn uses_std_logic(t: &ObjectType) -> bool {
            match t {
                ObjectType::Bit => true,
                ObjectType::Array(array_object) => {
                    array_object.is_bitvector() || uses_std_logic(array_object.typ())
                }
                ObjectType::Record(record_object) => record_object
                    .fields()
                    .into_iter()
                    .any(|(_, typ)| uses_std_logic(&typ)),
            }
        }

        if types.any(|x| uses_std_logic(&x)) {
            usings.add_using(Name::try_new("ieee")?, "std_logic_1164.all".to_string());
        }

        Ok(usings)
    }
}

impl Identify for Package {
    fn identifier(&self) -> &str {
        self.identifier.as_ref()
    }
}
