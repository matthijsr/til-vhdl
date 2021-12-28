use std::convert::TryInto;

use tydi_common::{
    error::{Error, Result},
    traits::Identify,
};
use tydi_intern::Id;
use tydi_vhdl::{architecture::arch_storage::Arch, port::Mode};

use crate::common::logical::logicaltype::Direction;

use super::{
    physical_properties::{self, Origin},
    IntoVhdl, Ir, LogicalType, Name, PhysicalProperties, Stream,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Port {
    name: Name,
    stream: Id<Stream>,
    physical_properties: PhysicalProperties,
}

impl Port {
    pub fn try_new(
        name: impl TryInto<Name, Error = Error>,
        stream: Id<Stream>,
        physical_properties: PhysicalProperties,
    ) -> Result<Port> {
        Ok(Port {
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

impl Identify for Port {
    fn identifier(&self) -> &str {
        self.name.as_ref()
    }
}
// // TODO
// impl IntoVhdl<Vec<tydi_vhdl::port::Port>> for Port {
//     fn into_vhdl(&self, ir_db: &dyn Ir, vhdl_db: &dyn Arch) -> Vec<tydi_vhdl::port::Port> {
//         let stream = self.stream(ir_db);
//         let default_mode = match self.physical_properties().origin() {
//             Origin::Source => Mode::Out,
//             Origin::Sink => Mode::In,
//         };
//         let result_mode = |x: Mode, d: Direction| match d {
//             Direction::Forward => x,
//             Direction::Reverse => x.reverse(),
//         };
//         let mut result = vec![];
//         match stream.data(ir_db) {
//             LogicalType::Null => (),
//             LogicalType::Bits(n) => tydi_vhdl::port::Port::new(),
//             LogicalType::Group(_) => todo!(),
//             LogicalType::Union(_) => todo!(),
//             LogicalType::Stream(_) => todo!(),
//         }

//         result
//     }
// }
