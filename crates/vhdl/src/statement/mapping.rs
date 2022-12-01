use super::label::Label;
use crate::{
    architecture::arch_storage::Arch,
    assignment::{Assign, AssignDeclaration, Assignment},
    common::vhdl_name::{VhdlName, VhdlNameSelf},
    component::Component,
    declaration::ObjectDeclaration,
    usings::{ListUsingsDb, Usings},
};
use tydi_common::{
    error::{Error, Result, TryResult},
    map::InsertionOrderedMap,
    traits::{Document, Documents, Identify},
};
use tydi_intern::Id;

pub struct MapAssignStatement {
    object: Id<ObjectDeclaration>,
    assignment: Assignment,
    doc: Option<String>,
}

impl MapAssignStatement {
    pub fn new(object: Id<ObjectDeclaration>, assignment: Assignment) -> MapAssignStatement {
        MapAssignStatement {
            object,
            assignment,
            doc: None,
        }
    }

    pub fn object(&self) -> Id<ObjectDeclaration> {
        self.object
    }

    pub fn assignment(&self) -> &Assignment {
        &self.assignment
    }

    /// The object declaration with any field selections on it
    pub fn object_string(&self, db: &dyn Arch) -> String {
        let mut result = db
            .lookup_intern_object_declaration(self.object())
            .identifier()
            .to_string();
        for field in self.assignment().to_field() {
            result.push_str(&field.to_string());
        }
        result
    }
}

impl Document for MapAssignStatement {
    fn doc(&self) -> Option<&String> {
        self.doc.as_ref()
    }
}

impl Documents for MapAssignStatement {
    fn set_doc(&mut self, doc: impl Into<String>) {
        self.doc = Some(doc.into());
    }
}

pub enum MapAssignment {
    Unassigned(Id<ObjectDeclaration>),
    Assigned(MapAssignStatement),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Mapping {
    label: VhdlName,
    component_name: VhdlName,
    /// The ports, in the order they were declared on the component
    ports: InsertionOrderedMap<VhdlName, Id<ObjectDeclaration>>,
    /// Mappings for those ports, will be declared in the order of the original component declaration,
    /// irrespective of the order they're mapped during generation.
    port_mappings: InsertionOrderedMap<VhdlName, AssignDeclaration>,
}

impl Mapping {
    pub fn from_component(
        db: &mut dyn Arch,
        component: &Component,
        label: impl TryResult<VhdlName>,
    ) -> Result<Mapping> {
        let mut ports = InsertionOrderedMap::new();
        for port in component.ports() {
            let obj = ObjectDeclaration::from_port(db, port, false);
            ports.try_insert(port.vhdl_name().clone(), obj)?;
        }
        Ok(Mapping {
            label: label.try_result()?,
            component_name: component.vhdl_name().clone(),
            ports,
            port_mappings: InsertionOrderedMap::new(),
        })
    }

    pub fn ports(&self) -> &InsertionOrderedMap<VhdlName, Id<ObjectDeclaration>> {
        &self.ports
    }

    pub fn port_mappings(&self) -> &InsertionOrderedMap<VhdlName, AssignDeclaration> {
        &self.port_mappings
    }

    pub fn map_port(
        &mut self,
        db: &dyn Arch,
        identifier: impl TryResult<VhdlName>,
        assignment: impl Into<Assignment>,
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
        self.port_mappings.try_insert(identifier, assigned)?;
        Ok(())
    }

    pub fn finish(self) -> Result<Self> {
        if self.ports().len() == self.port_mappings().len() {
            Ok(self)
        } else {
            Err(Error::BackEndError(format!(
                "The number of mappings ({}) does not match the number of ports ({}).\nExpected: {}\nActual: {}",
                self.port_mappings().len(),
                self.ports().len(),
                self.ports().keys().map(|k| k.to_string()).collect::<Vec<String>>().join(", "),
                self.port_mappings().keys().map(|k| k.to_string()).collect::<Vec<String>>().join(", "),
            )))
        }
    }

    pub fn component_name(&self) -> &VhdlName {
        &self.component_name
    }
}

impl Label for Mapping {
    fn label(&self) -> Option<&VhdlName> {
        Some(&self.label)
    }

    fn set_label(&mut self, label: impl Into<VhdlName>) {
        self.label = label.into()
    }
}

impl VhdlNameSelf for Mapping {
    fn vhdl_name(&self) -> &VhdlName {
        &self.label
    }
}

impl ListUsingsDb for Mapping {
    fn list_usings_db(&self, db: &dyn Arch) -> Result<crate::usings::Usings> {
        let mut usings = Usings::new_empty();
        for (_, object) in self.ports() {
            usings.combine(&object.list_usings_db(db)?);
        }
        Ok(usings)
    }
}
