use std::sync::Arc;

use bitvec::prelude::*;

use til_parser::query::into_query_storage;
use til_query::{
    common::{
        signals::PhysicalSignals,
        transfer::{
            element_type::ElementType,
            physical_transfer::{LastMode, StrobeMode},
        },
    },
    ir::{connection::InterfaceReference, traits::GetSelf, Ir},
};
use til_vhdl::{
    common::signals::PhysicalStreamProcess, ir::streamlet::StreamletArchitecture, IntoVhdl,
};
use tydi_common::{
    error::Result,
    name::{Name, PathName},
};
use tydi_vhdl::{
    architecture::arch_storage::Arch,
    common::vhdl_name::VhdlNameSelf,
    declaration::{Declare, ObjectDeclaration},
    package::Package,
};

#[test]
fn process_playground() -> Result<()> {
    let db = into_query_storage(
        "
namespace my::test::space {
    type stream1 = Stream(
        data: Bits(8),
        dimensionality: 3,
        throughput: 3,
        synchronicity: Sync,
        complexity: 8,
        direction: Forward,
        user: Bits(2),
    );

    #\
    streamlet documentation \
    is multi-line but can act as a split string\
    #
    streamlet doc_streamlet = (
      #interface documentation
is also
multiline#
      x: in stream1
    );
}
    ",
    )?;

    let proj = db.project();
    let streamlet = proj
        .namespaces()
        .get(&("my::test::space".try_into()?))
        .unwrap()
        .get(&db)
        .get_streamlet(&db, "doc_streamlet")?;

    let mut arch_db = tydi_vhdl::architecture::arch_storage::db::Database::default();
    let mut package = Package::new_default_empty();

    let mut vhdl_streamlet = streamlet.canonical(&db, &mut arch_db, None)?;
    let component = vhdl_streamlet.to_component();

    arch_db.set_subject_component_name(Arc::new(component.vhdl_name().clone()));
    package.add_component(component);
    arch_db.set_default_package(Arc::new(package));

    let mut arch = vhdl_streamlet.to_architecture(&db, &mut arch_db)?;
    if let StreamletArchitecture::Generated(arch) = &mut arch {
        let clk = ObjectDeclaration::entity_clk(&arch_db);
        let rst = ObjectDeclaration::entity_rst(&arch_db);
        let instance =
            vhdl_streamlet.to_instance(&mut arch_db, Name::try_new("a")?, arch, clk, rst)?;
        let iref = InterfaceReference::try_from(("a", "x"))?;
        let port_obj = instance.get(&iref).unwrap();
        let stream_obj = port_obj
            .typed_stream()
            .logical_stream()
            .streams()
            .get(&PathName::new_empty())
            .unwrap();
        let stream_proc = PhysicalStreamProcess::from(stream_obj.clone());
        let mut enclosed = stream_proc.with_db(&arch_db);
        enclosed.handshake_start()?;
        enclosed.auto_last(&LastMode::Lane(vec![None, None, Some(2..1)]), "last test")?;
        enclosed.auto_strb(&StrobeMode::Lane(vec![false, true, true]), "strb test")?;
        enclosed.auto_stai(0, "stai test")?;
        enclosed.auto_endi(2, "endi test")?;
        enclosed.auto_user_default("user default")?;
        enclosed.auto_user(&ElementType::Bits(bitvec![0, 1]), "user 10")?;
        let proc = enclosed.get();
        println!("{}", proc.process().declare(&arch_db)?);
    }

    Ok(())
}
