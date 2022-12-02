use core::fmt;

use tydi_common::error::{Error, Result, TryResult};

use crate::{
    architecture::arch_storage::Arch,
    assignment::{bitvec::BitVecValue, ObjectSelection, StdLogicValue, ValueAssignment},
    common::vhdl_name::VhdlName,
    declaration::DeclareWithIndent,
    object::object_type::ObjectType,
    usings::{ListUsings, Usings},
};

use self::edge::Edge;

pub mod edge;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LogicalOperator {
    And,
    Or,
    Xor,
    Nand,
    Nor,
    Xnor,
}

impl fmt::Display for LogicalOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogicalOperator::And => write!(f, "and"),
            LogicalOperator::Or => write!(f, "or"),
            LogicalOperator::Xor => write!(f, "xor"),
            LogicalOperator::Nand => write!(f, "nand"),
            LogicalOperator::Nor => write!(f, "nor"),
            LogicalOperator::Xnor => write!(f, "xnor"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LogicalExpression {
    left: Box<Relation>,
    right: Box<Relation>,
    operator: LogicalOperator,
}

impl LogicalExpression {
    /// Get a reference to the logical expression's left.
    #[must_use]
    pub fn left(&self) -> &Relation {
        self.left.as_ref()
    }

    /// Get a reference to the logical expression's right.
    #[must_use]
    pub fn right(&self) -> &Relation {
        self.right.as_ref()
    }

    /// Get a reference to the logical expression's operator.
    #[must_use]
    pub fn operator(&self) -> &LogicalOperator {
        &self.operator
    }

    pub fn can_assign(&self, db: &dyn Arch, to_typ: &ObjectType) -> Result<()> {
        self.left().can_assign(db, to_typ)
    }
}

impl DeclareWithIndent for LogicalExpression {
    fn declare_with_indent(&self, db: &dyn Arch, indent_style: &str) -> Result<String> {
        Ok(format!(
            "{} {} {}",
            self.left().declare_with_indent(db, indent_style)?,
            self.operator(),
            self.right().declare_with_indent(db, indent_style)?
        ))
    }
}

pub trait CreateLogicalExpression: Sized {
    fn validate_lex(
        db: &dyn Arch,
        left: impl TryResult<Relation>,
        right: impl TryResult<Relation>,
    ) -> Result<(Box<Relation>, Box<Relation>)> {
        let left = left.try_result()?;
        let right = right.try_result()?;
        left.matching_relation(db, &right)?;
        Ok((Box::new(left), Box::new(right)))
    }

    fn and(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<LogicalExpression>;
    fn or(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<LogicalExpression>;
    fn xor(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<LogicalExpression>;
    fn nand(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<LogicalExpression>;
    fn nor(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<LogicalExpression>;
    fn xnor(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<LogicalExpression>;
}

impl<T: TryResult<Relation>> CreateLogicalExpression for T {
    fn and(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<LogicalExpression> {
        let (left, right) = Self::validate_lex(db, self, right)?;
        Ok(LogicalExpression {
            left,
            right,
            operator: LogicalOperator::And,
        })
    }

    fn or(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<LogicalExpression> {
        let (left, right) = Self::validate_lex(db, self, right)?;
        Ok(LogicalExpression {
            left,
            right,
            operator: LogicalOperator::Or,
        })
    }

    fn xor(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<LogicalExpression> {
        let (left, right) = Self::validate_lex(db, self, right)?;
        Ok(LogicalExpression {
            left,
            right,
            operator: LogicalOperator::Xor,
        })
    }

    fn nand(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<LogicalExpression> {
        let (left, right) = Self::validate_lex(db, self, right)?;
        Ok(LogicalExpression {
            left,
            right,
            operator: LogicalOperator::Nand,
        })
    }

    fn nor(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<LogicalExpression> {
        let (left, right) = Self::validate_lex(db, self, right)?;
        Ok(LogicalExpression {
            left,
            right,
            operator: LogicalOperator::Nor,
        })
    }

    fn xnor(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<LogicalExpression> {
        let (left, right) = Self::validate_lex(db, self, right)?;
        Ok(LogicalExpression {
            left,
            right,
            operator: LogicalOperator::Xnor,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RelationalOperator {
    Eq,
    NEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
}

impl fmt::Display for RelationalOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RelationalOperator::Eq => write!(f, "="),
            RelationalOperator::NEq => write!(f, "/="),
            RelationalOperator::Lt => write!(f, "<"),
            RelationalOperator::LtEq => write!(f, "<="),
            RelationalOperator::Gt => write!(f, ">"),
            RelationalOperator::GtEq => write!(f, ">="),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RelationalCombination {
    left: Box<Relation>,
    right: Box<Relation>,
    operator: RelationalOperator,
}

impl RelationalCombination {
    /// Get a reference to the relational combination's left.
    #[must_use]
    pub fn left(&self) -> &Relation {
        self.left.as_ref()
    }

    /// Get a reference to the relational combination's right.
    #[must_use]
    pub fn right(&self) -> &Relation {
        self.right.as_ref()
    }

    /// Get a reference to the relational combination's operator.
    #[must_use]
    pub fn operator(&self) -> &RelationalOperator {
        &self.operator
    }
}

impl DeclareWithIndent for RelationalCombination {
    fn declare_with_indent(&self, db: &dyn Arch, indent_style: &str) -> Result<String> {
        Ok(format!(
            "{} {} {}",
            self.left().declare_with_indent(db, indent_style)?,
            self.operator(),
            self.right().declare_with_indent(db, indent_style)?
        ))
    }
}

pub trait CombineRelation: Sized {
    fn validate_combine(
        db: &dyn Arch,
        left: impl TryResult<Relation>,
        right: impl TryResult<Relation>,
    ) -> Result<(Box<Relation>, Box<Relation>)> {
        let left = left.try_result()?;
        let right = right.try_result()?;
        left.matching_relation(db, &right)?;
        Ok((Box::new(left), Box::new(right)))
    }

    fn r_eq(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<RelationalCombination>;
    fn r_neq(self, db: &dyn Arch, right: impl TryResult<Relation>)
        -> Result<RelationalCombination>;
    fn r_lt(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<RelationalCombination>;
    fn r_lteq(
        self,
        db: &dyn Arch,
        right: impl TryResult<Relation>,
    ) -> Result<RelationalCombination>;
    fn r_gt(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<RelationalCombination>;
    fn r_gteq(
        self,
        db: &dyn Arch,
        right: impl TryResult<Relation>,
    ) -> Result<RelationalCombination>;
}

impl<T: TryResult<Relation>> CombineRelation for T {
    fn r_eq(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<RelationalCombination> {
        let (left, right) = Self::validate_combine(db, self, right)?;
        Ok(RelationalCombination {
            left,
            right,
            operator: RelationalOperator::Eq,
        })
    }

    fn r_neq(
        self,
        db: &dyn Arch,
        right: impl TryResult<Relation>,
    ) -> Result<RelationalCombination> {
        let (left, right) = Self::validate_combine(db, self, right)?;
        Ok(RelationalCombination {
            left,
            right,
            operator: RelationalOperator::NEq,
        })
    }

    fn r_lt(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<RelationalCombination> {
        let (left, right) = Self::validate_combine(db, self, right)?;
        Ok(RelationalCombination {
            left,
            right,
            operator: RelationalOperator::Lt,
        })
    }

    fn r_lteq(
        self,
        db: &dyn Arch,
        right: impl TryResult<Relation>,
    ) -> Result<RelationalCombination> {
        let (left, right) = Self::validate_combine(db, self, right)?;
        Ok(RelationalCombination {
            left,
            right,
            operator: RelationalOperator::LtEq,
        })
    }

    fn r_gt(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<RelationalCombination> {
        let (left, right) = Self::validate_combine(db, self, right)?;
        Ok(RelationalCombination {
            left,
            right,
            operator: RelationalOperator::Gt,
        })
    }

    fn r_gteq(
        self,
        db: &dyn Arch,
        right: impl TryResult<Relation>,
    ) -> Result<RelationalCombination> {
        let (left, right) = Self::validate_combine(db, self, right)?;
        Ok(RelationalCombination {
            left,
            right,
            operator: RelationalOperator::GtEq,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Relation {
    Value(Box<ValueAssignment>),
    Object(ObjectSelection),
    Combination(RelationalCombination),
    Edge(Edge),
    LogicalExpression(LogicalExpression),
}

impl From<ValueAssignment> for Relation {
    fn from(val: ValueAssignment) -> Self {
        Self::Value(Box::new(val))
    }
}

impl<T: Into<ObjectSelection>> From<T> for Relation {
    fn from(val: T) -> Self {
        Self::Object(val.into())
    }
}

impl From<RelationalCombination> for Relation {
    fn from(val: RelationalCombination) -> Self {
        Self::Combination(val)
    }
}

impl From<LogicalExpression> for Relation {
    fn from(val: LogicalExpression) -> Self {
        Self::LogicalExpression(val)
    }
}

impl From<Edge> for Relation {
    fn from(edge: Edge) -> Self {
        Self::Edge(edge)
    }
}

impl From<StdLogicValue> for Relation {
    fn from(assignment: StdLogicValue) -> Self {
        Relation::from(ValueAssignment::from(assignment))
    }
}

impl From<BitVecValue> for Relation {
    fn from(assignment: BitVecValue) -> Self {
        Relation::from(ValueAssignment::from(assignment))
    }
}

impl fmt::Display for Relation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Relation::Value(_) => write!(f, "Value"),
            Relation::Object(_) => write!(f, "Object"),
            Relation::Combination(_) => write!(f, "Combination"),
            Relation::Edge(_) => write!(f, "Edge"),
            Relation::LogicalExpression(_) => write!(f, "LogicalExpression"),
        }
    }
}

impl Relation {
    pub fn is_bool(&self, db: &dyn Arch) -> Result<()> {
        self.can_assign(db, &ObjectType::Boolean)
    }

    pub fn can_assign(&self, db: &dyn Arch, to_typ: &ObjectType) -> Result<()> {
        match self {
            Relation::Value(v) => v.can_assign(to_typ),
            Relation::Object(o) => {
                let obj = db.get_object(o.as_object_key(db))?;
                obj.assignable.from_or_err()?;
                obj.typ(db).can_assign_type(to_typ)
            }
            Relation::Combination(_) | Relation::Edge(_) => {
                ObjectType::Boolean.can_assign_type(to_typ)
            }
            Relation::LogicalExpression(lex) => lex.can_assign(db, to_typ),
        }
    }

    pub fn can_be_assigned(&self, db: &dyn Arch, from_typ: &ObjectType) -> Result<()> {
        if let Relation::Object(o) = self {
            let obj = db.get_object(o.as_object_key(db))?;
            obj.assignable.to_or_err()?;
            from_typ.can_assign_type(&obj.typ(db))
        } else {
            Err(Error::InvalidTarget(format!(
                "Can only assign to objects, this relation is a {}",
                self
            )))
        }
    }

    pub fn matching_relation(&self, db: &dyn Arch, other: &Relation) -> Result<()> {
        match self {
            Relation::Value(v) => match other {
                Relation::Value(ov) => {
                    if v.matching_value(ov) {
                        Ok(())
                    } else {
                        Err(Error::InvalidArgument(format!(
                            "Cannot create a relation between {} and {}.",
                            v.declare()?,
                            ov.declare()?
                        )))
                    }
                }
                Relation::Object(o) => {
                    v.can_assign(db.get_object_type(o.as_object_key(db))?.as_ref())
                }
                Relation::Combination(_) | Relation::Edge(_) => match v.as_ref() {
                    ValueAssignment::Boolean(_) => Ok(()),
                    _ => Err(Error::InvalidArgument(format!(
                        "Cannot create a relation between {} and a boolean relation.",
                        v.declare()?,
                    ))),
                },
                Relation::LogicalExpression(lex) => self.matching_relation(db, lex.left()),
            },
            Relation::Object(o) => match other {
                Relation::Value(v) => {
                    v.can_assign(db.get_object_type(o.as_object_key(db))?.as_ref())
                }
                Relation::Object(oo) => db
                    .get_object_type(o.as_object_key(db))?
                    .can_assign_type(db.get_object_type(oo.as_object_key(db))?.as_ref()),
                Relation::Combination(_) | Relation::Edge(_) => ValueAssignment::Boolean(false)
                    .can_assign(db.get_object_type(o.as_object_key(db))?.as_ref()),
                Relation::LogicalExpression(lex) => self.matching_relation(db, lex.left()),
            },
            Relation::Combination(_) | Relation::Edge(_) => match other {
                Relation::Value(v) => match v.as_ref() {
                    ValueAssignment::Boolean(_) => Ok(()),
                    _ => Err(Error::InvalidArgument(format!(
                        "Cannot create a relation between {} and a boolean relation.",
                        v.declare()?,
                    ))),
                },
                Relation::Object(o) => ValueAssignment::Boolean(false)
                    .can_assign(db.get_object_type(o.as_object_key(db))?.as_ref()),
                Relation::Combination(_) | Relation::Edge(_) => Ok(()),
                Relation::LogicalExpression(lex) => self.matching_relation(db, lex.left()),
            },
            Relation::LogicalExpression(lex) => lex.right().matching_relation(db, other),
        }
    }
}

impl DeclareWithIndent for Relation {
    fn declare_with_indent(&self, db: &dyn Arch, indent_style: &str) -> Result<String> {
        match self {
            Relation::Value(v) => v.declare(),
            Relation::Object(obj) => obj.declare_with_indent(db, indent_style),
            Relation::Combination(c) => c.declare_with_indent(db, indent_style),
            Relation::LogicalExpression(lex) => lex.declare_with_indent(db, indent_style),
            Relation::Edge(e) => e.declare_with_indent(db, indent_style),
        }
    }
}

impl ListUsings for Relation {
    fn list_usings(&self) -> Result<Usings> {
        let mut usings = Usings::new_empty();
        match self {
            Relation::Value(value) => match value.as_ref() {
                ValueAssignment::Bit(_) => (),
                ValueAssignment::BitVec(bitvec) => match bitvec {
                    BitVecValue::Others(_) => (),
                    BitVecValue::Full(_) => (),
                    BitVecValue::Unsigned(_, _) | BitVecValue::Signed(_, _) => {
                        usings.add_using(VhdlName::try_new("ieee")?, "numeric_std.all")?;
                    }
                },
                ValueAssignment::Time(_) => (),
                ValueAssignment::Boolean(_) => (),
            },
            Relation::Object(_) => (),
            Relation::Combination(comb) => {
                usings.combine(&comb.left().list_usings()?);
                usings.combine(&comb.right().list_usings()?);
            }
            Relation::Edge(_) => (),
            Relation::LogicalExpression(lex) => {
                usings.combine(&lex.left().list_usings()?);
                usings.combine(&lex.right().list_usings()?);
            }
        }
        Ok(usings)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        architecture::arch_storage::db::Database,
        assignment::SelectObject,
        declaration::{Declare, ObjectDeclaration},
    };

    use super::*;

    #[test]
    fn test_declare() -> Result<()> {
        let _db = Database::default();
        let db = &_db;
        let lex = ValueAssignment::Boolean(true)
            .and(db, ValueAssignment::Boolean(true))?
            .or(db, ValueAssignment::Boolean(true))?
            .xor(db, ValueAssignment::Boolean(false))?
            .nand(db, ValueAssignment::Boolean(false))?
            .nor(db, ValueAssignment::Boolean(false))?
            .xnor(db, ValueAssignment::Boolean(true))?;
        assert_eq!(
            "true and true or true xor false nand false nor false xnor true",
            lex.declare(db)?
        );
        let comb = lex
            .clone()
            .r_eq(db, lex)?
            .r_neq(db, ValueAssignment::Boolean(false))?
            .r_lt(db, ValueAssignment::Boolean(false))?
            .r_lteq(db, ValueAssignment::Boolean(false))?
            .r_gt(db, ValueAssignment::Boolean(false))?
            .r_gteq(db, ValueAssignment::Boolean(false))?;
        assert_eq!(
            "true and true or true xor false nand false nor false xnor true = true and true or true xor false nand false nor false xnor true /= false < false <= false > false >= false",
            comb.declare(db)?
        );
        let obj1 = ObjectDeclaration::signal(db, "test_sig1", ObjectType::Bit, None)?;
        let obj2 = ObjectDeclaration::signal(db, "test_sig2", ObjectType::bit_vector(1, 0)?, None)?
            .select_nested([0])?;
        let rising_edge = Edge::rising_edge(db, obj1)?;
        let falling_edge = Edge::falling_edge(db, obj2)?;
        assert_eq!(
            "rising_edge(test_sig1) nand falling_edge(test_sig2(0))",
            rising_edge.nand(db, falling_edge)?.declare(db)?
        );

        Ok(())
    }
}
