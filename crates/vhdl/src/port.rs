use std::fmt::Display;

use tydi_common::{
    error::Result,
    name::Name,
    traits::{Document, Identify, Reverse, Reversed},
};

use crate::{
    architecture::arch_storage::Arch,
    common::vhdl_name::{VhdlName, VhdlNameSelf},
    declaration::Declare,
    object::ObjectType,
    traits::VhdlDocument,
};

/// A port.
#[derive(Debug, Clone, PartialEq)]
pub struct Port {
    /// Port identifier.
    identifier: VhdlName,
    /// Port mode.
    mode: Mode,
    /// Port type.
    typ: ObjectType,
    /// Port documentation.
    doc: Option<String>,
}

impl Port {
    /// Create a new port.
    pub fn new(name: impl Into<VhdlName>, mode: Mode, typ: ObjectType) -> Port {
        Port {
            identifier: name.into(),
            mode,
            typ,
            doc: None,
        }
    }

    /// Create a new port with documentation.
    pub fn new_documented(
        name: impl Into<VhdlName>,
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

    /// Create a new port with `Mode::In`
    pub fn new_in(name: impl Into<VhdlName>, typ: ObjectType) -> Port {
        Port::new(name, Mode::In, typ)
    }

    /// Create a new port with `Mode::Out`
    pub fn new_out(name: impl Into<VhdlName>, typ: ObjectType) -> Port {
        Port::new(name, Mode::Out, typ)
    }

    /// Create an in port with type `std_logic`
    pub fn bit_in(name: impl Into<VhdlName>) -> Port {
        Port::new(name, Mode::In, ObjectType::Bit)
    }

    /// Create an out port with type `std_logic`
    pub fn bit_out(name: impl Into<VhdlName>) -> Port {
        Port::new(name, Mode::Out, ObjectType::Bit)
    }

    /// Create a `clk` port, `clk : in std_logic`.
    pub fn clk() -> Port {
        Port::new(VhdlName::try_new("clk").unwrap(), Mode::In, ObjectType::Bit)
    }

    /// Create a `rst` port, `rst : in std_logic`.
    pub fn rst() -> Port {
        Port::new(VhdlName::try_new("rst").unwrap(), Mode::In, ObjectType::Bit)
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
    fn identifier(&self) -> String {
        self.identifier.to_string()
    }
}

impl VhdlNameSelf for Port {
    fn vhdl_name(&self) -> &VhdlName {
        &self.identifier
    }
}

impl Document for Port {
    fn doc(&self) -> Option<String> {
        self.doc.clone()
    }
}

impl Declare for Port {
    fn declare(&self, db: &dyn Arch) -> Result<String> {
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

impl Reversed for Mode {
    fn reversed(&self) -> Self {
        match self {
            Mode::In => Mode::Out,
            Mode::Out => Mode::In,
        }
    }
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
