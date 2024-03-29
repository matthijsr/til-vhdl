use core::fmt;
use std::convert::TryInto;

use indexmap::map::IndexMap;

use array_assignment::ArrayAssignment;
use textwrap::indent;
use tydi_common::error::{Error, Result, TryResult};
use tydi_common::traits::{Document, Documents, Identify};
use tydi_intern::Id;

use crate::architecture::arch_storage::object_queries::object_key::ObjectKey;

use crate::architecture::arch_storage::Arch;
use crate::common::vhdl_name::VhdlName;
use crate::declaration::DeclareWithIndent;
use crate::object::object_type::time::TimeValue;
use crate::object::object_type::ObjectType;
use crate::properties::Width;
use crate::statement::label::Label;
use crate::statement::relation::Relation;

use super::declaration::ObjectDeclaration;

use self::bitvec::BitVecValue;

pub mod array_assignment;
pub mod assign;
pub mod assignment_from;
pub mod bitvec;
pub mod declare;
// pub mod flatten;
pub mod impls;

pub trait Assign {
    fn assign(&self, db: &dyn Arch, assignment: impl Into<Assignment>)
        -> Result<AssignDeclaration>;
}

/// Describing the declaration of an assignment
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AssignDeclaration {
    label: Option<VhdlName>,
    /// The declared object being assigned
    object: Id<ObjectDeclaration>,
    /// The assignment to the declared object
    assignment: Assignment,
    doc: Option<String>,
}

impl AssignDeclaration {
    pub fn new(object: Id<ObjectDeclaration>, assignment: Assignment) -> AssignDeclaration {
        AssignDeclaration {
            label: None,
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
    pub fn object_string(&self, db: &dyn Arch, indent_style: &str) -> Result<String> {
        let mut result = db
            .lookup_intern_object_declaration(self.object())
            .identifier();
        for field in self.assignment().to_field() {
            result.push_str(&field.declare_with_indent(db, indent_style)?);
        }
        Ok(result)
    }

    /// If this is an object to object assignment, return the object being assigned from
    pub fn from(&self) -> Option<Id<ObjectDeclaration>> {
        if let AssignmentKind::Relation(Relation::Object(o)) = self.assignment().kind() {
            Some(o.object())
        } else {
            None
        }
    }

    /// Based on assignment, return the expected resulting object mode.
    ///
    /// This method assumes the object being assigned to is in a suitable mode,
    /// further validation occurs as part of the database query.
    ///
    /// Will return an error if the assignment itself is somehow incorrect.
    // pub fn resulting_mode(&self, db: &dyn Arch) -> Result<ObjectMode> {

    //     match self.assignment().kind() {
    //         AssignmentKind::Object(_) => todo!(),
    //         AssignmentKind::Direct(d) => match d {
    //             DirectAssignment::Value(_) => Ok(ObjectMode::new()),
    //             DirectAssignment::FullRecord(fas) => {
    //                 let mut mode = None;
    //                 let mut prev_field = "";
    //                 for fa in fas {
    //                     let mode2 = fa.assignment().resulting_mode(db)?;
    //                     match mode {
    //                         Some(m) => {
    //                             if m == mode2 {
    //                                 return Err(Error::BackEndError(format!("Attempted to find mode of FullRecord assignment, but assignment has conflicting modes: {}: {} and {}: {}", prev_field, m, fa.identifier(), mode2)));
    //                             }
    //                         }
    //                         None => mode = Some(mode2),
    //                     }
    //                 }
    //                 if let Some(m) = mode {
    //                     Ok(m)
    //                 } else {
    //                     Err(Error::BackEndError("Attempted to find mode of FullRecord assignment, but assignment was empty.".to_string()))
    //                 }
    //             }
    //             DirectAssignment::FullArray(_) => todo!(),
    //         },
    //     }
    // }

    /// Attempts to reverse the assignment. This is (currently) only possible for object assignments
    pub fn reverse(&self, db: &dyn Arch) -> Result<AssignDeclaration> {
        match self.assignment().kind() {
            AssignmentKind::Relation(Relation::Object(object)) => object.object().assign(
                db,
                Assignment::from(
                    ObjectSelection::from(self.object()).assign_from(self.assignment().to_field()),
                )
                .to_nested(object.from_field()),
            ),
            _ => Err(Error::InvalidTarget(
                "Cannot reverse an assignment that's not between objects.".to_string(),
            )),
        }
    }
}

impl Label for AssignDeclaration {
    fn label(&self) -> Option<&VhdlName> {
        self.label.as_ref()
    }

    fn set_label(&mut self, label: impl Into<VhdlName>) {
        self.label = Some(label.into())
    }
}

impl Document for AssignDeclaration {
    fn doc(&self) -> Option<&String> {
        self.doc.as_ref()
    }
}

impl Documents for AssignDeclaration {
    fn set_doc(&mut self, doc: impl Into<String>) {
        self.doc = Some(doc.into());
    }
}

/// An object can be assigned from another object or directly
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Assignment {
    /// Indicates assignment to (nested) fields. (Named or range)
    to_field: Vec<FieldSelection>,
    /// Indicates the kind of assignment (object to object, or directly to object)
    kind: AssignmentKind,
}

impl Assignment {
    pub fn to(mut self, to: FieldSelection) -> Self {
        self.to_field.push(to);
        self
    }

    pub fn to_nested(mut self, nested: &Vec<FieldSelection>) -> Self {
        self.to_field.extend(nested.clone());
        self
    }

    /// Append a named field selection
    pub fn to_named(self, to: impl Into<VhdlName>) -> Self {
        self.to(FieldSelection::Name(to.into()))
    }

    /// Append a range field selection
    pub fn to_range(self, to: RangeConstraint) -> Self {
        self.to(FieldSelection::Range(to))
    }

    /// Append a downto range field selection
    pub fn to_downto(self, start: i32, end: i32) -> Result<Self> {
        Ok(self.to_range(RangeConstraint::downto(start, end)?))
    }

    /// Append a to range field selection
    pub fn to_to(self, start: i32, end: i32) -> Result<Self> {
        Ok(self.to_range(RangeConstraint::to(start, end)?))
    }

    /// Append a to range field selection
    pub fn to_index(self, index: i32) -> Self {
        self.to_range(RangeConstraint::Index(index.into()))
    }

    /// Returns the fields selected
    pub fn to_field(&self) -> &Vec<FieldSelection> {
        &self.to_field
    }

    /// Returns the assignment kind
    pub fn kind(&self) -> &AssignmentKind {
        &self.kind
    }

    pub fn declare_for(
        &self,
        db: &dyn Arch,
        object_identifier: impl TryResult<VhdlName>,
        indent_style: &str,
    ) -> Result<String> {
        if let AssignmentKind::Relation(Relation::Value(va)) = &self.kind() {
            if let ValueAssignment::BitVec(bitvec) = va.as_ref() {
                if let Some(FieldSelection::Range(range)) = self.to_field().last() {
                    return bitvec.declare_for_range(range);
                }
            }
        }
        self.kind().declare_for(db, object_identifier, indent_style)
    }
}

/// An object can be assigned a value or from another object
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AssignmentKind {
    /// An object is assigned from or driven by a relation
    Relation(Relation),
    /// An object is assigned a value, or all fields are assigned/driven at once
    Direct(DirectAssignment),
}

impl AssignmentKind {
    pub fn full_record(fields: IndexMap<VhdlName, AssignmentKind>) -> AssignmentKind {
        AssignmentKind::Direct(DirectAssignment::FullRecord(
            fields
                .into_iter()
                .map(|(k, v)| FieldAssignment::new(k, v))
                .collect(),
        ))
    }

    // /// Converts an object assignment into a direct assignment. Useful when array or record types have identical fields but different type names.
    // ///
    // /// `convert_all` will also unwrap further nested objects
    // pub fn to_direct(
    //     object: &(impl Into<ObjectAssignment> + Clone),
    //     convert_all: bool,
    // ) -> Result<AssignmentKind> {
    //     let object = object.clone().into();
    //     match object.typ()? {
    //         ObjectType::Bit => Ok(object.into()),
    //         ObjectType::Record(rec) => {
    //             let mut fields = IndexMap::new();
    //             for (field, typ) in rec.fields() {
    //                 match typ {
    //                     ObjectType::Array(_) if convert_all => {
    //                         fields.insert(
    //                             field.clone(),
    //                             AssignmentKind::to_direct(
    //                                 &object
    //                                     .clone()
    //                                     .assign_from(&vec![FieldSelection::name(field)])?,
    //                                 true,
    //                             )?,
    //                         );
    //                     }
    //                     ObjectType::Record(_) if convert_all => {
    //                         fields.insert(
    //                             field.clone(),
    //                             AssignmentKind::to_direct(
    //                                 &object
    //                                     .clone()
    //                                     .assign_from(&vec![FieldSelection::name(field)])?,
    //                                 true,
    //                             )?,
    //                         );
    //                     }
    //                     _ => {
    //                         fields.insert(
    //                             field.clone(),
    //                             object
    //                                 .clone()
    //                                 .assign_from(&vec![FieldSelection::name(field)])?
    //                                 .into(),
    //                         );
    //                     }
    //                 }
    //             }
    //             Ok(AssignmentKind::Direct(DirectAssignment::FullRecord(fields)))
    //         }
    //         ObjectType::Array(arr) => {
    //             if arr.is_bitvector() {
    //                 Ok(object.into())
    //             } else {
    //                 let mut fields = vec![];
    //                 match arr.typ() {
    //                     ObjectType::Array(_) if convert_all => {
    //                         for i in arr.low()..arr.high() + 1 {
    //                             fields.push(AssignmentKind::to_direct(
    //                                 &object
    //                                     .clone()
    //                                     .assign_from(&vec![FieldSelection::index(i)])?,
    //                                 true,
    //                             )?);
    //                         }
    //                     }
    //                     ObjectType::Record(_) if convert_all => {
    //                         for i in arr.low()..arr.high() + 1 {
    //                             fields.push(AssignmentKind::to_direct(
    //                                 &object
    //                                     .clone()
    //                                     .assign_from(&vec![FieldSelection::index(i)])?,
    //                                 true,
    //                             )?);
    //                         }
    //                     }
    //                     _ => {
    //                         for i in arr.low()..arr.high() + 1 {
    //                             fields.push(
    //                                 object
    //                                     .clone()
    //                                     .assign_from(&vec![FieldSelection::index(i)])?
    //                                     .into(),
    //                             );
    //                         }
    //                     }
    //                 }
    //                 Ok(AssignmentKind::Direct(DirectAssignment::FullArray(
    //                     ArrayAssignment::Direct(fields),
    //                 )))
    //             }
    //         }
    //     }
    // }

    pub fn declare_for(
        &self,
        db: &dyn Arch,
        object_identifier: impl TryResult<VhdlName>,
        indent_style: &str,
    ) -> Result<String> {
        let object_identifier = object_identifier.try_result()?;
        match self {
            AssignmentKind::Relation(relation) => relation.declare_with_indent(db, indent_style),
            AssignmentKind::Direct(direct) => match direct {
                DirectAssignment::FullRecord(record) => {
                    let mut field_assignments = Vec::new();
                    for rf in record {
                        field_assignments.push(format!(
                            "\n{} => {}",
                            rf.field(),
                            rf.assignment().declare_for(
                                db,
                                format!("{}.{}", object_identifier, rf.field()),
                                indent_style,
                            )?
                        ));
                    }
                    Ok(format!(
                        "({}\n)",
                        indent(&field_assignments.join(","), indent_style)
                    ))
                }
                DirectAssignment::FullArray(array) => match array {
                    ArrayAssignment::Direct(direct) => {
                        let mut positionals = Vec::new();
                        for value in direct {
                            positionals.push(value.declare_for(
                                db,
                                format!("{}'element", object_identifier),
                                indent_style,
                            )?);
                        }
                        Ok(format!("( {} )", positionals.join(", ")))
                    }
                    ArrayAssignment::Sliced { direct, others } => {
                        let mut field_assignments = Vec::new();
                        for ra in direct {
                            field_assignments.push(format!(
                                "\n{} => {}",
                                ra.constraint().as_slice_index(),
                                ra.assignment().declare_for(
                                    db,
                                    format!("{}'element", object_identifier),
                                    indent_style,
                                )?
                            ));
                        }
                        if let Some(value) = others {
                            field_assignments.push(format!(
                                "\nothers => {}",
                                value.declare_for(
                                    db,
                                    format!("{}'element", object_identifier),
                                    indent_style,
                                )?
                            ));
                        }
                        Ok(format!(
                            "({}\n)",
                            indent(&field_assignments.join(","), indent_style),
                        ))
                    }
                    ArrayAssignment::Others(value) => Ok(format!(
                        "( others => {} )",
                        value.declare_for(
                            db,
                            format!("{}'element", object_identifier),
                            indent_style,
                        )?
                    )),
                },
            },
        }
    }
}

/// An object can be assigned a value or another object
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectSelection {
    /// The object being assigned from
    object: Id<ObjectDeclaration>,
    /// Optional selections on the object being assigned from, representing nested selections
    from_field: Vec<FieldSelection>,
}

impl ObjectSelection {
    /// Returns a reference to the object being assigned from
    pub fn object(&self) -> Id<ObjectDeclaration> {
        self.object
    }

    /// Select fields from the object being assigned
    pub fn assign_from(mut self, fields: &Vec<FieldSelection>) -> Self {
        self.from_field.append(&mut fields.clone());

        self
    }

    pub fn from_field(&self) -> &Vec<FieldSelection> {
        &self.from_field
    }

    pub fn as_object_key(&self, db: &dyn Arch) -> ObjectKey {
        db.lookup_intern_object_declaration(self.object())
            .object_key()
            .clone()
            .with_nested(self.from_field().clone())
    }
}

pub trait SelectObject: Sized {
    fn select(self, field: impl TryResult<FieldSelection>) -> Result<ObjectSelection>;
    fn select_nested(
        self,
        fields: impl IntoIterator<Item = impl TryResult<FieldSelection>>,
    ) -> Result<ObjectSelection>;
}

impl<T: TryResult<ObjectSelection>> SelectObject for T {
    fn select_nested(
        self,
        fields: impl IntoIterator<Item = impl TryResult<FieldSelection>>,
    ) -> Result<ObjectSelection> {
        let mut selection = self.try_result()?;
        let mut fields_result = vec![];
        for field in fields {
            fields_result.push(field.try_result()?);
        }
        selection.from_field.append(&mut fields_result);
        Ok(selection)
    }

    fn select(self, field: impl TryResult<FieldSelection>) -> Result<ObjectSelection> {
        let mut selection = self.try_result()?;
        selection.from_field.push(field.try_result()?);
        Ok(selection)
    }
}

impl DeclareWithIndent for ObjectSelection {
    fn declare_with_indent(&self, db: &dyn Arch, indent_style: &str) -> Result<String> {
        let mut result = db
            .lookup_intern_object_declaration(self.object())
            .identifier()
            .to_string();
        for field in self.from_field() {
            result.push_str(&field.declare_with_indent(db, indent_style)?);
        }
        Ok(result)
    }
}

/// Possible values which can be assigned to std_logic
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StdLogicValue {
    /// Uninitialized, 'U'
    U,
    /// Unknown, 'X',
    X,
    /// Logic, '0' or '1'
    Logic(bool),
    /// High Impedance, 'Z'
    Z,
    /// Weak signal (either '0' or '1'), 'W'
    W,
    /// Weak signal (likely '0'), 'L'
    L,
    /// Weak signal (likely '1'), 'H'
    H,
    /// Don't care, '-'
    DontCare,
}

impl StdLogicValue {
    pub fn from_char(val: char) -> Result<StdLogicValue> {
        match val {
            'U' => Ok(StdLogicValue::U),
            'X' => Ok(StdLogicValue::X),
            '1' => Ok(StdLogicValue::Logic(true)),
            '0' => Ok(StdLogicValue::Logic(false)),
            'Z' => Ok(StdLogicValue::Z),
            'W' => Ok(StdLogicValue::W),
            'L' => Ok(StdLogicValue::L),
            'H' => Ok(StdLogicValue::H),
            '-' => Ok(StdLogicValue::DontCare),
            _ => Err(Error::InvalidArgument(format!(
                "Unsupported std_logic value {}",
                val
            ))),
        }
    }
}

impl fmt::Display for StdLogicValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = match self {
            StdLogicValue::U => "U",
            StdLogicValue::X => "X",
            StdLogicValue::Logic(value) => {
                if *value {
                    "1"
                } else {
                    "0"
                }
            }
            StdLogicValue::Z => "Z",
            StdLogicValue::W => "W",
            StdLogicValue::L => "L",
            StdLogicValue::H => "H",
            StdLogicValue::DontCare => "-",
        };
        write!(f, "{}", symbol)
    }
}

/// Directly assigning a value or an entire Record/Array
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DirectAssignment {
    /// Assigning all fields of a Record
    FullRecord(Vec<FieldAssignment>),
    /// Assigning all fields of an Array
    FullArray(ArrayAssignment),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FieldAssignment {
    field: VhdlName,
    assignment: AssignmentKind,
}

impl FieldAssignment {
    pub fn new(field: VhdlName, assignment: AssignmentKind) -> Self {
        FieldAssignment { field, assignment }
    }

    pub fn field(&self) -> &VhdlName {
        &self.field
    }

    pub fn assignment(&self) -> &AssignmentKind {
        &self.assignment
    }
}

impl Identify for FieldAssignment {
    fn identifier(&self) -> String {
        self.field().to_string()
    }
}

/// Directly assigning a value or an entire Record, corresponds to the Types defined in `tydi::generator::common::Type`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ValueAssignment {
    /// Assigning a boolean to something
    Boolean(bool),
    /// Assigning an amount of time to something
    Time(TimeValue),
    /// Assigning a value to a single bit
    Bit(StdLogicValue),
    /// Assigning a value to a (part of) a bit vector
    BitVec(BitVecValue),
    /// Assigning a value to an integer
    Integer(i32),
}

impl ValueAssignment {
    pub fn declare(&self) -> Result<String> {
        match self {
            ValueAssignment::Bit(b) => Ok(format!("'{}'", b)),
            ValueAssignment::BitVec(bv) => bv.declare(),
            ValueAssignment::Time(t) => t.declare(),
            ValueAssignment::Boolean(b) => Ok(b.to_string()),
            ValueAssignment::Integer(i) => Ok(i.to_string()),
        }
    }

    pub fn matching_value(&self, other: &ValueAssignment) -> bool {
        match self {
            ValueAssignment::Boolean(_) => match other {
                ValueAssignment::Boolean(_) => true,
                _ => false,
            },
            ValueAssignment::Time(_) => match other {
                ValueAssignment::Time(_) => true,
                _ => false,
            },
            ValueAssignment::Bit(_) => match other {
                ValueAssignment::Bit(_) => true,
                _ => false,
            },
            ValueAssignment::BitVec(bv) => match other {
                ValueAssignment::BitVec(obv) => bv.matching_bitvec(obv),
                _ => false,
            },
            ValueAssignment::Integer(_) => match other {
                ValueAssignment::Integer(_) => true,
                _ => false,
            },
        }
    }

    pub fn can_assign(&self, to_typ: &ObjectType) -> Result<()> {
        match self {
            ValueAssignment::Bit(_) => match to_typ {
                ObjectType::Bit => Ok(()),
                ObjectType::Array(_)
                | ObjectType::Record(_)
                | ObjectType::Time
                | ObjectType::Boolean
                | ObjectType::Integer(_) => Err(Error::InvalidTarget(format!(
                    "Cannot assign Bit to {}",
                    to_typ
                ))),
            },
            ValueAssignment::BitVec(bitvec) => match to_typ {
                ObjectType::Array(array) if array.is_bitvector() => {
                    if let Some(w) = array.width()? {
                        bitvec.validate_width(w)
                    } else {
                        Ok(())
                    }
                }
                ObjectType::Array(_)
                | ObjectType::Bit
                | ObjectType::Record(_)
                | ObjectType::Time
                | ObjectType::Boolean
                | ObjectType::Integer(_) => Err(Error::InvalidTarget(format!(
                    "Cannot assign Bit Vector to {}",
                    to_typ
                ))),
            },
            ValueAssignment::Time(_) => match to_typ {
                ObjectType::Time => Ok(()),
                ObjectType::Bit
                | ObjectType::Record(_)
                | ObjectType::Array(_)
                | ObjectType::Boolean
                | ObjectType::Integer(_) => Err(Error::InvalidTarget(format!(
                    "Cannot assign Time to {}",
                    to_typ
                ))),
            },
            ValueAssignment::Boolean(_) => match to_typ {
                ObjectType::Boolean => Ok(()),
                ObjectType::Bit
                | ObjectType::Record(_)
                | ObjectType::Array(_)
                | ObjectType::Time
                | ObjectType::Integer(_) => Err(Error::InvalidTarget(format!(
                    "Cannot assign boolean to {}",
                    to_typ
                ))),
            },
            ValueAssignment::Integer(_) => match to_typ {
                ObjectType::Integer(_) => Ok(()),
                ObjectType::Bit
                | ObjectType::Record(_)
                | ObjectType::Array(_)
                | ObjectType::Time
                | ObjectType::Boolean => Err(Error::InvalidTarget(format!(
                    "Cannot assign boolean to {}",
                    to_typ
                ))),
            },
        }
    }
}

/// A VHDL assignment constraint
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FieldSelection {
    /// The most common kind of constraint, a specific range or index
    Range(RangeConstraint),
    /// The field of a record
    Name(VhdlName),
}

// impl fmt::Display for FieldSelection {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match self {
//             FieldSelection::Range(range) => range.fmt(f),
//             FieldSelection::Name(name) => write!(f, ".{}", name),
//         }
//     }
// }

impl FieldSelection {
    pub fn to(start: impl Into<i32>, end: impl Into<i32>) -> Result<FieldSelection> {
        Ok(FieldSelection::Range(RangeConstraint::to(
            start.into(),
            end.into(),
        )?))
    }

    pub fn downto(start: impl Into<i32>, end: impl Into<i32>) -> Result<FieldSelection> {
        Ok(FieldSelection::Range(RangeConstraint::downto(
            start.into(),
            end.into(),
        )?))
    }

    pub fn relation_to(
        db: &dyn Arch,
        start: impl Into<Relation>,
        end: impl Into<Relation>,
    ) -> Result<FieldSelection> {
        Ok(FieldSelection::Range(RangeConstraint::relation_downto(
            db,
            start.into(),
            end.into(),
        )?))
    }

    pub fn relation_downto(
        db: &dyn Arch,
        start: impl Into<Relation>,
        end: impl Into<Relation>,
    ) -> Result<FieldSelection> {
        Ok(FieldSelection::Range(RangeConstraint::relation_downto(
            db,
            start.into(),
            end.into(),
        )?))
    }

    pub fn index(index: impl Into<Relation>) -> FieldSelection {
        FieldSelection::Range(RangeConstraint::Index(index.into()))
    }

    pub fn try_name(name: impl TryResult<VhdlName>) -> Result<FieldSelection> {
        Ok(FieldSelection::Name(name.try_result()?))
    }

    pub fn name(name: impl Into<VhdlName>) -> FieldSelection {
        FieldSelection::Name(name.into())
    }
}

impl TryFrom<&str> for FieldSelection {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        FieldSelection::try_name(value)
    }
}

impl From<VhdlName> for FieldSelection {
    fn from(name: VhdlName) -> Self {
        FieldSelection::Name(name)
    }
}

impl DeclareWithIndent for FieldSelection {
    fn declare_with_indent(&self, db: &dyn Arch, indent_style: &str) -> Result<String> {
        Ok(match self {
            FieldSelection::Range(range) => range.declare_with_indent(db, indent_style)?,
            FieldSelection::Name(name) => format!(".{}", name),
        })
    }
}

pub enum FixedRangeConstraint {
    To { start: i32, end: i32 },
    Downto { start: i32, end: i32 },
    Index(i32),
}

impl FixedRangeConstraint {
    /// Returns the greatest index within the range constraint
    pub fn high(&self) -> i32 {
        match self {
            FixedRangeConstraint::To { start: _, end } => *end,
            FixedRangeConstraint::Downto { start, end: _ } => *start,
            FixedRangeConstraint::Index(index) => *index,
        }
    }

    /// Returns the smallest index within the range constraint
    pub fn low(&self) -> i32 {
        match self {
            FixedRangeConstraint::To { start, end: _ } => *start,
            FixedRangeConstraint::Downto { start: _, end } => *end,
            FixedRangeConstraint::Index(index) => *index,
        }
    }
}

/// A VHDL range constraint
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RangeConstraint {
    /// A range [start] to [end]
    To { start: Relation, end: Relation },
    /// A range [start] downto [end]
    Downto { start: Relation, end: Relation },
    /// An index within a range
    Index(Relation),
}

// impl fmt::Display for RangeConstraint {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match self {
//             RangeConstraint::To { start, end } => write!(f, "({} to {})", start, end),
//             RangeConstraint::Downto { start, end } => write!(f, "({} downto {})", start, end),
//             RangeConstraint::Index(index) => write!(f, "({})", index),
//         }
//     }
// }

impl RangeConstraint {
    // TODO: This should be a result, to propagate the try_eval errors
    pub fn fixed(&self) -> Result<Option<FixedRangeConstraint>> {
        Ok(match self {
            RangeConstraint::To { start, end } => match (start.try_eval()?, end.try_eval()?) {
                (Some(ValueAssignment::Integer(start)), Some(ValueAssignment::Integer(end))) => {
                    Some(FixedRangeConstraint::To { start, end })
                }
                _ => None,
            },
            RangeConstraint::Downto { start, end } => match (start.try_eval()?, end.try_eval()?) {
                (Some(ValueAssignment::Integer(start)), Some(ValueAssignment::Integer(end))) => {
                    Some(FixedRangeConstraint::Downto { start, end })
                }
                _ => None,
            },
            RangeConstraint::Index(r) => {
                if let Some(ValueAssignment::Integer(i)) = r.try_eval()? {
                    Some(FixedRangeConstraint::Index(i))
                } else {
                    None
                }
            }
        })
    }

    /// Create a `RangeConstraint::To` and ensure correctness (end > start)
    pub fn to(start: i32, end: i32) -> Result<RangeConstraint> {
        if start > end {
            Err(Error::InvalidArgument(format!(
                "{} > {}!\nStart cannot be greater than end when constraining a range [start] to [end]",
                start, end
            )))
        } else {
            Ok(RangeConstraint::To {
                start: start.into(),
                end: end.into(),
            })
        }
    }

    pub fn relation_to(
        db: &dyn Arch,
        start: impl Into<Relation>,
        end: impl Into<Relation>,
    ) -> Result<RangeConstraint> {
        let start = start.into();
        let end = end.into();
        start.is_integer(db)?;
        end.is_integer(db)?;
        Ok(RangeConstraint::To { start, end })
    }

    /// Create a `RangeConstraint::DownTo` and ensure correctness (start > end)
    pub fn downto(start: i32, end: i32) -> Result<RangeConstraint> {
        if end > start {
            Err(Error::InvalidArgument(format!(
                "{} > {}!\nEnd cannot be greater than start when constraining a range [start] downto [end]",
                end, start
            )))
        } else {
            Ok(RangeConstraint::Downto {
                start: start.into(),
                end: end.into(),
            })
        }
    }

    pub fn relation_downto(
        db: &dyn Arch,
        start: impl Into<Relation>,
        end: impl Into<Relation>,
    ) -> Result<RangeConstraint> {
        let start = start.into();
        let end = end.into();
        start.is_integer(db)?;
        end.is_integer(db)?;
        Ok(RangeConstraint::Downto { start, end })
    }

    /// Returns the width of the range
    pub fn width(&self) -> Result<Option<Width>> {
        let fixed = self.fixed()?;
        Ok(if let Some(fixed) = fixed {
            Some(match fixed {
                FixedRangeConstraint::To { start, end } => {
                    Width::Vector((1 + end - start).try_into().map_err(|err| {
                        Error::BackEndError(format!(
                            "Something went wrong calculating the width of a range constraint: {}",
                            err
                        ))
                    })?)
                }
                FixedRangeConstraint::Downto { start, end } => {
                    Width::Vector((1 + start - end).try_into().map_err(|err| {
                        Error::BackEndError(format!(
                            "Something went wrong calculating the width of a range constraint: {}",
                            err
                        ))
                    })?)
                }
                FixedRangeConstraint::Index(_) => Width::Scalar,
            })
        } else {
            None
        })
    }

    /// Returns the width of the range
    pub fn width_u32(&self) -> Result<Option<u32>> {
        let width = self.width()?;
        Ok(width.map(|w| match w {
            Width::Scalar => 1,
            Width::Vector(width) => width,
        }))
    }

    /// Returns the greatest index within the range constraint
    pub fn high(&self) -> &Relation {
        match self {
            RangeConstraint::To { start: _, end } => end,
            RangeConstraint::Downto { start, end: _ } => start,
            RangeConstraint::Index(index) => index,
        }
    }

    /// Returns the smallest index within the range constraint
    pub fn low(&self) -> &Relation {
        match self {
            RangeConstraint::To { start, end: _ } => start,
            RangeConstraint::Downto { start: _, end } => end,
            RangeConstraint::Index(index) => index,
        }
    }

    /// Verifies whether a range constraint overlaps with this range constraint
    pub fn overlaps(&self, other: &RangeConstraint) -> Result<bool> {
        Ok(match (self.fixed()?, other.fixed()?) {
            (Some(lhs), Some(rhs)) => lhs.low() <= rhs.high() && rhs.low() <= lhs.high(),
            _ => true,
        })
    }

    /// Verifies whether a range constraint is inside of this range constraint
    pub fn contains(&self, other: &RangeConstraint) -> Result<bool> {
        Ok(match (self.fixed()?, other.fixed()?) {
            (Some(lhs), Some(rhs)) => lhs.high() >= rhs.high() && lhs.low() <= rhs.low(),
            _ => true,
        })
    }

    /// Verifies whether this range constraint is between `high` and `low`
    pub fn is_between(&self, high: &Relation, low: &Relation) -> Result<bool> {
        match (self.fixed()?, high.try_eval()?, low.try_eval()?) {
            (
                Some(fixed),
                Some(ValueAssignment::Integer(high)),
                Some(ValueAssignment::Integer(low)),
            ) => {
                if low > high {
                    Err(Error::InvalidArgument(format!(
                        "{} > {}! Low cannot be greater than high",
                        low, high
                    )))
                } else {
                    Ok(high >= fixed.high() && low <= fixed.low())
                }
            }
            _ => Ok(true),
        }
    }

    pub fn as_slice_index(&self) -> String {
        match self {
            RangeConstraint::To { start, end } => format!("{} to {}", start, end),
            RangeConstraint::Downto { start, end } => format!("{} downto {}", start, end),
            RangeConstraint::Index(index) => format!("{}", index),
        }
    }
}

impl DeclareWithIndent for RangeConstraint {
    fn declare_with_indent(&self, db: &dyn Arch, indent_style: &str) -> Result<String> {
        Ok(match self {
            RangeConstraint::To { start, end } => format!(
                "({} to {})",
                start.declare_with_indent(db, indent_style)?,
                end.declare_with_indent(db, indent_style)?
            ),
            RangeConstraint::Downto { start, end } => format!(
                "({} downto {})",
                start.declare_with_indent(db, indent_style)?,
                end.declare_with_indent(db, indent_style)?
            ),
            RangeConstraint::Index(index) => {
                format!("({})", index.declare_with_indent(db, indent_style)?)
            }
        })
    }
}
