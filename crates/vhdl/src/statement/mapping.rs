use super::label::Label;
use crate::{
    architecture::arch_storage::{Arch, AssignmentState},
    assignment::AssignmentKind,
    common::vhdl_name::{VhdlName, VhdlNameSelf},
    component::Component,
    declaration::{DeclareWithIndent, ObjectDeclaration, ObjectKind},
    port::Mode,
    traits::VhdlDocument,
    usings::{ListUsingsDb, Usings},
};
use tydi_common::{
    error::{Error, Result, TryResult},
    map::InsertionOrderedMap,
    traits::{Document, Documents, Identify},
};
use tydi_intern::Id;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MapAssignExpression {
    target: Id<ObjectDeclaration>,
    assignment: AssignmentKind,
    doc: Option<String>,
}

impl MapAssignExpression {
    pub fn try_new(
        db: &dyn Arch,
        target: Id<ObjectDeclaration>,
        assignment: impl Into<AssignmentKind>,
    ) -> Result<MapAssignExpression> {
        let assignment = assignment.into();
        let kind = db.get_object_kind(target);
        fn try_assign(
            kind: impl AsRef<ObjectKind>,
            db: &dyn Arch,
            target: Id<ObjectDeclaration>,
            assignment: &AssignmentKind,
        ) -> Result<()> {
            match kind.as_ref() {
                ObjectKind::Signal | ObjectKind::Variable | ObjectKind::Constant => db.can_assign(
                    db.get_object_key(target),
                    assignment.clone().into(),
                    AssignmentState::Initialization,
                ),
                ObjectKind::ComponentPort(mode) => db.can_assign(
                    db.get_object_key(target),
                    assignment.clone().into(),
                    match mode {
                        Mode::In => AssignmentState::Default,
                        Mode::Out => AssignmentState::OutMapping,
                    },
                ),
                ObjectKind::EntityPort(_) => Err(Error::InvalidTarget(
                    "Cannot map to an entity port, it must use assign statements.".to_string(),
                )),
                ObjectKind::Alias(_, alias_kind) => try_assign(alias_kind, db, target, &assignment),
            }
        }
        try_assign(kind, db, target, &assignment)?;
        Ok(MapAssignExpression {
            target,
            assignment,
            doc: None,
        })
    }

    pub fn target(&self) -> Id<ObjectDeclaration> {
        self.target
    }

    pub fn assignment_kind(&self) -> &AssignmentKind {
        &self.assignment
    }

    /// The object declaration
    pub fn object_string(&self, db: &dyn Arch) -> String {
        db.lookup_intern_object_declaration(self.target())
            .identifier()
    }
}

impl Document for MapAssignExpression {
    fn doc(&self) -> Option<&String> {
        self.doc.as_ref()
    }
}

impl Documents for MapAssignExpression {
    fn set_doc(&mut self, doc: impl Into<String>) {
        self.doc = Some(doc.into());
    }
}

impl DeclareWithIndent for MapAssignExpression {
    fn declare_with_indent(&self, db: &dyn Arch, indent_style: &str) -> Result<String> {
        let mut result = String::new();
        if let Some(doc) = self.vhdl_doc() {
            result.push_str(&doc);
        }
        result.push_str(&format!("{} => ", &self.object_string(db)));
        result.push_str(&self.assignment_kind().declare_for(
            db,
            self.object_string(db),
            indent_style,
        )?);
        Ok(result)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MapAssignment {
    Unassigned(Id<ObjectDeclaration>),
    Assigned(MapAssignExpression),
}

impl MapAssignment {
    pub fn object(&self) -> Id<ObjectDeclaration> {
        match self {
            MapAssignment::Unassigned(obj) => *obj,
            MapAssignment::Assigned(expr) => expr.target(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Mapping {
    label: VhdlName,
    component_name: VhdlName,
    /// The ports and their mappings, in the order they were declared on the component
    port_mappings: InsertionOrderedMap<VhdlName, MapAssignment>,
}

impl Mapping {
    pub fn from_component(
        db: &mut dyn Arch,
        component: &Component,
        label: impl TryResult<VhdlName>,
    ) -> Result<Mapping> {
        let mut port_mappings = InsertionOrderedMap::new();
        for port in component.ports() {
            let obj = ObjectDeclaration::from_port(db, port, false);
            port_mappings.try_insert(port.vhdl_name().clone(), MapAssignment::Unassigned(obj))?;
        }
        Ok(Mapping {
            label: label.try_result()?,
            component_name: component.vhdl_name().clone(),
            port_mappings,
        })
    }

    pub fn port_mappings(&self) -> &InsertionOrderedMap<VhdlName, MapAssignment> {
        &self.port_mappings
    }

    pub fn map_port(
        &mut self,
        db: &dyn Arch,
        identifier: impl TryResult<VhdlName>,
        assignment_kind: impl Into<AssignmentKind>,
    ) -> Result<()> {
        let identifier = identifier.try_result()?;
        let port = self
            .port_mappings()
            .get(&identifier)
            .ok_or(Error::InvalidArgument(format!(
                "Port {} does not exist on this component",
                identifier
            )))?;
        match port {
            MapAssignment::Unassigned(obj) => {
                let assigned = MapAssignExpression::try_new(db, *obj, assignment_kind)?;
                self.port_mappings
                    .try_replace(&identifier, MapAssignment::Assigned(assigned))
            }
            MapAssignment::Assigned(_) => Err(Error::InvalidArgument(format!(
                "Port {} was already assigned",
                identifier
            ))),
        }
    }

    pub fn finish(self) -> Result<Self> {
        for (name, assignment) in self.port_mappings() {
            if let MapAssignment::Unassigned(_) = assignment {
                return Err(Error::BackEndError(format!(
                    "Port {} was not mapped.",
                    name
                )));
            }
        }
        Ok(self)
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
        for (_, assignment) in self.port_mappings() {
            usings.combine(&assignment.object().list_usings_db(db)?);
        }
        Ok(usings)
    }
}
