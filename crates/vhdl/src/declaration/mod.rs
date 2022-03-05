use std::fmt;

use tydi_common::error::{Error, Result, TryResult};
use tydi_common::traits::Identify;
use tydi_intern::Id;

use crate::architecture::arch_storage::Arch;
use crate::architecture::arch_storage::interner::InternSelf;
use crate::common::vhdl_name::{VhdlName, VhdlNameSelf};
use crate::object::object_type::ObjectType;
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
    /// Alias for an object declaration, with optional range constraint
    Alias(AliasDeclaration),
    Component(String), // TODO: Component declarations within the architecture
    Custom(String),    // TODO: Custom (templates?)
}

/// The kind of object declared (signal, variable, constant, ports)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObjectKind {
    Signal,
    Variable,
    Constant,
    /// Represents ports declared on the entity this architecture is describing
    EntityPort,
    /// Represents ports on components within the architecture
    ComponentPort,
}

impl fmt::Display for ObjectKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ObjectKind::Signal => write!(f, "Signal"),
            ObjectKind::Variable => write!(f, "Variable"),
            ObjectKind::Constant => write!(f, "Constant"),
            ObjectKind::EntityPort => write!(f, "EntityPort"),
            ObjectKind::ComponentPort => write!(f, "ComponentPort"),
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
    /// (Sub-)Type of the object
    typ: ObjectType,
    mode: ObjectMode,
    /// Default value assigned to the object (required for constants, cannot be used for ports)
    default: Option<AssignmentKind>,
    /// The kind of object
    kind: ObjectKind,
}

impl ObjectDeclaration {
    pub fn signal(
        db: &mut dyn Arch,
        identifier: impl TryResult<VhdlName>,
        typ: ObjectType,
        default: Option<AssignmentKind>,
    ) -> Result<Id<ObjectDeclaration>> {
        Ok(ObjectDeclaration {
            identifier: identifier.try_result()?,
            typ,
            mode: ObjectMode::new(true, ObjectState::Unassigned),
            default,
            kind: ObjectKind::Signal,
        }
        .intern(db))
    }

    pub fn variable(
        db: &mut dyn Arch,
        identifier: impl TryResult<VhdlName>,
        typ: ObjectType,
        default: Option<AssignmentKind>,
    ) -> Result<Id<ObjectDeclaration>> {
        Ok(ObjectDeclaration {
            identifier: identifier.try_result()?,
            typ,
            mode: if let Some(_) = default {
                ObjectMode::new(true, ObjectState::Assigned)
            } else {
                ObjectMode::new(true, ObjectState::Unassigned)
            },
            default,
            kind: ObjectKind::Variable,
        }
        .intern(db))
    }

    pub fn constant(
        db: &mut dyn Arch,
        identifier: impl TryResult<VhdlName>,
        typ: ObjectType,
        value: impl Into<AssignmentKind>,
    ) -> Result<Id<ObjectDeclaration>> {
        Ok(ObjectDeclaration {
            identifier: identifier.try_result()?,
            typ,
            mode: ObjectMode::new(false, ObjectState::Unassigned),
            default: Some(value.into()),
            kind: ObjectKind::Constant,
        }
        .intern(db))
    }

    /// Entity Ports serve as a way to represent the ports of an entity the architecture is describing.
    /// They are not declared within the architecture itself, but can drive or be driven by other objects.
    pub fn entity_port(
        db: &mut dyn Arch,
        identifier: impl TryResult<VhdlName>,
        typ: ObjectType,
        mode: Mode,
    ) -> Result<Id<ObjectDeclaration>> {
        Ok(ObjectDeclaration {
            identifier: identifier.try_result()?,
            typ,
            mode: match mode {
                Mode::In => ObjectMode::new(false, ObjectState::Assigned),
                Mode::Out => ObjectMode::new(false, ObjectState::Unassigned),
            },
            default: None,
            kind: ObjectKind::EntityPort,
        }
        .intern(db))
    }

    pub fn component_port(
        db: &mut dyn Arch,
        identifier: impl TryResult<VhdlName>,
        typ: ObjectType,
        mode: Mode,
    ) -> Result<Id<ObjectDeclaration>> {
        Ok(ObjectDeclaration {
            identifier: identifier.try_result()?,
            typ,
            mode: match mode {
                Mode::In => ObjectMode::new(false, ObjectState::Unassigned), // An "in" port requires an object going out of the architecture
                Mode::Out => ObjectMode::new(false, ObjectState::Assigned), // An "out" port is already assigned a value
            },
            default: None,
            kind: ObjectKind::ComponentPort,
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

    pub fn kind(&self) -> ObjectKind {
        self.kind
    }

    pub fn typ(&self) -> &ObjectType {
        &self.typ
    }

    pub fn default(&self) -> &Option<AssignmentKind> {
        &self.default
    }

    pub fn mode(&self) -> ObjectMode {
        self.mode
    }

    pub fn can_be_modified(&self) -> bool {
        self.mode().can_be_modified()
    }

    pub fn state(&self) -> ObjectState {
        self.mode().state()
    }

    pub fn set_state(&mut self, state: ObjectState) -> Result<()> {
        match self.mode.set_state(state) {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::InvalidTarget(format!(
                "Cannot set state of object {}",
                self.identifier()
            ))),
        }
    }

    pub fn from_port(db: &mut dyn Arch, port: &Port, is_entity: bool) -> Id<ObjectDeclaration> {
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

/// Aliases an existing object, with optional field constraint
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AliasDeclaration {
    identifier: VhdlName,
    /// Reference to an existing object declaration
    object: Id<ObjectDeclaration>,
    /// Optional field selection(s) - when assigning to or from the alias, this is used to determine the fields it represents
    field_selection: Vec<FieldSelection>,
}

impl AliasDeclaration {
    pub fn new(
        db: &dyn Arch,
        object: Id<ObjectDeclaration>,
        identifier: impl TryResult<VhdlName>,
        fields: Vec<FieldSelection>,
    ) -> Result<AliasDeclaration> {
        AliasDeclaration::from_object(object, identifier)?.with_selection(db, fields)
    }

    pub fn from_object(
        object: Id<ObjectDeclaration>,
        identifier: impl TryResult<VhdlName>,
    ) -> Result<AliasDeclaration> {
        Ok(AliasDeclaration {
            identifier: identifier.try_result()?,
            object,
            field_selection: vec![],
        })
    }

    /// Apply one or more field selections to the alias
    pub fn with_selection(mut self, db: &dyn Arch, fields: Vec<FieldSelection>) -> Result<Self> {
        let mut object = db
            .lookup_intern_object_declaration(self.object())
            .typ()
            .clone();
        for field in self.field_selection() {
            object = object.get_field(field)?;
        }
        for field in fields {
            object = object.get_field(&field)?;
            self.field_selection.push(field)
        }

        Ok(self)
    }

    /// Returns the actual object this is aliasing
    pub fn object(&self) -> Id<ObjectDeclaration> {
        self.object
    }

    /// Returns the optional field selection of this alias
    pub fn field_selection(&self) -> &Vec<FieldSelection> {
        &self.field_selection
    }

    /// Returns the object type of the alias (after fields have been selected)
    pub fn typ(&self, db: &dyn Arch) -> Result<ObjectType> {
        let mut object = db
            .lookup_intern_object_declaration(self.object())
            .typ()
            .clone();
        for field in self.field_selection() {
            object = object.get_field(field)?;
        }
        Ok(object)
    }
}

impl Identify for AliasDeclaration {
    fn identifier(&self) -> String {
        self.identifier.to_string()
    }
}

impl VhdlNameSelf for AliasDeclaration {
    fn vhdl_name(&self) -> &VhdlName {
        &self.identifier
    }
}

#[cfg(test)]
pub mod tests {
    use indexmap::IndexMap;

    use crate::{architecture::arch_storage::db::Database, object::record::RecordObject};

    use super::*;

    pub(crate) fn test_bit_signal(db: &mut dyn Arch) -> Result<Id<ObjectDeclaration>> {
        ObjectDeclaration::signal(db, "test_signal", ObjectType::Bit, None)
    }

    pub(crate) fn test_complex_signal(db: &mut dyn Arch) -> Result<Id<ObjectDeclaration>> {
        let mut fields = IndexMap::new();
        fields.insert(VhdlName::try_new("a")?, ObjectType::bit_vector(10, -4)?);
        ObjectDeclaration::signal(
            db,
            "test_signal",
            ObjectType::Record(RecordObject::new(VhdlName::try_new("record_typ")?, fields)),
            None,
        )
    }

    #[test]
    fn alias_verification_success() -> Result<()> {
        let mut db = Database::default();
        let test_bit_signal = test_bit_signal(&mut db)?;
        let test_complex_signal = test_complex_signal(&mut db)?;
        AliasDeclaration::from_object(test_bit_signal, "test_signal_alias")?;
        AliasDeclaration::from_object(test_complex_signal, "test_signal_alias")?
            .with_selection(&db, vec![FieldSelection::try_name("a")?])?;
        AliasDeclaration::from_object(test_complex_signal, "test_signal_alias")?.with_selection(
            &db,
            vec![
                FieldSelection::try_name("a")?,
                FieldSelection::downto(10, -4)?,
            ],
        )?;
        AliasDeclaration::from_object(test_complex_signal, "test_signal_alias")?
            .with_selection(&db, vec![FieldSelection::try_name("a")?])?
            .with_selection(&db, vec![FieldSelection::downto(10, -4)?])?;
        AliasDeclaration::from_object(test_complex_signal, "test_signal_alias")?.with_selection(
            &db,
            vec![
                FieldSelection::try_name("a")?,
                FieldSelection::downto(4, -1)?,
            ],
        )?;
        AliasDeclaration::from_object(test_complex_signal, "test_signal_alias")?.with_selection(
            &db,
            vec![FieldSelection::try_name("a")?, FieldSelection::to(-4, 10)?],
        )?;
        AliasDeclaration::from_object(test_complex_signal, "test_signal_alias")?.with_selection(
            &db,
            vec![FieldSelection::try_name("a")?, FieldSelection::index(10)],
        )?;
        AliasDeclaration::from_object(test_complex_signal, "test_signal_alias")?.with_selection(
            &db,
            vec![FieldSelection::try_name("a")?, FieldSelection::index(-4)],
        )?;
        Ok(())
    }

    #[test]
    fn alias_verification_error() -> Result<()> {
        let mut db = Database::default();
        let test_bit_signal = test_bit_signal(&mut db)?;
        let test_complex_signal = test_complex_signal(&mut db)?;
        is_invalid_target(
            AliasDeclaration::from_object(test_bit_signal, "test_signal_alias")?
                .with_selection(&db, vec![FieldSelection::try_name("a")?]),
        )?;
        is_invalid_target(
            AliasDeclaration::from_object(test_bit_signal, "test_signal_alias")?
                .with_selection(&db, vec![FieldSelection::index(1)]),
        )?;
        is_invalid_target(
            AliasDeclaration::from_object(test_complex_signal, "test_signal_alias")?
                .with_selection(&db, vec![FieldSelection::index(1)]),
        )?;
        is_invalid_argument(
            AliasDeclaration::from_object(test_complex_signal, "test_signal_alias")?
                .with_selection(&db, vec![FieldSelection::try_name("b")?]),
        )?;
        is_invalid_target(
            AliasDeclaration::from_object(test_complex_signal, "test_signal_alias")?
                .with_selection(
                    &db,
                    vec![
                        FieldSelection::try_name("a")?,
                        FieldSelection::try_name("a")?,
                    ],
                ),
        )?;
        is_invalid_argument(
            AliasDeclaration::from_object(test_complex_signal, "test_signal_alias")?
                .with_selection(
                    &db,
                    vec![
                        FieldSelection::try_name("a")?,
                        FieldSelection::downto(11, -4)?,
                    ],
                ),
        )?;
        Ok(())
    }

    fn is_invalid_target<T>(result: Result<T>) -> Result<()> {
        match result {
            Err(Error::InvalidTarget(_)) => Ok(()),
            _ => Err(Error::UnknownError),
        }
    }

    fn is_invalid_argument<T>(result: Result<T>) -> Result<()> {
        match result {
            Err(Error::InvalidArgument(_)) => Ok(()),
            _ => Err(Error::UnknownError),
        }
    }
}
