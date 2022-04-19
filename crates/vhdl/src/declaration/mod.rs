use std::fmt;

use tydi_common::error::{Error, Result, TryResult};
use tydi_common::traits::Identify;
use tydi_intern::Id;

use crate::architecture::arch_storage::interner::InternSelf;
use crate::architecture::arch_storage::object_queries::object_key::ObjectKey;
use crate::architecture::arch_storage::Arch;
use crate::common::vhdl_name::{VhdlName, VhdlNameSelf};
use crate::object::object_type::ObjectType;
use crate::object::Object;
use crate::port::{Mode, Port};

use super::assignment::{AssignmentKind, FieldSelection};

pub mod architecturedeclaration_from;
pub mod declare;
pub mod impls;

/// Generate trait for generic VHDL declarations.
pub trait Declare {
    /// Generate a VHDL declaration from self.
    fn declare(&self, db: &dyn Arch) -> Result<String>;
}

/// Allows users to specify the indent of scopes when declaring VHDL
///
/// E.g., when `pre` is set to two spaces
/// ```vhdl
/// entity component_with_nested_types is
///   port (
///     some_other_port : out record_type;
///     clk : in std_logic
///   );
/// end component_with_nested_types;
/// ```
pub trait DeclareWithIndent {
    fn declare_with_indent(&self, db: &dyn Arch, indent_style: &str) -> Result<String>;
}

impl<T: DeclareWithIndent> Declare for T {
    fn declare(&self, db: &dyn Arch) -> Result<String> {
        self.declare_with_indent(db, "  ")
    }
}

// Declarations may typically be any of the following: type, subtype, signal, constant, file, alias, component, attribute, function, procedure, configuration specification. (per: https://www.ics.uci.edu/~jmoorkan/vhdlref/architec.html)
// Per: https://insights.sigasi.com/tech/vhdl2008.ebnf/#block_declarative_item
//     subprogram_declaration
// | subprogram_body
// | subprogram_instantiation_declaration
// | package_declaration
// | package_body
// | package_instantiation_declaration
// | type_declaration
// | subtype_declaration
// | constant_declaration
// | signal_declaration
// | shared_variable_declaration
// | file_declaration
// | alias_declaration
// | component_declaration
// | attribute_declaration
// | attribute_specification
// | configuration_specification
// | disconnection_specification
// | use_clause
// | group_template_declaration
// | group_declaration
// | PSL_Property_Declaration
// | PSL_Sequence_Declaration
// | PSL_Clock_Declaration
/// Architecture declaration.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ArchitectureDeclaration {
    Type(String),      // TODO: Type declarations within the architecture
    SubType(String),   // TODO: Do we want subtypes, or should these just be (part of) types?
    Procedure(String), // TODO: Procedure
    Function(String),  // TODO: Function
    /// Object declaration, covering signals, variables, constants and ports*
    ///
    /// *Ports cannot be declared within the architecture itself, but can be used in the statement part,
    /// as such, the ports of the entity implemented are treated as inferred declarations.
    Object(Id<ObjectDeclaration>),
    Component(String), // TODO: Component declarations within the architecture
    Custom(String),    // TODO: Custom (templates?)
}

/// The kind of object declared (signal, variable, constant, ports)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ObjectKind {
    Signal,
    Variable,
    Constant,
    /// Represents ports declared on the entity this architecture is describing
    EntityPort(Mode),
    /// Represents ports on components within the architecture
    ComponentPort(Mode),
    Alias(String, Box<ObjectKind>),
}

impl fmt::Display for ObjectKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ObjectKind::Signal => write!(f, "Signal"),
            ObjectKind::Variable => write!(f, "Variable"),
            ObjectKind::Constant => write!(f, "Constant"),
            ObjectKind::EntityPort(mode) => write!(f, "EntityPort({})", mode),
            ObjectKind::ComponentPort(mode) => write!(f, "ComponentPort({})", mode),
            ObjectKind::Alias(obj, kind) => write!(f, "Alias({}, {})", obj, kind),
        }
    }
}

pub type ObjectModeId = usize;

/// The mode of an object, indicating whether it holds a value and whether it can be modified.
///
/// For instance, Ports cannot be modified: The "in" port of a component will remain Unassigned, and the "in" port of an entity cannot be assigned
/// a new value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectMode {
    can_be_modified: bool,
    state: ObjectState,
}

impl ObjectMode {
    pub fn new(can_be_modified: bool, state: ObjectState) -> Self {
        ObjectMode {
            can_be_modified,
            state,
        }
    }

    pub fn can_be_modified(&self) -> bool {
        self.can_be_modified
    }

    pub fn state(&self) -> ObjectState {
        self.state
    }

    pub fn set_state(&mut self, state: ObjectState) -> Result<()> {
        if self.can_be_modified() {
            self.state = state;
            Ok(())
        } else {
            Err(Error::InvalidTarget(
                "ObjectMode cannot be modified".to_string(),
            ))
        }
    }
}

/// The state of the object, relative to the architecture
///
/// (E.g., an "in" port on the entity is "Assigned", but so is an "out" port of a component inside the architecture)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObjectState {
    /// The object is not assigned a value (yet). (A signal which is not connected, an "out" port on an entity, or an "in" port on a component.)
    Unassigned,
    /// The object is carrying a value. (The "in" port of an entity and the "out" port of a component, or a signal which was assigned a value.)
    Assigned,
}

impl fmt::Display for ObjectState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObjectState::Unassigned => write!(f, "Unassigned"),
            ObjectState::Assigned => write!(f, "Assigned"),
        }
    }
}

/// Struct describing the identifier of the object, its type, its kind, and a potential default value
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectDeclaration {
    /// Name of the signal
    identifier: VhdlName,
    /// The Object this declaration refers to
    obj: ObjectKey,
    /// Default value assigned to the object (required for constants, cannot be used for ports)
    default: Option<AssignmentKind>,
    /// The kind of object
    kind: ObjectKind,
}

impl ObjectDeclaration {
    pub fn signal(
        db: &dyn Arch,
        identifier: impl TryResult<VhdlName>,
        typ: impl TryResult<ObjectType>,
        default: Option<AssignmentKind>,
    ) -> Result<Id<ObjectDeclaration>> {
        let kind = ObjectKind::Signal;
        Ok(ObjectDeclaration {
            identifier: identifier.try_result()?,
            obj: Object::try_new(db, typ, &kind)?,
            default,
            kind,
        }
        .intern(db))
    }

    pub fn variable(
        db: &dyn Arch,
        identifier: impl TryResult<VhdlName>,
        typ: impl TryResult<ObjectType>,
        default: Option<AssignmentKind>,
    ) -> Result<Id<ObjectDeclaration>> {
        let kind = ObjectKind::Variable;
        Ok(ObjectDeclaration {
            identifier: identifier.try_result()?,
            obj: Object::try_new(db, typ, &kind)?,
            default,
            kind,
        }
        .intern(db))
    }

    pub fn constant(
        db: &dyn Arch,
        identifier: impl TryResult<VhdlName>,
        typ: impl TryResult<ObjectType>,
        value: impl Into<AssignmentKind>,
    ) -> Result<Id<ObjectDeclaration>> {
        let kind = ObjectKind::Constant;
        Ok(ObjectDeclaration {
            identifier: identifier.try_result()?,
            obj: Object::try_new(db, typ, &kind)?,
            default: Some(value.into()),
            kind,
        }
        .intern(db))
    }

    /// Entity Ports serve as a way to represent the ports of an entity the architecture is describing.
    /// They are not declared within the architecture itself, but can drive or be driven by other objects.
    pub fn entity_port(
        db: &dyn Arch,
        identifier: impl TryResult<VhdlName>,
        typ: impl TryResult<ObjectType>,
        mode: Mode,
    ) -> Result<Id<ObjectDeclaration>> {
        let kind = ObjectKind::EntityPort(mode);
        Ok(ObjectDeclaration {
            identifier: identifier.try_result()?,
            obj: Object::try_new(db, typ, &kind)?,
            default: None,
            kind,
        }
        .intern(db))
    }

    pub fn component_port(
        db: &dyn Arch,
        identifier: impl TryResult<VhdlName>,
        typ: impl TryResult<ObjectType>,
        mode: Mode,
    ) -> Result<Id<ObjectDeclaration>> {
        let kind = ObjectKind::ComponentPort(mode);
        Ok(ObjectDeclaration {
            identifier: identifier.try_result()?,
            obj: Object::try_new(db, typ, &kind)?,
            default: None,
            kind,
        }
        .intern(db))
    }

    /// Aliases an existing object, with optional field constraint
    pub fn alias(
        db: &dyn Arch,
        identifier: impl TryResult<VhdlName>,
        object_declaration: Id<ObjectDeclaration>,
        selection: Vec<FieldSelection>,
    ) -> Result<Id<ObjectDeclaration>> {
        let object_declaration = db.lookup_intern_object_declaration(object_declaration);
        Ok(ObjectDeclaration {
            identifier: identifier.try_result()?,
            obj: object_declaration
                .object_key()
                .clone()
                .with_nested(selection),
            default: None,
            kind: ObjectKind::Alias(
                object_declaration.identifier(),
                Box::new(object_declaration.kind().clone()),
            ),
        }
        .intern(db))
    }

    /// Create a default "clk" entity port object
    pub fn entity_clk(db: &mut dyn Arch) -> Id<ObjectDeclaration> {
        ObjectDeclaration::entity_port(db, "clk", ObjectType::Bit, Mode::In).unwrap()
    }

    /// Create a default "rst" entity port object
    pub fn entity_rst(db: &mut dyn Arch) -> Id<ObjectDeclaration> {
        ObjectDeclaration::entity_port(db, "rst", ObjectType::Bit, Mode::In).unwrap()
    }

    pub fn kind(&self) -> &ObjectKind {
        &self.kind
    }

    pub fn default(&self) -> &Option<AssignmentKind> {
        &self.default
    }

    pub fn object_key(&self) -> &ObjectKey {
        &self.obj
    }

    pub fn object(&self, db: &dyn Arch) -> Result<Object> {
        db.get_object(self.object_key().clone())
    }

    pub fn typ(&self, db: &dyn Arch) -> Result<ObjectType> {
        Ok(self.object(db)?.typ(db))
    }

    pub fn from_port(db: &dyn Arch, port: &Port, is_entity: bool) -> Id<ObjectDeclaration> {
        if is_entity {
            ObjectDeclaration::entity_port(
                db,
                port.vhdl_name().clone(),
                port.typ().clone(),
                port.mode(),
            )
            .unwrap()
        } else {
            ObjectDeclaration::component_port(
                db,
                port.vhdl_name().clone(),
                port.typ().clone(),
                port.mode(),
            )
            .unwrap()
        }
    }
}

impl Identify for ObjectDeclaration {
    fn identifier(&self) -> String {
        self.identifier.to_string()
    }
}

impl VhdlNameSelf for ObjectDeclaration {
    fn vhdl_name(&self) -> &VhdlName {
        &self.identifier
    }
}
