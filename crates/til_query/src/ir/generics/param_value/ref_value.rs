use core::fmt;

use tydi_common::{
    name::{Name, NameSelf},
    traits::Identify,
};

use crate::ir::generics::{GenericKind, GenericParameter};

// TODO/Nice to have: If we could somehow evaluate the actual value of these
// once they're fully assigned, then test it against conditions, that would be
// super useful.
// Might be something I can figure out once we get to the assignment logic.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RefValue {
    name: Name,
    kind: GenericKind,
}

impl fmt::Display for RefValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name(), self.kind())
    }
}

impl RefValue {
    pub fn kind(&self) -> &GenericKind {
        &self.kind
    }
}

impl Identify for RefValue {
    fn identifier(&self) -> String {
        self.name().to_string()
    }
}

impl NameSelf for RefValue {
    fn name(&self) -> &Name {
        &self.name
    }
}

impl From<GenericParameter> for RefValue {
    fn from(param: GenericParameter) -> Self {
        Self {
            name: param.name,
            kind: param.kind,
        }
    }
}

impl From<&GenericParameter> for RefValue {
    fn from(param: &GenericParameter) -> Self {
        Self {
            name: param.name().clone(),
            kind: param.kind().clone(),
        }
    }
}
