use std::convert::TryFrom;

use tydi_common::{
    cat,
    error::{Error, Result, TryResult, TryOptional},
    traits::Identify,
};
use tydi_intern::Id;
use tydi_vhdl::{
    architecture::arch_storage::Arch,
    common::vhdl_name::VhdlName,
    port::{Mode, Port},
};

use crate::common::logical::logical_stream::SynthesizeLogicalStream;

use super::{
    physical_properties::InterfaceDirection, IntoVhdl, Ir, Name, PhysicalProperties, Stream,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Interface {
    name: Name,
    stream: Id<Stream>,
    physical_properties: PhysicalProperties,
}

impl Interface {
    pub fn try_new(
        name: impl TryResult<Name>,
        stream: Id<Stream>,
        physical_properties: PhysicalProperties,
    ) -> Result<Interface> {
        Ok(Interface {
            name: name.try_result()?,
            stream,
            physical_properties,
        })
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn stream(&self, db: &dyn Ir) -> Stream {
        db.lookup_intern_stream(self.stream)
    }

    pub fn stream_id(&self) -> Id<Stream> {
        self.stream
    }

    pub fn physical_properties(&self) -> &PhysicalProperties {
        &self.physical_properties
    }

    pub fn direction(&self) -> InterfaceDirection {
        self.physical_properties().direction()
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
        _arch_db: &mut dyn Arch,
        prefix: impl TryOptional<Name>,
    ) -> Result<Vec<Port>> {
        let n: String = match prefix.try_optional()? {
            Some(some) => cat!(some, self.identifier()),
            None => self.identifier().to_string(),
        };
        let mut ports = Vec::new();

        let synth = self.stream.synthesize(ir_db);

        for (path, width) in synth.signals() {
            ports.push(Port::new(
                VhdlName::try_new(cat!(n.clone(), path.to_string()))?,
                match self.physical_properties().direction() {
                    InterfaceDirection::Out => Mode::Out,
                    InterfaceDirection::In => Mode::In,
                },
                width.clone().into(),
            ));
        }

        for (path, phys) in synth.streams() {
            ports.extend(phys.into_vhdl(&n, path, self.physical_properties().direction())?);
        }

        Ok(ports)
    }
}

impl<N, P> TryFrom<(N, Id<Stream>, P)> for Interface
where
    N: TryResult<Name>,
    P: TryResult<PhysicalProperties>,
{
    type Error = Error;

    fn try_from((name, stream, physical_properties): (N, Id<Stream>, P)) -> Result<Self> {
        Ok(Interface {
            name: name.try_result()?,
            stream,
            physical_properties: physical_properties.try_result()?,
        })
    }
}
