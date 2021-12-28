use std::collections::HashSet;

use tydi_common::{
    error::{Error, Result},
    traits::Identify,
};
use tydi_intern::Id;
use tydi_vhdl::{architecture::arch_storage::Arch, component::Component};

use super::{physical_properties::Origin, GetSelf, Implementation, InternSelf, IntoVhdl, Ir, Port};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Streamlet {
    implementation: Option<Id<Implementation>>,
    ports: Vec<Id<Port>>,
}

impl Streamlet {
    pub fn try_new(db: &dyn Ir, ports: Vec<Port>) -> Result<Streamlet> {
        let mut set = HashSet::new();
        for port in &ports {
            if !set.insert(port.identifier()) {
                return Err(Error::UnexpectedDuplicate);
            }
        }
        let ports = ports.into_iter().map(|x| x.intern(db)).collect();
        Ok(Streamlet {
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
            implementation,
            ports: self.ports,
        })
    }

    pub fn implementation(&self, db: &dyn Ir) -> Option<Implementation> {
        if let Some(id) = self.implementation {
            Some(db.lookup_intern_implementation(id))
        } else {
            None
        }
    }

    pub fn ports(&self, db: &dyn Ir) -> Vec<Port> {
        let mut result = vec![];
        for port in &self.ports {
            result.push(port.get(db))
        }
        result
    }

    pub fn inputs(&self, db: &dyn Ir) -> Vec<Port> {
        self.ports(db)
            .into_iter()
            .filter(|x| x.physical_properties().origin() == Origin::Sink)
            .collect()
    }

    pub fn outputs(&self, db: &dyn Ir) -> Vec<Port> {
        self.ports(db)
            .into_iter()
            .filter(|x| x.physical_properties().origin() == Origin::Source)
            .collect()
    }
}

impl IntoVhdl<Component> for Streamlet {
    fn into_vhdl(&self, ir_db: &dyn Ir, vhdl_db: &dyn Arch) -> Component {
        todo!()
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
        let streamlet = Streamlet::try_new(&db, vec![])?.with_implementation(&db, Some(implid));
        assert_eq!(
            db.lookup_intern_streamlet(streamlet)
                .implementation(&db)
                .unwrap(),
            imple
        );
        Ok(())
    }
}
