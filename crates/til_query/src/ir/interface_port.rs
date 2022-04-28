use std::convert::TryFrom;

use tydi_common::{
    error::{Error, Result, TryResult},
    name::Name,
    traits::{Document, Documents, Identify},
};
use tydi_intern::Id;

use crate::common::logical::logicaltype::stream::Stream;

use super::{
    physical_properties::{InterfaceDirection, PhysicalProperties},
    traits::{InternSelf, MoveDb},
    Ir,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InterfacePort {
    name: Name,
    stream: Id<Stream>,
    physical_properties: PhysicalProperties,
    /// Documentation.
    doc: Option<String>,
}

impl InterfacePort {
    pub fn try_new(
        name: impl TryResult<Name>,
        stream: Id<Stream>,
        physical_properties: PhysicalProperties,
    ) -> Result<InterfacePort> {
        Ok(InterfacePort {
            name: name.try_result()?,
            stream,
            physical_properties,
            doc: None,
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

    pub fn domain(&self) -> Option<&Name> {
        self.physical_properties().domain()
    }

    pub fn set_domain(&mut self, domain: Name) {
        self.physical_properties.set_domain(domain)
    }
}

impl Identify for InterfacePort {
    fn identifier(&self) -> String {
        self.name.to_string()
    }
}

impl<N, S, P> TryFrom<(N, S, P)> for InterfacePort
where
    N: TryResult<Name>,
    S: TryResult<Id<Stream>>,
    P: TryResult<PhysicalProperties>,
{
    type Error = Error;

    fn try_from((name, stream, physical_properties): (N, S, P)) -> Result<Self> {
        Ok(InterfacePort {
            name: name.try_result()?,
            stream: stream.try_result()?,
            physical_properties: physical_properties.try_result()?,
            doc: None,
        })
    }
}

impl MoveDb<Id<InterfacePort>> for InterfacePort {
    fn move_db(
        &self,
        original_db: &dyn Ir,
        target_db: &dyn Ir,
        prefix: &Option<Name>,
    ) -> Result<Id<InterfacePort>> {
        Ok(InterfacePort {
            name: self.name.clone(),
            stream: self.stream.move_db(original_db, target_db, prefix)?,
            physical_properties: self.physical_properties.clone(),
            doc: self.doc.clone(),
        }
        .intern(target_db))
    }
}

impl Document for InterfacePort {
    fn doc(&self) -> Option<&String> {
        self.doc.as_ref()
    }
}

impl Documents for InterfacePort {
    fn set_doc(&mut self, doc: impl Into<String>) {
        self.doc = Some(doc.into());
    }
}
