use til_query::common::logical::logicaltype::genericproperty::{
    GenericProperty, GenericPropertyOperator,
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
                GenericPropertyOperator::Add => Relation::from(l.r_add(db, r)?),
                GenericPropertyOperator::Subtract => Relation::from(l.r_subtract(db, r)?),
                GenericPropertyOperator::Multiply => Relation::from(l.r_multiply(db, r)?),
                GenericPropertyOperator::Divide => Relation::from(l.r_divide_by(db, r)?),
            }
        }
        GenericProperty::Fixed(f) => Relation::from(u32_to_i32(*f)?),
        GenericProperty::Parameterized(n) => Relation::from(*(parent_params.try_get(n)?)),
    })
}
