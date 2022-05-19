use std::sync::Arc;

use tydi_common::{
    error::{Error, Result, WrapError},
    map::InsertionOrderedMap,
    name::{Name, NameSelf},
    traits::{Document, Documents, Identify},
};

use crate::ir::{interface_port::InterfacePort, physical_properties::Domain, streamlet::Streamlet};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ListOrDefault {
    /// Definition has a list of Domains, and is assigned either a Default
    /// Domain (None), or a named Domain (Some).
    List(InsertionOrderedMap<Domain, Option<Domain>>),
    /// Definition has a Default Domain, and is assigned either a Default
    /// Domain (None), or a named Domain (Some).
    Default(Option<Domain>),
}

impl ListOrDefault {
    fn get_assignment_default(&self) -> Result<Option<&Domain>> {
        match self {
            ListOrDefault::List(_) => Err(Error::ProjectError(
                "Attempting to determine domain for Default, but Streamlet has named Domains."
                    .to_string(),
            )),
            ListOrDefault::Default(res) => Ok(res.as_ref()),
        }
    }

    pub fn get_assignment(&self, base_domain: Option<&Domain>) -> Result<Option<&Domain>> {
        match base_domain {
            Some(base_domain_name) => match self {
                ListOrDefault::List(domain_map) => Ok(domain_map
                    .try_get(base_domain_name)
                    .wrap_err(Error::ProjectError(format!(
                        "Cannot find assigned domain for {}, no such domain on Interface",
                        base_domain_name
                    )))?
                    .as_ref()),
                ListOrDefault::Default(res) => Ok(res.as_ref()),
            },
            None => self.get_assignment_default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StreamletInstance {
    name: Name,
    definition: Arc<Streamlet>,
    domain_assignments: ListOrDefault,
    ports: InsertionOrderedMap<Name, InterfacePort>,
    doc: Option<String>,
}

impl Document for StreamletInstance {
    fn doc(&self) -> Option<&String> {
        self.doc.as_ref()
    }
}

impl Documents for StreamletInstance {
    fn set_doc(&mut self, doc: impl Into<String>) {
        self.doc = Some(doc.into())
    }
}

impl Identify for StreamletInstance {
    fn identifier(&self) -> String {
        self.name.to_string()
    }
}

impl NameSelf for StreamletInstance {
    fn name(&self) -> &Name {
        &self.name
    }
}
