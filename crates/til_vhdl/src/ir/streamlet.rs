use std::{
    collections::{BTreeMap, HashSet},
    convert::TryInto,
};

use tydi_common::{
    cat,
    error::{Error, Result, TryResult},
    traits::Identify,
};
use tydi_intern::Id;
use tydi_vhdl::{
    architecture::arch_storage::Arch, common::vhdl_name::VhdlName, component::Component, port::Port,
};

use super::{
    physical_properties::InterfaceDirection, GetSelf, Implementation, Interface, InternSelf,
    IntoVhdl, Ir, Name,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Streamlet {
    name: Name,
    implementation: Option<Id<Implementation>>,
    ports: BTreeMap<Name, Id<Interface>>,
    port_order: Vec<Name>,
}

impl Streamlet {
    pub fn try_portless(name: impl TryResult<Name>) -> Result<Self> {
        Ok(Streamlet {
            name: name.try_result()?,
            implementation: None,
            ports: BTreeMap::new(),
            port_order: vec![],
        })
    }

    pub fn try_new(
        db: &dyn Ir,
        name: impl TryResult<Name>,
        ports: Vec<impl TryResult<Interface>>,
    ) -> Result<Streamlet> {
        let mut port_order = vec![];
        let mut port_map = BTreeMap::new();
        for port in ports {
            let port = port.try_result()?;
            port_order.push(port.name().clone());
            if port_map.insert(port.name().clone(), port.intern(db)) != None {
                return Err(Error::UnexpectedDuplicate);
            }
        }

        Ok(Streamlet {
            name: name.try_result()?,
            implementation: None,
            ports: port_map,
            port_order,
        })
    }

    pub fn with_implementation(
        self,
        db: &dyn Ir,
        implementation: Option<Id<Implementation>>,
    ) -> Id<Streamlet> {
        db.intern_streamlet(Streamlet {
            name: self.name,
            implementation,
            ports: self.ports,
            port_order: self.port_order,
        })
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn implementation(&self, db: &dyn Ir) -> Option<Implementation> {
        if let Some(id) = self.implementation {
            Some(db.lookup_intern_implementation(id))
        } else {
            None
        }
    }

    pub fn port_ids(&self) -> &BTreeMap<Name, Id<Interface>> {
        &self.ports
    }

    pub fn ports(&self, db: &dyn Ir) -> Vec<Interface> {
        let mut result = vec![];
        for name in &self.port_order {
            result.push(self.port_ids().get(name).unwrap().get(db));
        }
        result
    }

    pub fn inputs(&self, db: &dyn Ir) -> Vec<Interface> {
        self.ports(db)
            .into_iter()
            .filter(|x| x.physical_properties().direction() == InterfaceDirection::In)
            .collect()
    }

    pub fn outputs(&self, db: &dyn Ir) -> Vec<Interface> {
        self.ports(db)
            .into_iter()
            .filter(|x| x.physical_properties().direction() == InterfaceDirection::Out)
            .collect()
    }

    pub fn try_get_port(&self, db: &dyn Ir, name: &Name) -> Result<Interface> {
        match self.port_ids().get(name) {
            Some(port) => Ok(port.get(db)),
            None => Err(Error::InvalidArgument(format!(
                "No port with name {} exists on Streamlet {}",
                name,
                self.identifier()
            ))),
        }
    }
}

impl Identify for Streamlet {
    fn identifier(&self) -> &str {
        self.name.as_ref()
    }
}

impl IntoVhdl<Component> for Streamlet {
    fn canonical(
        &self,
        ir_db: &dyn Ir,
        vhdl_db: &dyn Arch,
        prefix: impl Into<String>,
    ) -> Result<Component> {
        let mut ports = vec![];
        ports.push(Port::clk());
        ports.push(Port::rst());
        for input in self.inputs(ir_db) {
            ports.extend(input.canonical(ir_db, vhdl_db, "")?);
        }
        for output in self.outputs(ir_db) {
            ports.extend(output.canonical(ir_db, vhdl_db, "")?);
        }
        // TODO: Streamlet should also have documentation?

        Ok(Component::new(
            VhdlName::try_new(cat!(self.identifier(), "com"))?,
            vec![],
            ports,
            None,
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::ir::{Database, Stream};
    use tydi_common::error::{Error, Result};

    use super::*;

    #[test]
    fn new() -> Result<()> {
        let db = Database::default();
        let imple = Implementation::Link;
        let implid = db.intern_implementation(imple.clone());
        let streamlet = Streamlet::try_portless("test")?.with_implementation(&db, Some(implid));
        assert_eq!(
            db.lookup_intern_streamlet(streamlet)
                .implementation(&db)
                .unwrap(),
            imple
        );
        Ok(())
    }
}
