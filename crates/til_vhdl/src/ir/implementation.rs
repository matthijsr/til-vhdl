use tydi_common::{
    error::{Result, TryOptional, TryResult},
    name::{Name, NameSelf},
};

use tydi_vhdl::{
    architecture::{arch_storage::Arch, Architecture},
    declaration::Declare,
};

use super::{context::Context, IntoVhdl, Ir};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Implementation {
    name: Name,
    kind: ImplementationKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ImplementationKind {
    Structural(Context),
    Link,
}

impl Implementation {
    pub fn structural(
        name: impl TryResult<Name>,
        context: impl TryResult<Context>,
    ) -> Result<Self> {
        Ok(Implementation {
            name: name.try_result()?,
            kind: ImplementationKind::Structural(context.try_result()?),
        })
    }

    /// TODO
    pub fn link(name: impl TryResult<Name>) -> Result<Self> {
        Ok(Implementation {
            name: name.try_result()?,
            kind: ImplementationKind::Link,
        })
    }

    pub fn kind(&self) -> &ImplementationKind {
        &self.kind
    }

    pub fn resolve_for(
        &self,
        ir_db: &dyn Ir,
        arch_db: &mut dyn Arch,
        prefix: impl TryOptional<Name>,
    ) -> Result<String> {
        match self.kind() {
            ImplementationKind::Structural(context) => {
                let arch_body = context.canonical(ir_db, arch_db, prefix)?;
                let mut architecture = Architecture::from_database(arch_db, "Behavioral")?;
                architecture.add_body(arch_db, &arch_body)?;

                let result_string = architecture.declare(arch_db)?;
                arch_db.set_architecture(architecture);

                Ok(result_string)
            }
            ImplementationKind::Link => todo!(),
        }
    }
}

impl NameSelf for Implementation {
    fn name(&self) -> &Name {
        &self.name
    }
}

impl IntoVhdl<String> for Option<Implementation> {
    fn canonical(
        &self,
        ir_db: &dyn Ir,
        arch_db: &mut dyn Arch,
        prefix: impl TryOptional<Name>,
    ) -> Result<String> {
        match self {
            Some(implementation) => match implementation.kind() {
                ImplementationKind::Structural(context) => {
                    let arch_body = context.canonical(ir_db, arch_db, prefix)?;
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
