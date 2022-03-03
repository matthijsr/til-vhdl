use super::fields::Fields;

#[derive(Debug, Clone, PartialEq)]
pub enum ElementType<T: Clone + PartialEq> {
    Null,
    Bits(T),
    Group(Fields<T>),
    Union(UnionElement<T>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnionElement<T: Clone + PartialEq> {
    fields: Fields<T>,
    data: T,
    tag: T,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Element<T: Clone + PartialEq> {
    data: ElementType<T>,
    last: Option<T>,
    stai: Option<T>,
    endi: Option<T>,
    strb: Option<T>,
}
