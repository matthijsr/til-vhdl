use std::collections::BTreeMap;

use tydi_common::{
    error::{Error, Result, TryResult},
    name::{Name, NameSelf, PathName, PathNameSelf},
    traits::Identify,
};
use tydi_intern::Id;

use super::{
    physical_properties::InterfaceDirection, GetSelf, Implementation, Interface, InternSelf, Ir,
    MoveDb,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Streamlet {
    name: PathName,
    implementation: Option<Id<Implementation>>,
    ports: BTreeMap<Name, Id<Interface>>,
    port_order: Vec<Name>,
}

impl Streamlet {
    pub fn try_portless(name: impl TryResult<Name>) -> Result<Self> {
        Ok(Streamlet {
            name: name.try_result()?.into(),
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
            name: name.try_result()?.into(),
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
        let name = match &implementation {
            Some(some) => self.name.with_child(some.get(db).name().clone()),
            None => self.path_name().clone(),
        };
        db.intern_streamlet(Streamlet {
            name: name,
            implementation,
            ports: self.ports,
            port_order: self.port_order,
        })
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

impl PathNameSelf for Streamlet {
    fn path_name(&self) -> &PathName {
        &self.name
    }
}

impl Identify for Streamlet {
    fn identifier(&self) -> String {
        self.path_name().to_string()
    }
}

impl MoveDb<Id<Streamlet>> for Streamlet {
    fn move_db(&self, original_db: &dyn Ir, target_db: &dyn Ir) -> Result<Id<Streamlet>> {
        let port_order = self.port_order.clone();
        let ports = self
            .ports
            .iter()
            .map(|(k, v)| Ok((k.clone(), v.move_db(original_db, target_db)?)))
            .collect::<Result<_>>()?;
        let implementation = match &self.implementation {
            Some(id) => Some(id.move_db(original_db, target_db)?),
            None => None,
        };
        Ok(Streamlet {
            name: self.name.clone(),
            implementation,
            ports,
            port_order,
        }
        .intern(target_db))
    }
}

#[cfg(test)]
mod tests {
    use tydi_common::error::Result;

    use crate::ir::db::Database;

    use super::*;

    #[test]
    fn new() -> Result<()> {
        let db = Database::default();
        let imple = Implementation::link("link")?;
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
