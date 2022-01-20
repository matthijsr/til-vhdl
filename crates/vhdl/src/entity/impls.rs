use textwrap::indent;
use tydi_common::error::Result;
use tydi_common::traits::{Document, Identify};

use crate::architecture::arch_storage::Arch;
use crate::declaration::DeclareWithIndent;
use crate::traits::VhdlDocument;
use crate::{
    component::Component,
    declaration::Declare,
    port::{Parameter, Port},
};

use super::Entity;

impl DeclareWithIndent for Entity {
    fn declare_with_indent(&self, db: &dyn Arch, pre: &str) -> Result<String> {
        let mut result = String::new();
        if let Some(doc) = self.vhdl_doc() {
            result.push_str(&doc);
        }
        result.push_str(format!("entity {} is\n", self.identifier()).as_str());
        let mut port_body = "port (\n".to_string();
        let ports = self
            .ports()
            .iter()
            .map(|x| x.declare(db))
            .collect::<Result<Vec<String>>>()?
            .join(";\n");
        port_body.push_str(&indent(&ports, pre));
        port_body.push_str("\n);\n");
        result.push_str(&indent(&port_body, pre));
        result.push_str(format!("end {};\n", self.identifier()).as_str());
        Ok(result)
    }
}

impl Identify for Entity {
    fn identifier(&self) -> String {
        self.identifier.clone()
    }
}

impl Document for Entity {
    fn doc(&self) -> Option<String> {
        self.doc.clone()
    }
}

impl Entity {
    /// Create a new entity.
    pub fn new(
        identifier: impl Into<String>,
        parameters: Vec<Parameter>,
        ports: Vec<Port>,
        doc: Option<String>,
    ) -> Entity {
        Entity {
            identifier: identifier.into(),
            parameters,
            ports,
            doc,
        }
    }

    /// Return a reference to the ports of this entity.
    pub fn ports(&self) -> &Vec<Port> {
        &self.ports
    }

    /// Return a reference to the parameters of this entity.
    pub fn parameters(&self) -> &Vec<Parameter> {
        &self.parameters
    }

    /// Return this entity with documentation added.
    pub fn with_doc(mut self, doc: impl Into<String>) -> Self {
        self.doc = Some(doc.into());
        self
    }

    /// Set the documentation of this entity.
    pub fn set_doc(&mut self, doc: impl Into<String>) {
        self.doc = Some(doc.into())
    }
}

impl From<&Component> for Entity {
    fn from(comp: &Component) -> Self {
        Entity::new(
            comp.identifier(),
            comp.parameters().to_vec(),
            comp.ports().to_vec(),
            comp.doc(),
        )
    }
}

impl From<Component> for Entity {
    fn from(comp: Component) -> Self {
        Entity::from(&comp)
    }
}

#[cfg(test)]
mod tests {
    // TODO

    //     use crate::generator::common::test::test_comp;
    //     use crate::generator::vhdl::Declare;
    //     use crate::stdlib::common::entity::*;

    //     #[test]
    //     fn entity_declare() {
    //         let c = Entity::from(test_comp()).with_doc(" My awesome\n Entity".to_string());
    //         assert_eq!(
    //             c.declare().unwrap(),
    //             concat!(
    //                 "-- My awesome
    // -- Entity
    // entity test_comp is
    //   port(
    //     a_dn : in a_dn_type;
    //     a_up : out a_up_type;
    //     b_dn : out b_dn_type;
    //     b_up : in b_up_type
    //   );
    // end test_comp;
    // "
    //             )
    //         );
    //     }
}
