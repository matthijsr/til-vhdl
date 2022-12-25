use til_query::{
    common::{
        logical::logicaltype::genericproperty::GenericProperty, physical::stream::PhysicalBitCount,
    },
    ir::generics::param_value::combination::MathOperator,
};
use tydi_common::{
    error::Result,
    map::InsertionOrderedMap,
    name::Name,
    numbers::{u32_to_i32, NonNegative},
};
use tydi_intern::Id;
use tydi_vhdl::{
    architecture::arch_storage::Arch,
    declaration::ObjectDeclaration,
    statement::relation::{math::CreateMath, Relation},
};

// TODO: PhysicalBitCount should just be a GenericProperty<Positive>, probably
pub fn physical_bitcount_to_relation(
    db: &dyn Arch,
    bitcount: &PhysicalBitCount,
    parent_params: &InsertionOrderedMap<Name, Id<ObjectDeclaration>>,
) -> Result<Relation> {
    Ok(match bitcount {
        PhysicalBitCount::Combination(l, op, r) => {
            let l = Relation::parentheses(physical_bitcount_to_relation(db, l, parent_params)?)?;
            let r = physical_bitcount_to_relation(db, r, parent_params)?;
            match op {
                MathOperator::Add => Relation::from(l.r_add(db, r)?),
                MathOperator::Subtract => Relation::from(l.r_subtract(db, r)?),
                MathOperator::Multiply => Relation::from(l.r_multiply(db, r)?),
                MathOperator::Divide => Relation::from(l.r_divide_by(db, r)?),
                MathOperator::Modulo => Relation::from(l.r_mod(db, r)?),
            }
        }
        PhysicalBitCount::Fixed(f) => Relation::from(u32_to_i32(f.get())?),
        PhysicalBitCount::Parameterized(n) => Relation::from(*(parent_params.try_get(n)?)),
    })
}

pub fn generic_property_to_relation(
    db: &dyn Arch,
    property: &GenericProperty<NonNegative>,
    parent_params: &InsertionOrderedMap<Name, Id<ObjectDeclaration>>,
) -> Result<Relation> {
    Ok(match property {
        GenericProperty::Combination(l, op, r) => {
            let l = Relation::parentheses(generic_property_to_relation(db, l, parent_params)?)?;
            let r = generic_property_to_relation(db, r, parent_params)?;
            match op {
                MathOperator::Add => Relation::from(l.r_add(db, r)?),
                MathOperator::Subtract => Relation::from(l.r_subtract(db, r)?),
                MathOperator::Multiply => Relation::from(l.r_multiply(db, r)?),
                MathOperator::Divide => Relation::from(l.r_divide_by(db, r)?),
                MathOperator::Modulo => Relation::from(l.r_mod(db, r)?),
            }
        }
        GenericProperty::Fixed(f) => Relation::from(u32_to_i32(*f)?),
        GenericProperty::Parameterized(n) => Relation::from(*(parent_params.try_get(n)?)),
    })
}
