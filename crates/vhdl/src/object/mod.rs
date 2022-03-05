use object_type::ObjectType;

pub mod array;
pub mod object_from;
pub mod object_type;
pub mod record;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub struct Assignable {
    /// Can be assigned to
    pub to: bool,
    /// Can be assigned from
    pub from: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Object {
    typ: ObjectType,
    assignable: Assignable,
}
