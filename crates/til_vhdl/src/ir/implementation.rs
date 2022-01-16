use salsa::InternKey;
use tydi_common::error::Result;

use tydi_intern::Id;
use tydi_vhdl::{
    architecture::{self, arch_storage::Arch, Architecture},
    declaration::{DeclareWithIndent, Declare},
};

use super::{context::Context, AnnotationKey, IntoVhdl, Ir, Streamlet};

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
        prefix: impl Into<String>,
    ) -> Result<String> {
        match self {
            Implementation::Structural(context) => {
                let prefix = prefix.into();

                let arch_body = context.canonical(ir_db, arch_db, &prefix)?;
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
        prefix: impl Into<String>,
    ) -> Result<String> {
        match self {
            Some(implementation) => match implementation {
                Implementation::Structural(context) => {
                    let prefix = prefix.into();

                    let arch_body = context.canonical(ir_db, arch_db, &prefix)?;
                    let mut architecture = Architecture::from_database(arch_db, "Behavioral")?;
                    architecture.add_body(arch_db, &arch_body)?;

                    let result_string = architecture.declare(arch_db)?;
                    arch_db.set_architecture(architecture);

                    Ok(result_string)
                }
                Implementation::Link => todo!(),
            },
            None => {
                let prefix = prefix.into();

                let architecture = Architecture::from_database(arch_db, "Behavioral")?;

                let result_string = architecture.declare_with_indent(arch_db, &prefix)?;
                arch_db.set_architecture(architecture);

                Ok(result_string)
            }
        }
    }
}
