use tydi_common::name::PathName;

use super::TypeReference;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScopeStream {
    pub name: PathName,
    pub child: Box<TypeReference>,
}
