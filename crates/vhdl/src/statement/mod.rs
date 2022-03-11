use std::collections::HashMap;

use tydi_common::{
    error::{Error, Result, TryResult},
    insertion_ordered_map::InsertionOrderedMap,
};
use tydi_intern::Id;

use crate::{
    architecture::arch_storage::Arch,
    assignment::Assign,
    common::vhdl_name::{VhdlName, VhdlNameSelf},
    component::Component,
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
    label: VhdlName,
    component_name: VhdlName,
    /// The ports, in the order they were declared on the component
    ports: InsertionOrderedMap<VhdlName, Id<ObjectDeclaration>>,
    /// Mappings for those ports, will be declared in the order of the original component declaration,
    /// irrespective of the order they're mapped during generation.
    mappings: HashMap<VhdlName, AssignDeclaration>,
}

impl PortMapping {
    pub fn from_component(
        db: &mut dyn Arch,
        component: &Component,
        label: impl TryResult<VhdlName>,
    ) -> Result<PortMapping> {
        let mut ports = InsertionOrderedMap::new();
        for port in component.ports() {
            let obj = ObjectDeclaration::from_port(db, port, false);
            ports.try_insert(port.vhdl_name().clone(), obj)?;
        }
        Ok(PortMapping {
            label: label.try_result()?,
            component_name: component.vhdl_name().clone(),
            ports,
            mappings: HashMap::new(),
        })
    }

    pub fn ports(&self) -> &InsertionOrderedMap<VhdlName, Id<ObjectDeclaration>> {
        &self.ports
    }

    pub fn mappings(&self) -> &HashMap<VhdlName, AssignDeclaration> {
        &self.mappings
    }

    pub fn map_port(
        &mut self,
        db: &dyn Arch,
        identifier: impl TryResult<VhdlName>,
        assignment: &(impl Into<Assignment> + Clone),
    ) -> Result<()> {
        let identifier = identifier.try_result()?;
        let port = self
            .ports()
            .get(&identifier)
            .ok_or(Error::InvalidArgument(format!(
                "Port {} does not exist on this component",
                identifier
            )))?;
        let assigned = port.assign(db, assignment)?;
        self.mappings.insert(identifier, assigned);
        Ok(())
    }

    pub fn finish(self) -> Result<Self> {
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

    pub fn label(&self) -> &VhdlName {
        &self.label
    }

    pub fn component_name(&self) -> &VhdlName {
        &self.component_name
    }
}

impl VhdlNameSelf for PortMapping {
    fn vhdl_name(&self) -> &VhdlName {
        &self.label
    }
}
