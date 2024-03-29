use til_query::ir::generics::param_value::{
    combination::{Combination, MathCombination, MathOperator},
    GenericParamValue,
};
use tydi_common::{
    error::Result,
    map::InsertionOrderedMap,
    name::{Name, NameSelf},
};
use tydi_intern::Id;
use tydi_vhdl::{
    architecture::arch_storage::Arch,
    declaration::ObjectDeclaration,
    statement::relation::{
        math::{CreateMath, MathExpression},
        Relation,
    },
};

pub fn math_combination_to_relation(
    arch_db: &dyn Arch,
    math: &MathCombination,
    parent_params: &InsertionOrderedMap<Name, Id<ObjectDeclaration>>,
) -> Result<Relation> {
    match math {
        MathCombination::Parentheses(m) => {
            Relation::parentheses(math_combination_to_relation(arch_db, m, parent_params)?)
        }
        MathCombination::Negative(n) => Ok(MathExpression::negative(
            arch_db,
            param_value_to_vhdl(arch_db, n, parent_params)?,
        )?
        .into()),
        MathCombination::Combination(l, op, r) => {
            let left = param_value_to_vhdl(arch_db, l, parent_params)?;
            let right = param_value_to_vhdl(arch_db, r, parent_params)?;
            Ok(match op {
                MathOperator::Add => left.r_add(arch_db, right)?.into(),
                MathOperator::Subtract => left.r_subtract(arch_db, right)?.into(),
                MathOperator::Multiply => left.r_multiply(arch_db, right)?.into(),
                MathOperator::Divide => left.r_divide_by(arch_db, right)?.into(),
                MathOperator::Modulo => left.r_mod(arch_db, right)?.into(),
            })
        }
    }
}

pub fn param_value_to_vhdl(
    arch_db: &dyn Arch,
    val: &GenericParamValue,
    parent_params: &InsertionOrderedMap<Name, Id<ObjectDeclaration>>,
) -> Result<Relation> {
    match val {
        GenericParamValue::Integer(i) => Ok((*i).into()),
        GenericParamValue::Ref(r) => {
            let param = *parent_params.try_get(r.name())?;
            Ok(param.into())
        }
        GenericParamValue::Combination(c) => match c {
            Combination::Math(m) => {
                Ok(math_combination_to_relation(arch_db, m, parent_params)?.into())
            }
        },
    }
}
