use std::fmt::Display;

use tydi_common::{
    error::Result,
    traits::{Document, Identify},
};

use crate::{declaration::Declare, object::ObjectType, traits::VhdlDocument};

/// A port.
#[derive(Debug, Clone, PartialEq)]
pub struct Port {
    /// Port identifier.
    identifier: String,
    /// Port mode.
    mode: Mode,
    /// Port type.
    typ: ObjectType,
    /// Port documentation.
    doc: Option<String>,
}

impl Port {
    /// Create a new port.
    pub fn new(name: impl Into<String>, mode: Mode, typ: ObjectType) -> Port {
        Port {
            identifier: name.into(),
            mode,
            typ,
            doc: None,
        }
    }

    /// Create a new port with documentation.
    pub fn new_documented(
        name: impl Into<String>,
        mode: Mode,
        typ: ObjectType,
        doc: impl Into<String>,
    ) -> Port {
        Port {
            identifier: name.into(),
            mode,
            typ,
            doc: Some(doc.into()),
        }
    }

    /// Return the port mode.
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// Return the type of the port.
    pub fn typ(&self) -> &ObjectType {
        &self.typ
    }

    // /// Returns true if the port type contains reversed fields.
    // pub fn has_reversed(&self) -> bool {
    //     self.typ.has_reversed()
    // }

    /// Return this port with documentation added.
    pub fn with_doc(mut self, doc: impl Into<String>) -> Self {
        self.doc = Some(doc.into());
        self
    }

    /// Set the documentation of this port.
    pub fn set_doc(&mut self, doc: impl Into<String>) {
        self.doc = Some(doc.into())
    }
}

impl Identify for Port {
    fn identifier(&self) -> &str {
        self.identifier.as_str()
    }
}

impl Document for Port {
    fn doc(&self) -> Option<String> {
        self.doc.clone()
    }
}

impl Declare for Port {
    fn declare(&self) -> Result<String> {
        let mut result = String::new();
        if let Some(doc) = self.vhdl_doc() {
            result.push_str(doc.as_str());
        }
        result.push_str(
            format!(
                "{} : {} {}",
                self.identifier(),
                self.mode(),
                self.typ().type_name()
            )
            .as_str(),
        );
        Ok(result)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    In,
    Out,
}

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::In => write!(f, "in"),
            Mode::Out => write!(f, "out"),
        }
    }
}

/// A parameter for functions and components (generics).
/// TODO: Add specific types.
#[derive(Debug, Clone)]
pub struct Parameter {
    /// Parameter identifier.
    pub identifier: String,
}
