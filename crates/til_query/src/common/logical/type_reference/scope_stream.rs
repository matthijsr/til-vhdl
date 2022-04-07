use core::fmt;

use textwrap::indent;
use tydi_common::name::PathName;

use super::TypeReference;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScopeStream {
    name: PathName,
    child: Box<TypeReference>,
}

impl ScopeStream {
    pub fn new(name: PathName, child: Box<TypeReference>) -> Self {
        Self { name, child }
    }

    pub fn name(&self) -> &PathName {
        &self.name
    }
    pub fn child(&self) -> &Box<TypeReference> {
        &self.child
    }
}

impl fmt::Display for ScopeStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Scope (\n{}\n)",
            indent(
                &format!(
                    r#"name:  {},
child: {}"#,
                    self.name(),
                    self.child().as_ref()
                ),
                "  "
            )
        )
    }
}
