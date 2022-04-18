use tydi_common::{
    error::{Error, Result, TryResult},
    map::InsertionOrderedMap,
};
use tydi_intern::Id;

use crate::{
    architecture::arch_storage::Arch,
    assignment::Assign,
    common::vhdl_name::{VhdlName, VhdlNameSelf},
    component::Component,
};

use self::label::Label;

use super::{
    assignment::{AssignDeclaration, Assignment},
    declaration::ObjectDeclaration,
};

pub mod declare;
pub mod label;
pub mod logical_expression;

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Assignment(AssignDeclaration),
    PortMapping(PortMapping),
}

impl Label for Statement {
    fn label(&self) -> Option<&VhdlName> {
        match self {
            Statement::Assignment(a) => a.label(),
            Statement::PortMapping(p) => p.label(),
        }
    }

    fn set_label(&mut self, label: impl Into<VhdlName>) {
        match self {
            Statement::Assignment(a) => a.set_label(label),
            Statement::PortMapping(p) => p.set_label(label),
        }
    }
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PortMapping {
    label: VhdlName,
    component_name: VhdlName,
    /// The ports, in the order they were declared on the component
    ports: InsertionOrderedMap<VhdlName, Id<ObjectDeclaration>>,
    /// Mappings for those ports, will be declared in the order of the original component declaration,
    /// irrespective of the order they're mapped during generation.
    mappings: InsertionOrderedMap<VhdlName, AssignDeclaration>,
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
            mappings: InsertionOrderedMap::new(),
        })
    }

    pub fn ports(&self) -> &InsertionOrderedMap<VhdlName, Id<ObjectDeclaration>> {
        &self.ports
    }

    pub fn mappings(&self) -> &InsertionOrderedMap<VhdlName, AssignDeclaration> {
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
        self.mappings.try_insert(identifier, assigned)?;
        Ok(())
    }

    pub fn finish(self) -> Result<Self> {
        if self.ports().len() == self.mappings().len() {
            Ok(self)
        } else {
            Err(Error::BackEndError(format!(
                "The number of mappings ({}) does not match the number of ports ({}).\nExpected: {}\nActual: {}",
                self.mappings().len(),
                self.ports().len(),
                self.ports().keys().map(|k| k.to_string()).collect::<Vec<String>>().join(", "),
                self.mappings().keys().map(|k| k.to_string()).collect::<Vec<String>>().join(", "),
            )))
        }
    }

    pub fn component_name(&self) -> &VhdlName {
        &self.component_name
    }
}

impl Label for PortMapping {
    fn label(&self) -> Option<&VhdlName> {
        Some(&self.label)
    }

    fn set_label(&mut self, label: impl Into<VhdlName>) {
        self.label = label.into()
    }
}

impl VhdlNameSelf for PortMapping {
    fn vhdl_name(&self) -> &VhdlName {
        &self.label
    }
}
