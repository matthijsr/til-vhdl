use std::sync::Arc;

use indexmap::IndexMap;
use itertools::Itertools;
use textwrap::indent;

use tydi_common::{
    error::{Error, Result, TryResult},
    traits::Identify,
};

use crate::object::object_type::ObjectType;
use crate::{
    architecture::arch_storage::Arch,
    common::vhdl_name::{VhdlName, VhdlNameSelf},
    component::Component,
    declaration::{Declare, DeclareWithIndent},
    object::object_type::DeclarationTypeName,
    properties::Analyze,
    usings::{DeclareUsings, ListUsings, Usings},
};

// TODO: Eventually functions as well.
/// A library of components and types.
#[derive(Debug, Clone)]
pub struct Package {
    /// The identifier.
    identifier: VhdlName,
    /// The components declared within the library.
    components: IndexMap<VhdlName, Arc<Component>>,
    /// The types declared within the library.
    types: Vec<ObjectType>,
}

impl Package {
    pub fn new_named(identifier: impl TryResult<VhdlName>) -> Result<Self> {
        Ok(Package {
            identifier: identifier.try_result()?,
            components: IndexMap::new(),
            types: vec![],
        })
    }

    pub fn try_new(
        db: &dyn Arch,
        identifier: impl TryResult<VhdlName>,
        components: &Vec<Arc<Component>>,
        types: &Vec<ObjectType>,
    ) -> Result<Self> {
        let mut all_types: Vec<ObjectType> = types.clone();
        for component in components {
            all_types.append(&mut component.list_nested_types(db));
        }
        Ok(Package {
            identifier: identifier.try_result()?,
            components: components
                .iter()
                .map(|c| (c.vhdl_name().clone(), c.clone()))
                .collect(),
            types: all_types
                .into_iter()
                .unique_by(|x| x.declaration_type_name(db))
                .collect(),
        })
    }

    /// Creates an empty "default" library
    pub fn new_default_empty() -> Self {
        Package {
            identifier: "default".try_into().unwrap(),
            components: IndexMap::new(),
            types: vec![],
        }
    }

    pub fn get_subject_component(&self, db: &dyn Arch) -> Result<Arc<Component>> {
        let subj_name = db.subject_component_name();
        match self.components.get(subj_name.as_ref()) {
            Some(component) => Ok(component.clone()),
            None => Err(Error::LibraryError(format!(
                "Subject Component with identifier {} does not exist in package.",
                subj_name
            ))),
        }
    }

    pub fn get_component(&self, identifier: impl TryResult<VhdlName>) -> Result<&Component> {
        let identifier = identifier.try_result()?;
        match self.components.get(&identifier) {
            Some(component) => Ok(component),
            None => Err(Error::LibraryError(format!(
                "Component with identifier {} does not exist in package.",
                identifier
            ))),
        }
    }

    pub fn add_component(&mut self, component: Arc<Component>) {
        self.components
            .insert(component.vhdl_name().clone(), component);
    }

    pub fn add_type(&mut self, typ: ObjectType) {
        self.types.push(typ);
    }

    pub fn components(&self) -> &IndexMap<VhdlName, Arc<Component>> {
        &self.components
    }

    pub fn types(&self) -> &Vec<ObjectType> {
        &self.types
    }
}

impl DeclareWithIndent for Package {
    fn declare_with_indent(&self, db: &dyn Arch, indent_style: &str) -> Result<String> {
        let mut result = String::new();
        result.push_str(self.declare_usings()?.as_str());
        result.push_str(format!("package {} is\n\n", self.identifier).as_str());

        let mut body = String::new();
        for t in self.types() {
            body.push_str(format!("{}\n\n", t.declare_with_indent(db, indent_style)?).as_str());
        }
        for (_, c) in &self.components {
            body.push_str(format!("{}\n\n", c.declare(db)?).as_str());
        }
        result.push_str(&indent(&body, indent_style));
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
            .flat_map(|(_, x)| x.ports().iter().map(|(_, p)| p.typ()));
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
                ObjectType::Time => false,
                ObjectType::Boolean => false,
                ObjectType::Integer(_) => false,
            }
        }

        if types.any(|x| uses_std_logic(&x)) {
            usings.add_using(VhdlName::try_new("ieee")?, "std_logic_1164.all".to_string())?;
        }

        Ok(usings)
    }
}

impl Identify for Package {
    fn identifier(&self) -> String {
        self.identifier.declare()
    }
}

impl VhdlNameSelf for Package {
    fn vhdl_name(&self) -> &VhdlName {
        &self.identifier
    }
}
