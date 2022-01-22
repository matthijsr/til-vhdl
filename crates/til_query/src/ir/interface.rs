use std::convert::TryFrom;

use tydi_common::{
    cat,
    error::{Error, Result, TryOptional, TryResult},
    traits::Identify,
};
use tydi_intern::Id;

use crate::common::logical::logical_stream::SynthesizeLogicalStream;

use super::{physical_properties::InterfaceDirection, Ir, Name, PhysicalProperties, Stream};

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
    fn identifier(&self) -> String {
        self.name.to_string()
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
