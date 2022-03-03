use super::fields::Fields;

#[derive(Debug, Clone, PartialEq)]
pub struct Element<T: Clone + PartialEq> {
    data: Fields<T>,
    last: Option<T>,
    stai: Option<T>,
    endi: Option<T>,
    strb: Option<T>,
}
