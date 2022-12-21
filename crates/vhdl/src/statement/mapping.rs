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
    /// The parameters and their mappings, in the order they were declared on the component
    param_mappings: InsertionOrderedMap<VhdlName, MapAssignment>,
    /// The ports and their mappings, in the order they were declared on the component
    port_mappings: InsertionOrderedMap<VhdlName, MapAssignment>,
}

impl Mapping {
    pub fn from_component(
        db: &mut dyn Arch,
        component: &Component,
        label: impl TryResult<VhdlName>,
    ) -> Result<Mapping> {
        let param_mappings = component.parameters().clone().try_map_convert(|p| {
            Ok(MapAssignment::Unassigned(
                ObjectDeclaration::from_parameter(db, &p)?,
            ))
        })?;
        let port_mappings = component.ports().clone().map_convert(|p| {
            MapAssignment::Unassigned(ObjectDeclaration::from_port(db, &p, false))
        });
        Ok(Mapping {
            label: label.try_result()?,
            component_name: component.vhdl_name().clone(),
            param_mappings,
            port_mappings,
        })
    }

    pub fn param_mappings(&self) -> &InsertionOrderedMap<VhdlName, MapAssignment> {
        &self.param_mappings
    }

    pub fn has_param_assignments(&self) -> bool {
        for (_, param) in self.param_mappings() {
            if let MapAssignment::Assigned(_) = param {
                return true;
            }
        }
        false
    }

    pub fn port_mappings(&self) -> &InsertionOrderedMap<VhdlName, MapAssignment> {
        &self.port_mappings
    }

    pub fn map_param(
        &mut self,
        db: &dyn Arch,
        identifier: impl TryResult<VhdlName>,
        assignment_kind: impl Into<AssignmentKind>,
    ) -> Result<()> {
        let identifier = identifier.try_result()?;
        let param = self
            .param_mappings()
            .get(&identifier)
            .ok_or(Error::InvalidArgument(format!(
                "Parameter {} does not exist on this component",
                identifier
            )))?;
        match param {
            MapAssignment::Unassigned(obj) => {
                let assigned = MapAssignExpression::try_new(db, *obj, assignment_kind)?;
                self.param_mappings
                    .try_replace(&identifier, MapAssignment::Assigned(assigned))
            }
            MapAssignment::Assigned(_) => Err(Error::InvalidArgument(format!(
                "Parameter {} was already assigned",
                identifier
            ))),
        }
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

#[cfg(test)]
mod tests {
    use crate::{
        architecture::arch_storage::db::Database,
        assignment::StdLogicValue,
        declaration::Declare,
        object::object_type::IntegerType,
        statement::{relation::math::CreateMath, Statement},
        test_tools,
    };

    use super::*;

    #[test]
    fn test_empty_component_mapping_declare() -> Result<()> {
        let mut _db = Database::default();
        let db = &mut _db;
        let empty_comp = test_tools::empty_component();
        let mapping = Mapping::from_component(db, &empty_comp, "map_label")?;
        assert_eq!(
            r#"empty_component port map(

)"#,
            mapping.declare(db)?
        );

        let statement = Statement::from(mapping);
        assert_eq!(
            r#"map_label: empty_component port map(

)"#,
            statement.declare(db)?
        );

        Ok(())
    }

    #[test]
    fn test_simple_component_mapping_declare() -> Result<()> {
        let mut _db = Database::default();
        let db = &mut _db;
        let simple_comp = test_tools::simple_component()?;
        let mut mapping = Mapping::from_component(db, &simple_comp, "map_label2")?;

        let clk = ObjectDeclaration::entity_clk(db);
        let array_sig = ObjectDeclaration::signal(db, "array_sig", 43..0, None)?;

        mapping.map_port(db, "some_port", StdLogicValue::U)?;
        mapping.map_port(db, "some_other_port", array_sig)?;
        mapping.map_port(db, "clk", clk)?;

        assert_eq!(
            r#"test port map(
  some_port => 'U',
  some_other_port => array_sig,
  clk => clk
)"#,
            mapping.declare(db)?
        );

        let statement = Statement::from(mapping);
        assert_eq!(
            r#"map_label2: test port map(
  some_port => 'U',
  some_other_port => array_sig,
  clk => clk
)"#,
            statement.declare(db)?
        );

        Ok(())
    }

    #[test]
    fn test_simple_component_with_generics_mapping_declare() -> Result<()> {
        let mut _db = Database::default();
        let db = &mut _db;
        let param_comp = test_tools::simple_component_with_generics()?;
        let mut mapping = Mapping::from_component(db, &param_comp, "map_label3")?;

        let clk = ObjectDeclaration::entity_clk(db);
        let array_sig = ObjectDeclaration::signal(db, "array_sig", 43..0, None)?;

        mapping.map_param(db, "some_other_param", StdLogicValue::U)?;

        mapping.map_port(db, "some_port", StdLogicValue::U)?;
        mapping.map_port(db, "some_other_port", array_sig)?;
        mapping.map_port(db, "clk", clk)?;

        assert_eq!(
            r#"test generic map(
  some_other_param => 'U'
) port map(
  some_port => 'U',
  some_other_port => array_sig,
  clk => clk
)"#,
            mapping.declare(db)?
        );

        let outer_const = ObjectDeclaration::constant(db, "OUTER_CONST", IntegerType::Integer, 1)?;

        mapping.map_param(db, "some_param", 20)?;
        mapping.map_param(db, "some_other_param2", outer_const.r_add(db, 4)?)?;

        assert_eq!(
            r#"test generic map(
  some_param => 20,
  some_other_param => 'U',
  some_other_param2 => OUTER_CONST + 4
) port map(
  some_port => 'U',
  some_other_port => array_sig,
  clk => clk
)"#,
            mapping.declare(db)?
        );

        let statement = Statement::from(mapping);
        assert_eq!(
            r#"map_label3: test generic map(
  some_param => 20,
  some_other_param => 'U',
  some_other_param2 => OUTER_CONST + 4
) port map(
  some_port => 'U',
  some_other_port => array_sig,
  clk => clk
)"#,
            statement.declare(db)?
        );

        Ok(())
    }
}
