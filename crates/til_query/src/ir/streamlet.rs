use std::collections::BTreeMap;

use tydi_common::{
    cat,
    error::{Error, Result, TryOptional, TryResult},
    name::{NameSelf, PathName, PathNameSelf},
    traits::Identify,
};
use tydi_intern::Id;

use super::{
    implementation::ImplementationKind, physical_properties::InterfaceDirection, GetSelf,
    Implementation, Interface, InternSelf, IntoVhdl, Ir, Name,
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

impl IntoVhdl<Component> for Streamlet {
    fn canonical(
        &self,
        ir_db: &dyn Ir,
        arch_db: &mut dyn Arch,
        prefix: impl TryOptional<Name>,
    ) -> Result<Component> {
        let prefix = prefix.try_optional()?;
        let n: String = match &prefix {
            Some(some) => cat!(some, self.identifier(), "com"),
            None => cat!(self.identifier(), "com"),
        };

        let mut ports = vec![];
        ports.push(Port::clk());
        ports.push(Port::rst());
        for input in self.inputs(ir_db) {
            ports.extend(input.canonical(ir_db, arch_db, prefix.clone())?);
        }
        for output in self.outputs(ir_db) {
            ports.extend(output.canonical(ir_db, arch_db, prefix.clone())?);
        }
        // TODO: Streamlet should also have documentation?

        Ok(Component::new(VhdlName::try_new(n)?, vec![], ports, None))
    }
}

// TODO: For now, assume architecture output will be a string.
// The architecture for Structural and None is stored in the arch_db.
// Might make more sense/be safer if we could either parse Linked architectures to an object,
// or have some enclosing type which returns either an architecture or a string.
impl IntoVhdl<String> for Streamlet {
    fn canonical(
        &self,
        ir_db: &dyn Ir,
        arch_db: &mut dyn Arch,
        prefix: impl TryOptional<Name>,
    ) -> Result<String> {
        let prefix = prefix.try_optional()?;
        let component: Component = self.canonical(ir_db, arch_db, prefix.clone())?;
        arch_db.set_subject_component_name(component.vhdl_name().clone());

        let mut package = arch_db.default_package();
        package.add_component(component);
        arch_db.set_default_package(package);

        match self.implementation(ir_db) {
            Some(implementation) => match implementation.kind() {
                ImplementationKind::Structural(structure) => {
                    let arch_body = structure.canonical(ir_db, arch_db, prefix)?;
                    let mut architecture = Architecture::from_database(arch_db, "Behavioral")?;
                    architecture.add_body(arch_db, &arch_body)?;

                    let result_string = architecture.declare(arch_db)?;
                    arch_db.set_architecture(architecture);

                    Ok(result_string)
                }
                ImplementationKind::Link => todo!(),
            },
            None => {
                let architecture = Architecture::from_database(arch_db, "Behavioral")?;

                let result_string = architecture.declare(arch_db)?;
                arch_db.set_architecture(architecture);

                Ok(result_string)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ir::Database;
    use tydi_common::error::Result;

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
