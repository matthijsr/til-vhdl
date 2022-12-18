use til_query::{
    common::logical::logical_stream::{LogicalStream, SynthesizeLogicalStream, TypedStream},
    ir::{
        physical_properties::{InterfaceDirection, PhysicalProperties},
        Ir,
    },
};
use tydi_common::{
    cat,
    error::{Result, TryOptional},
    map::InsertionOrderedMap,
    name::{Name, NameSelf},
    traits::{Document, Identify},
};

use tydi_intern::Id;
use tydi_vhdl::{
    architecture::arch_storage::Arch,
    common::vhdl_name::VhdlName,
    declaration::ObjectDeclaration,
    port::{Mode, Port},
};

use crate::{
    common::physical::stream::{physical_stream_to_vhdl, VhdlPhysicalStream},
    IntoVhdl,
};

pub(crate) type InterfacePort = til_query::ir::interface_port::InterfacePort;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// The VHDL representation of a Tydi interface, consisting of physical streams
/// which are themselves made of ports.
pub struct VhdlInterface {
    name: Name,
    typed_stream: TypedStream<Port, VhdlPhysicalStream>,
    physical_properties: PhysicalProperties,
    doc: Option<String>,
}

impl VhdlInterface {
    pub fn typed_stream(&self) -> &TypedStream<Port, VhdlPhysicalStream> {
        &self.typed_stream
    }
    pub fn physical_properties(&self) -> &PhysicalProperties {
        &self.physical_properties
    }
}

impl Identify for VhdlInterface {
    fn identifier(&self) -> String {
        self.name.to_string()
    }
}

impl NameSelf for VhdlInterface {
    fn name(&self) -> &Name {
        &self.name
    }
}

impl Document for VhdlInterface {
    fn doc(&self) -> Option<&String> {
        self.doc.as_ref()
    }
}

pub fn interface_port_to_vhdl(
    ir_db: &dyn Ir,
    arch_db: &mut dyn Arch,
    interface_port: &InterfacePort,
    prefix: impl TryOptional<VhdlName>,
    parent_params: &InsertionOrderedMap<Name, Id<ObjectDeclaration>>,
) -> Result<VhdlInterface> {
    let n: VhdlName = match prefix.try_optional()? {
        Some(some) => VhdlName::try_new(cat!(some, interface_port.identifier()))?,
        None => interface_port.name().clone().into(),
    };

    let synth = interface_port.stream_id().synthesize(ir_db)?;

    let mut fields = InsertionOrderedMap::new();
    for (path, width) in synth.logical_stream().fields_iter() {
        let prefixed_path = format!("{}__{}", &n, path);
        fields.try_insert(
            path.clone(),
            Port::try_new(
                prefixed_path,
                match interface_port.physical_properties().direction() {
                    InterfaceDirection::Out => Mode::Out,
                    InterfaceDirection::In => Mode::In,
                },
                width.clone(),
            )?,
        )?;
    }

    let mut streams = InsertionOrderedMap::new();
    for (path, phys) in synth.logical_stream().streams_iter() {
        let phys_name = if path.len() > 0 {
            format!("{}__{}", &n, path)
        } else {
            n.to_string()
        };
        streams.try_insert(
            path.clone(),
            physical_stream_to_vhdl(arch_db, phys, phys_name.as_str(), parent_params)?
                .with_interface_direction(interface_port.physical_properties().direction()),
        )?;
    }

    let typed_stream = TypedStream::new(
        LogicalStream::new(fields, streams),
        synth.type_reference().clone(),
    );

    Ok(VhdlInterface {
        name: interface_port.name().clone(),
        typed_stream,
        physical_properties: interface_port.physical_properties().clone(),
        doc: interface_port.doc().cloned(),
    })
}

// impl IntoVhdl<VhdlInterface> for InterfacePort {
//     fn canonical(
//         &self,
//         ir_db: &dyn Ir,
//         arch_db: &mut dyn Arch,
//         prefix: impl TryOptional<VhdlName>,
//     ) -> Result<VhdlInterface> {
//         let n: VhdlName = match prefix.try_optional()? {
//             Some(some) => VhdlName::try_new(cat!(some, self.identifier()))?,
//             None => self.name().clone().into(),
//         };

//         let synth = self.stream_id().synthesize(ir_db)?;

//         let mut fields = InsertionOrderedMap::new();
//         for (path, width) in synth.logical_stream().fields_iter() {
//             let prefixed_path = format!("{}__{}", &n, path);
//             fields.try_insert(
//                 path.clone(),
//                 Port::try_new(
//                     prefixed_path,
//                     match self.physical_properties().direction() {
//                         InterfaceDirection::Out => Mode::Out,
//                         InterfaceDirection::In => Mode::In,
//                     },
//                     width.clone(),
//                 )?,
//             )?;
//         }

//         let mut streams = InsertionOrderedMap::new();
//         for (path, phys) in synth.logical_stream().streams_iter() {
//             let phys_name = if path.len() > 0 {
//                 format!("{}__{}", &n, path)
//             } else {
//                 n.to_string()
//             };
//             streams.try_insert(
//                 path.clone(),
//                 phys.canonical(ir_db, arch_db, phys_name.as_str())?
//                     .with_interface_direction(self.physical_properties().direction()),
//             )?;
//         }

//         let typed_stream = TypedStream::new(
//             LogicalStream::new(fields, streams),
//             synth.type_reference().clone(),
//         );

//         Ok(VhdlInterface {
//             name: self.name().clone(),
//             typed_stream,
//             physical_properties: self.physical_properties().clone(),
//             doc: self.doc().cloned(),
//         })
//     }
// }
