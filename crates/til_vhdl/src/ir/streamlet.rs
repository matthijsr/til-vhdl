use std::collections::HashSet;

use tydi_common::{
    cat,
    error::{Error, Result},
    traits::Identify,
};
use tydi_intern::Id;
use tydi_vhdl::{
    architecture::arch_storage::Arch, common::vhdl_name::VhdlName, component::Component, port::Port,
};

use super::{
    physical_properties::InterfaceDirection, GetSelf, Implementation, Interface, InternSelf, IntoVhdl, Ir, Name,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Streamlet {
    name: Name,
    implementation: Option<Id<Implementation>>,
    ports: Vec<Id<Interface>>,
}

impl Streamlet {
    pub fn try_new(db: &dyn Ir, name: impl Into<Name>, ports: Vec<Interface>) -> Result<Streamlet> {
        let mut set = HashSet::new();
        for port in &ports {
            if !set.insert(port.identifier()) {
                return Err(Error::UnexpectedDuplicate);
            }
        }
        let ports = ports.into_iter().map(|x| x.intern(db)).collect();
        Ok(Streamlet {
            name: name.into(),
            implementation: None,
            ports,
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

    pub fn ports(&self, db: &dyn Ir) -> Vec<Interface> {
        let mut result = vec![];
        for port in &self.ports {
            result.push(port.get(db))
        }
        result
    }

    pub fn inputs(&self, db: &dyn Ir) -> Vec<Interface> {
        self.ports(db)
            .into_iter()
            .filter(|x| x.physical_properties().origin() == InterfaceDirection::In)
            .collect()
    }

    pub fn outputs(&self, db: &dyn Ir) -> Vec<Interface> {
        self.ports(db)
            .into_iter()
            .filter(|x| x.physical_properties().origin() == InterfaceDirection::Out)
            .collect()
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
        let streamlet = Streamlet::try_new(&db, Name::try_new("test")?, vec![])?
            .with_implementation(&db, Some(implid));
        assert_eq!(
            db.lookup_intern_streamlet(streamlet)
                .implementation(&db)
                .unwrap(),
            imple
        );
        Ok(())
    }
}
