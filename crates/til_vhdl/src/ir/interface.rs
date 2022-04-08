use indexmap::IndexMap;
use til_query::{
    common::{
        logical::{
            logical_stream::{LogicalStream, SynthesizeLogicalStream, TypedStream},
            type_reference::TypeReference,
        },
        physical::{fields::Fields, signal_list::SignalList},
    },
    ir::{
        physical_properties::{InterfaceDirection, PhysicalProperties},
        Ir,
    },
};
use tydi_common::{
    cat,
    error::{Result, TryOptional},
    map::InsertionOrderedMap,
    name::{Name, PathName},
    traits::{Document, Identify},
};

use tydi_vhdl::{
    architecture::arch_storage::Arch,
    common::vhdl_name::VhdlName,
    port::{Mode, Port},
};

use crate::{common::physical::stream::VhdlPhysicalStream, IntoVhdl};

pub(crate) type Interface = til_query::ir::interface::Interface;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// The VHDL representation of a Tydi interface, consisting of physical streams
/// which are themselves made of ports.
pub struct VhdlInterface {
    name: Name,
    physical_streams: InsertionOrderedMap<PathName, VhdlPhysicalStream>,
    type_reference: TypeReference,
    physical_properties: PhysicalProperties,
    doc: Option<String>,
}

impl VhdlInterface {}

impl IntoVhdl<TypedStream<Port, SignalList<Port>>> for Interface {
    fn canonical(
        &self,
        ir_db: &dyn Ir,
        arch_db: &mut dyn Arch,
        prefix: impl TryOptional<VhdlName>,
    ) -> Result<TypedStream<Port, SignalList<Port>>> {
        let n: VhdlName = match prefix.try_optional()? {
            Some(some) => VhdlName::try_new(cat!(some, self.identifier()))?,
            None => self.name().clone().into(),
        };

        let synth = self.stream_id().synthesize(ir_db)?;

        let fields = Fields::new(synth.logical_stream().fields_iter().map(|(path, width)| {
            let prefixed_path = format!("{}__{}", &n, path);
            (
                path.clone(),
                Port::try_new(
                    prefixed_path,
                    match self.physical_properties().direction() {
                        InterfaceDirection::Out => Mode::Out,
                        InterfaceDirection::In => Mode::In,
                    },
                    width.clone(),
                )
                .unwrap(),
            )
        }))?;

        let mut first = true;
        let mut streams = IndexMap::new();
        for (path, phys) in synth.logical_stream().streams_iter() {
            let phys_name = if path.len() > 0 {
                format!("{}__{}", &n, path)
            } else {
                n.to_string()
            };
            let mut signal_list = phys
                .canonical(ir_db, arch_db, phys_name.as_str())?
                .with_direction(self.physical_properties().direction())
                .signal_list()
                .clone();
            if first && (&signal_list).into_iter().len() > 0 {
                if let Some(doc) = self.doc() {
                    signal_list.apply_first(|p| p.set_doc(doc.clone()));
                }
                first = false;
            }
            streams.insert(path.clone(), signal_list.clone());
        }

        Ok(TypedStream::new(
            LogicalStream::new(fields, streams),
            synth.type_reference().clone(),
        ))
    }
}
