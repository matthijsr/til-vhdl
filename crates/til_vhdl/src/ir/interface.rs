use std::convert::TryInto;

use tydi_common::{
    cat,
    error::{Error, Result},
    traits::{Identify, Reversed},
};
use tydi_intern::Id;
use tydi_vhdl::{
    architecture::arch_storage::Arch,
    common::vhdl_name::VhdlName,
    port::{Mode, Port},
};

use crate::common::logical::logicaltype::Direction;

use super::{
    physical_properties::{self, Origin},
    IntoVhdl, Ir, LogicalType, Name, PhysicalProperties, Stream,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Interface {
    name: Name,
    stream: Id<Stream>,
    physical_properties: PhysicalProperties,
}

impl Interface {
    pub fn try_new(
        name: impl TryInto<Name, Error = Error>,
        stream: Id<Stream>,
        physical_properties: PhysicalProperties,
    ) -> Result<Interface> {
        Ok(Interface {
            name: name.try_into()?,
            stream,
            physical_properties,
        })
    }

    pub fn stream(&self, db: &dyn Ir) -> Stream {
        db.lookup_intern_stream(self.stream)
    }

    pub fn physical_properties(&self) -> &PhysicalProperties {
        &self.physical_properties
    }
}

impl Identify for Interface {
    fn identifier(&self) -> &str {
        self.name.as_ref()
    }
}

impl IntoVhdl<Vec<Port>> for Interface {
    fn canonical(
        &self,
        ir_db: &dyn Ir,
        vhdl_db: &dyn Arch,
        prefix: impl Into<String>,
    ) -> Result<Vec<Port>> {
        let n: String = prefix.into();
        let mut ports = Vec::new();

        let synth = self.stream(ir_db).synthesize(ir_db);

        for (path, width) in synth.signals() {
            ports.push(Port::new(
                VhdlName::try_new(cat!(n.clone(), path.to_string()))?,
                match self.physical_properties().origin() {
                    Origin::Source => Mode::Out,
                    Origin::Sink => Mode::In,
                },
                width.clone().into(),
            ));
        }

        for (path, phys) in synth.streams() {
            ports.extend(phys.into_vhdl(&n, path, self.physical_properties().origin())?);
        }

        Ok(ports)
    }
}
