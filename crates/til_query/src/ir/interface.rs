use std::convert::TryFrom;

use tydi_common::{
    error::{Error, Result, TryResult},
    name::Name,
    traits::Identify,
};
use tydi_intern::Id;

use crate::common::logical::logicaltype::stream::Stream;

use super::{
    physical_properties::{InterfaceDirection, PhysicalProperties},
    InternSelf, Ir, MoveDb,
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

impl MoveDb<Id<Interface>> for Interface {
    fn move_db(&self, original_db: &dyn Ir, target_db: &dyn Ir) -> Result<Id<Interface>> {
        Ok(Interface {
            name: self.name.clone(),
            stream: self.stream.move_db(original_db, target_db)?,
            physical_properties: self.physical_properties.clone(),
        }
        .intern(target_db))
    }
}
