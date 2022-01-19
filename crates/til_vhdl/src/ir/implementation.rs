use tydi_common::{
    error::{Result, TryResult, TryOptional},
    name::Name,
};

use tydi_vhdl::{
    architecture::{arch_storage::Arch, Architecture},
    declaration::Declare,
};

use super::{context::Context, IntoVhdl, Ir};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Implementation {
    Structural(Context),
    Link,
}

impl Implementation {
    pub fn resolve_for(
        &self,
        ir_db: &dyn Ir,
        arch_db: &mut dyn Arch,
        prefix: impl TryOptional<Name>,
    ) -> Result<String> {
        match self {
            Implementation::Structural(context) => {
                let arch_body = context.canonical(ir_db, arch_db, prefix)?;
                let mut architecture = Architecture::from_database(arch_db, "Behavioral")?;
                architecture.add_body(arch_db, &arch_body)?;

                let result_string = architecture.declare(arch_db)?;
                arch_db.set_architecture(architecture);

                Ok(result_string)
            }
            Implementation::Link => todo!(),
        }
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
            Some(implementation) => match implementation {
                Implementation::Structural(context) => {
                    let arch_body = context.canonical(ir_db, arch_db, prefix)?;
                    let mut architecture = Architecture::from_database(arch_db, "Behavioral")?;
                    architecture.add_body(arch_db, &arch_body)?;

                    let result_string = architecture.declare(arch_db)?;
                    arch_db.set_architecture(architecture);

                    Ok(result_string)
                }
                Implementation::Link => todo!(),
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
