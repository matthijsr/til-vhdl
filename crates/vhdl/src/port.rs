use std::fmt::Display;

use tydi_common::{
    error::{Result, TryResult},
    traits::{Document, Identify, Reverse, Reversed},
};

use crate::{
    architecture::arch_storage::Arch,
    common::vhdl_name::{VhdlName, VhdlNameSelf},
    declaration::Declare,
    traits::VhdlDocument, object::object_type::DeclarationTypeName,
};
use crate::object::object_type::ObjectType;

/// A port.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
    pub fn try_new(
        name: impl TryResult<VhdlName>,
        mode: impl TryResult<Mode>,
        typ: impl TryResult<ObjectType>,
    ) -> Result<Self> {
        Ok(Port {
            identifier: name.try_result()?,
            mode: mode.try_result()?,
            typ: typ.try_result()?,
            doc: None,
        })
    }

    /// Create a new port with documentation.
    pub fn try_new_documented(
        name: impl TryResult<VhdlName>,
        mode: impl TryResult<Mode>,
        typ: impl TryResult<ObjectType>,
        doc: impl Into<String>,
    ) -> Result<Self> {
        Ok(Port {
            identifier: name.try_result()?,
            mode: mode.try_result()?,
            typ: typ.try_result()?,
            doc: Some(doc.into()),
        })
    }

    /// Create a new port with `Mode::In`
    pub fn try_new_in(
        name: impl TryResult<VhdlName>,
        typ: impl TryResult<ObjectType>,
    ) -> Result<Self> {
        Port::try_new(name, Mode::In, typ)
    }

    /// Create a new port with `Mode::Out`
    pub fn try_new_out(
        name: impl TryResult<VhdlName>,
        typ: impl TryResult<ObjectType>,
    ) -> Result<Self> {
        Port::try_new(name, Mode::Out, typ)
    }

    /// Create an in port with type `std_logic`
    pub fn try_bit_in(name: impl TryResult<VhdlName>) -> Result<Self> {
        Port::try_new(name, Mode::In, ObjectType::Bit)
    }

    /// Create an out port with type `std_logic`
    pub fn try_bit_out(name: impl TryResult<VhdlName>) -> Result<Self> {
        Port::try_new(name, Mode::Out, ObjectType::Bit)
    }

    /// Create a `clk` port, `clk : in std_logic`.
    pub fn clk() -> Port {
        Port::try_new("clk", Mode::In, ObjectType::Bit).unwrap()
    }

    /// Create a `rst` port, `rst : in std_logic`.
    pub fn rst() -> Port {
        Port::try_new("rst", Mode::In, ObjectType::Bit).unwrap()
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
    fn doc(&self) -> Option<&String> {
        self.doc.as_ref()
    }
}

impl Declare for Port {
    fn declare(&self, _db: &dyn Arch) -> Result<String> {
        let mut result = String::new();
        if let Some(doc) = self.vhdl_doc() {
            result.push_str(doc.as_str());
        }
        result.push_str(
            format!(
                "{} : {} {}",
                self.identifier(),
                self.mode(),
                self.typ().declaration_type_name()
            )
            .as_str(),
        );
        Ok(result)
    }
}

impl Reverse for Port {
    fn reverse(&mut self) {
        match self.mode() {
            Mode::In => self.mode = Mode::Out,
            Mode::Out => self.mode = Mode::In,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Parameter {
    /// Parameter identifier.
    pub identifier: VhdlName,
}
