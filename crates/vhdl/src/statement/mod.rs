use std::collections::HashMap;

use indexmap::IndexMap;
use tydi_common::{
    error::{Error, Result},
    traits::Identify,
};
use tydi_intern::Id;

use crate::{
    architecture::arch_storage::Arch, assignment::Assign, component::Component,
    declaration::ObjectState,
};

use super::{
    assignment::{AssignDeclaration, Assignment},
    declaration::ObjectDeclaration,
};

pub mod declare;

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Assignment(AssignDeclaration),
    PortMapping(PortMapping),
}

impl From<AssignDeclaration> for Statement {
    fn from(assign: AssignDeclaration) -> Self {
        Statement::Assignment(assign)
    }
}

impl From<PortMapping> for Statement {
    fn from(portmapping: PortMapping) -> Self {
        Statement::PortMapping(portmapping)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PortMapping {
    label: String,
    component_name: String,
    /// The ports, in the order they were declared on the component
    ports: IndexMap<String, Id<ObjectDeclaration>>,
    /// Mappings for those ports, will be declared in the order of the original component declaration,
    /// irrespective of the order they're mapped during generation.
    mappings: HashMap<String, AssignDeclaration>,
}

impl PortMapping {
    pub fn from_component(
        db: &mut dyn Arch,
        component: &Component,
        label: impl Into<String>,
    ) -> Result<PortMapping> {
        let mut ports = IndexMap::new();
        for port in component.ports() {
            let obj = ObjectDeclaration::from_port(db, port, false);
            ports.insert(port.identifier().to_string(), obj);
        }
        Ok(PortMapping {
            label: label.into(),
            component_name: component.identifier().to_string(),
            ports,
            mappings: HashMap::new(),
        })
    }

    pub fn ports(&self) -> &IndexMap<String, Id<ObjectDeclaration>> {
        &self.ports
    }

    pub fn mappings(&self) -> &HashMap<String, AssignDeclaration> {
        &self.mappings
    }

    pub fn map_port(
        &mut self,
        db: &dyn Arch,
        identifier: impl Into<String>,
        assignment: &(impl Into<Assignment> + Clone),
    ) -> Result<&mut Self> {
        let identifier: &str = &identifier.into();
        let port = self
            .ports()
            .get(identifier)
            .ok_or(Error::InvalidArgument(format!(
                "Port {} does not exist on this component",
                identifier
            )))?;
        let mut assigned = port.assign(db, assignment)?;
        // If the port is already assigned, reverse the assignment so that the object being assigned is on the "left"
        if db.lookup_intern_object_declaration(*port).state() == ObjectState::Assigned {
            assigned = assigned.reverse(db)?;
        }
        self.mappings.insert(identifier.to_string(), assigned);
        Ok(self)
    }

    pub fn finish(self) -> Result<Self> {
        // Note that the assign function prevents assignment to a field of a port
        // TODO: These are both bad ideas, should allow for assignment of individual fields, and should track this in port mappings.
        //       Or should make port mapping separate from assignment.
        if self.ports().len() == self.mappings().len() {
            Ok(self)
        } else {
            Err(Error::BackEndError(format!(
                "The number of mappings ({}) does not match the number of ports ({})",
                self.mappings().len(),
                self.ports().len()
            )))
        }
    }

    pub fn label(&self) -> &str {
        self.label.as_str()
    }

    pub fn component_name(&self) -> &str {
        self.component_name.as_str()
    }

    /// Find the assignment to an object based on a port name and its ID, assuming one exists.
    pub(crate) fn assignment_for(
        &self,
        port: &str,
        id: Id<ObjectDeclaration>,
    ) -> Option<&AssignDeclaration> {
        if let Some(_) = self.ports().get(port).filter(|x| **x == id) {
            self.mappings().get(port)
        } else {
            None
        }
    }
}
