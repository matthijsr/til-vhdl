use textwrap::indent;
use tydi_common::error::{Result, TryResult};
use tydi_common::traits::{Document, Documents, Identify};

use crate::architecture::arch_storage::Arch;
use crate::common::vhdl_name::{VhdlName, VhdlNameSelf};
use crate::declaration::DeclareWithIndent;
use crate::traits::VhdlDocument;
use crate::{
    component::Component,
    declaration::Declare,
    port::{GenericParameter, Port},
};

use super::Entity;

impl DeclareWithIndent for Entity {
    fn declare_with_indent(&self, db: &dyn Arch, indent_style: &str) -> Result<String> {
        let mut result = String::new();
        if let Some(doc) = self.vhdl_doc() {
            result.push_str(&doc);
        }
        result.push_str(format!("entity {} is\n", self.identifier()).as_str());

        if self.parameters().len() > 0 {
            let mut parameter_body = "generic (\n".to_string();
            let parameters = self
                .parameters()
                .iter()
                .map(|x| x.declare(db))
                .collect::<Result<Vec<String>>>()?
                .join(";\n");
            parameter_body.push_str(&indent(&parameters, indent_style));
            parameter_body.push_str("\n);\n");
            result.push_str(&indent(&parameter_body, indent_style));
        }

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

        result.push_str(format!("end {};\n", self.identifier()).as_str());
        Ok(result)
    }
}

impl Identify for Entity {
    fn identifier(&self) -> String {
        self.identifier.to_string()
    }
}

impl Document for Entity {
    fn doc(&self) -> Option<&String> {
        self.doc.as_ref()
    }
}

impl Documents for Entity {
    fn set_doc(&mut self, doc: impl Into<String>) {
        self.doc = Some(doc.into());
    }
}

impl Entity {
    /// Create a new entity.
    pub fn try_new(
        identifier: impl TryResult<VhdlName>,
        parameters: Vec<GenericParameter>,
        ports: Vec<Port>,
        doc: Option<String>,
    ) -> Result<Entity> {
        Ok(Entity {
            identifier: identifier.try_result()?,
            parameters,
            ports,
            doc,
        })
    }

    /// Return a reference to the ports of this entity.
    pub fn ports(&self) -> &Vec<Port> {
        &self.ports
    }

    /// Return a reference to the parameters of this entity.
    pub fn parameters(&self) -> &Vec<GenericParameter> {
        &self.parameters
    }
}

impl From<&Component> for Entity {
    fn from(comp: &Component) -> Self {
        Entity {
            identifier: comp.vhdl_name().clone(),
            parameters: comp.parameters().to_vec(),
            ports: comp.ports().to_vec(),
            doc: comp.doc().cloned(),
        }
    }
}

impl From<Component> for Entity {
    fn from(comp: Component) -> Self {
        Entity::from(&comp)
    }
}

#[cfg(test)]
mod tests {
    use crate::{architecture::arch_storage::db::Database, test_tools};

    use super::*;

    #[test]
    fn test_declare_empty() -> Result<()> {
        let db = Database::default();
        let empty_entity = Entity::from(test_tools::empty_component().with_doc("test\ntest"));
        assert_eq!(
            r#"-- test
-- test
entity empty_component is
  port (

  );
end empty_component;
"#,
            empty_entity.declare(&db)?
        );

        Ok(())
    }

    #[test]
    fn test_declare_ports() -> Result<()> {
        let db = Database::default();
        let entity = Entity::from(test_tools::simple_component()?);
        assert_eq!(
            r#"entity test is
  port (
    -- This is port documentation
    -- Next line.
    some_port : in std_logic;
    some_other_port : out std_logic_vector(43 downto 0);
    clk : in std_logic
  );
end test;
"#,
            entity.declare(&db)?
        );

        Ok(())
    }

    #[test]
    fn test_declare_generics() -> Result<()> {
        let db = Database::default();
        let entity = Entity::from(test_tools::simple_component_with_generics()?);
        assert_eq!(
            r#"entity test is
  generic (
    -- This is parameter documentation
    -- Next line.
    some_param : positive := 42;
    some_other_param : std_logic;
    some_other_param2 : integer := -42
  );
  port (
    -- This is port documentation
    -- Next line.
    some_port : in std_logic;
    some_other_port : out std_logic_vector(43 downto 0);
    clk : in std_logic
  );
end test;
"#,
            entity.declare(&db)?
        );

        Ok(())
    }
}
