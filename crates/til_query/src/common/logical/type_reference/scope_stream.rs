use tydi_common::name::PathName;

use super::TypeReference;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScopeStream<F: Clone + PartialEq> {
    pub name: PathName,
    pub child: Box<TypeReference<F>>,
}
