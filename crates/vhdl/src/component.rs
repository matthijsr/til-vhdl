use itertools::Itertools;
use textwrap::indent;

use tydi_common::{
    error::{Result, TryResult},
    traits::{Document, Identify},
};

use crate::{
    architecture::arch_storage::Arch,
    common::vhdl_name::{VhdlName, VhdlNameSelf},
    declaration::{Declare, DeclareWithIndent},
    port::{Parameter, Port},
    properties::Analyze,
    traits::VhdlDocument, object::object_type::DeclarationTypeName,
};
use crate::object::object_type::ObjectType;

/// A component.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Component {
    /// Component identifier.
    identifier: VhdlName,
    /// The parameters of the component..
    parameters: Vec<Parameter>,
    /// The ports of the component.
    ports: Vec<Port>,
    /// Documentation.
    doc: Option<String>,
}

impl Component {
    /// Create a new component.
    pub fn try_new(
        identifier: impl TryResult<VhdlName>,
        parameters: Vec<Parameter>,
        ports: Vec<Port>,
        doc: Option<String>,
    ) -> Result<Component> {
        Ok(Component {
            identifier: identifier.try_result()?,
            parameters,
            ports,
            doc,
        })
    }

    /// Return a reference to the ports of this component.
    pub fn ports(&self) -> &Vec<Port> {
        &self.ports
    }

    /// Return a reference to the parameters of this component.
    pub fn parameters(&self) -> &Vec<Parameter> {
        &self.parameters
    }

    /// Return this component with documentation added.
    pub fn with_doc(mut self, doc: impl Into<String>) -> Self {
        self.doc = Some(doc.into());
        self
    }

    /// Set the documentation of this component.
    pub fn set_doc(&mut self, doc: impl Into<String>) {
        self.doc = Some(doc.into())
    }

    pub fn name(&self) -> &VhdlName {
        &self.identifier
    }
}

impl Identify for Component {
    fn identifier(&self) -> String {
        self.identifier.to_string()
    }
}

impl VhdlNameSelf for Component {
    fn vhdl_name(&self) -> &VhdlName {
        &self.identifier
    }
}

impl Document for Component {
    fn doc(&self) -> Option<String> {
        self.doc.clone()
    }
}

impl Analyze for Component {
    fn list_nested_types(&self) -> Vec<ObjectType> {
        let mut result: Vec<ObjectType> = vec![];
        for p in self.ports().iter() {
            result.append(&mut p.typ().list_nested_types())
        }
        result.into_iter().unique_by(|x| x.declaration_type_name()).collect()
    }
}

impl DeclareWithIndent for Component {
    fn declare_with_indent(&self, db: &dyn Arch, indent_style: &str) -> Result<String> {
        let mut result = String::new();
        if let Some(doc) = self.vhdl_doc() {
            result.push_str(doc.as_str());
        }
        result.push_str(format!("component {}\n", self.identifier()).as_str());
        let mut port_body = "port (\n".to_string();
        let ports = self
            .ports()
            .iter()
            .map(|x| x.declare(db))
            .collect::<Result<Vec<String>>>()?
            .join(";\n");
        port_body.push_str(&indent(&ports, indent_style));
        port_body.push_str("\n);\n");
        result.push_str(&indent(&port_body, indent_style));
        result.push_str("end component;");
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::{architecture::arch_storage::db::Database, port::Mode, test_tools};

    use super::*;

    #[test]
    fn test_declare() -> Result<()> {
        let db = Database::default();
        let empty_comp = test_tools::empty_component().with_doc("test\ntest");
        assert_eq!(
            r#"-- test
-- test
component empty_component
  port (

  );
end component;"#,
            empty_comp.declare(&db)?
        );
        let port1 = Port::try_new_documented(
            "some_port",
            Mode::In,
            ObjectType::Bit,
            "This is port documentation\nNext line.",
        )?;
        let port2 = Port::try_new("some_other_port", Mode::Out, 43..0)?;
        let clk = Port::clk();
        let comp = Component::try_new("test", vec![], vec![port1, port2, clk], None)?;
        assert_eq!(
            r#"component test
  port (
    -- This is port documentation
    -- Next line.
    some_port : in std_logic;
    some_other_port : out std_logic_vector(43 downto 0);
    clk : in std_logic
  );
end component;"#,
            comp.declare(&db)?
        );

        Ok(())
    }
}
