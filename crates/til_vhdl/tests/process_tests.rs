use std::sync::Arc;

use bitvec::prelude::*;

use til_parser::query::into_query_storage_default;
use til_query::{
    common::{
        physical::complexity::Complexity,
        signals::{PhysicalSignals, PhysicalTransfers},
        transfer::{
            element_type::ElementType,
            physical_transfer::{LastMode, PhysicalTransfer, StrobeMode},
        },
    },
    ir::{
        connection::InterfaceReference,
        implementation::structure::streamlet_instance::StreamletInstance, traits::GetSelf, Ir,
    },
};
use til_vhdl::{
    common::signals::PhysicalStreamProcess,
    ir::{
        physical_properties::{VhdlDomain, VhdlDomainListOrDefault},
        streamlet::{create_instance, StreamletArchitecture},
    },
    IntoVhdl,
};
use tydi_common::{error::Result, map::InsertionOrderedMap, name::PathName, numbers::Positive};
use tydi_intern::Id;
use tydi_vhdl::{
    architecture::arch_storage::Arch,
    common::vhdl_name::VhdlNameSelf,
    declaration::{Declare, ObjectDeclaration},
    package::Package,
};

#[test]
fn process_playground() -> Result<()> {
    let db = into_query_storage_default(
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

    let proj = db.project_ref();
    let streamlet = proj
        .namespaces()
        .get(&("my::test::space".try_into()?))
        .unwrap()
        .get(&db)
        .get_streamlet_id("doc_streamlet")?;

    let mut arch_db = tydi_vhdl::architecture::arch_storage::db::Database::default();
    let mut package = Package::new_default_empty();

    let streamlet_instance = StreamletInstance::new_assign_default(&db, "a", streamlet)?;

    let mut vhdl_streamlet = streamlet_instance
        .definition()
        .canonical(&db, &mut arch_db, None)?;
    let component = vhdl_streamlet.to_component();

    arch_db.set_subject_component_name(Arc::new(component.vhdl_name().clone()));
    package.add_component(component);
    arch_db.set_default_package(Arc::new(package));

    let mut arch = vhdl_streamlet.to_architecture(&db, &mut arch_db)?;
    if let StreamletArchitecture::Generated(arch) = &mut arch {
        let domain_list = VhdlDomainListOrDefault::Default(
            VhdlDomain::<Id<ObjectDeclaration>>::default(&mut arch_db),
        );
        let instance = create_instance(
            &db,
            &mut arch_db,
            &streamlet_instance,
            arch,
            &domain_list,
            &InsertionOrderedMap::new(),
            None,
        )?;
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
        enclosed.auto_last(&LastMode::Lane(vec![None, None, Some(1..2)]), "last test")?;
        enclosed.auto_strb(&StrobeMode::Lane(vec![false, true, true]), "strb test")?;
        enclosed.auto_stai(0, "stai test")?;
        enclosed.auto_endi(2, "endi test")?;
        enclosed.auto_user_default("user default")?;
        enclosed.auto_user(&ElementType::Bits(bitvec![0, 1]), "user 10")?;
        enclosed.auto_data_default("data default")?;
        enclosed.auto_data(
            0,
            &ElementType::Bits(bitvec![0, 0, 1, 0, 0, 0, 0, 1]),
            "data[0] = 10000100",
        )?;
        enclosed.auto_data(
            1,
            &ElementType::Bits(bitvec![0, 0, 1, 0, 0, 0, 1, 1]),
            "data[1] = 11000100",
        )?;
        enclosed.auto_data(
            2,
            &ElementType::Bits(bitvec![0, 0, 1, 0, 0, 1, 0, 1]),
            "data[2] = 10100100",
        )?;
        let proc = enclosed.get();
        println!("{}", proc.process().declare(&arch_db)?);
    }

    Ok(())
}

#[test]
fn process_transfer_playground() -> Result<()> {
    let db = into_query_storage_default(
        "
namespace my::test::space {
    type stream1 = Stream(
        data: Bits(2),
        dimensionality: 3,
        throughput: 3,
        synchronicity: Sync,
        complexity: 8,
        direction: Forward,
        user: Bits(3),
    );

    #\
    streamlet documentation \
    is multi-line but can act as a split string\
    #
    streamlet doc_streamlet = (
      #interface documentation
is also
multiline#
      x: in stream1,
      y: out stream1,
    );
}
    ",
    )?;

    let proj = db.project_ref();
    let streamlet = proj
        .namespaces()
        .get(&("my::test::space".try_into()?))
        .unwrap()
        .get(&db)
        .get_streamlet_id("doc_streamlet")?;

    let mut arch_db = tydi_vhdl::architecture::arch_storage::db::Database::default();
    let mut package = Package::new_default_empty();

    let streamlet_instance = StreamletInstance::new_assign_default(&db, "a", streamlet)?;

    let mut vhdl_streamlet = streamlet_instance
        .definition()
        .canonical(&db, &mut arch_db, None)?;
    let component = vhdl_streamlet.to_component();

    arch_db.set_subject_component_name(Arc::new(component.vhdl_name().clone()));
    package.add_component(component);
    arch_db.set_default_package(Arc::new(package));

    let mut arch = vhdl_streamlet.to_architecture(&db, &mut arch_db)?;
    if let StreamletArchitecture::Generated(arch) = &mut arch {
        let transfer_1 =
            PhysicalTransfer::new(Complexity::new_major(8), Positive::new(3).unwrap(), 2, 3, 3)
                .with_logical_transfer(([Some("11"), None, Some("11")], "101"))?; // [[[11, -, 11
        let transfer_2 =
            PhysicalTransfer::new(Complexity::new_major(8), Positive::new(3).unwrap(), 2, 3, 3)
                .with_logical_transfer([("01", Some(0..0)), ("10", None), ("00", None)])?; // 10], [01, 00
        let transfer_3 =
            PhysicalTransfer::new(Complexity::new_major(8), Positive::new(3).unwrap(), 2, 3, 3)
                .with_logical_transfer([("01", Some(0..1)), ("-", Some(2..2)), ("-", None)])?; // 10]], -], -

        let domain_list = VhdlDomainListOrDefault::Default(
            VhdlDomain::<Id<ObjectDeclaration>>::default(&mut arch_db),
        );
        let instance = create_instance(
            &db,
            &mut arch_db,
            &streamlet_instance,
            arch,
            &domain_list,
            &InsertionOrderedMap::new(),
            None,
        )?;
        let iref = InterfaceReference::try_from(("a", "x"))?;
        let port_obj = instance.get(&iref).unwrap();
        let stream_obj = port_obj
            .typed_stream()
            .logical_stream()
            .streams()
            .get(&PathName::new_empty())
            .unwrap();
        let stream_proc_source = PhysicalStreamProcess::from(stream_obj.clone());

        let iref = InterfaceReference::try_from(("a", "y"))?;
        let port_obj = instance.get(&iref).unwrap();
        let stream_obj = port_obj
            .typed_stream()
            .logical_stream()
            .streams()
            .get(&PathName::new_empty())
            .unwrap();
        let stream_proc_sink = PhysicalStreamProcess::from(stream_obj.clone());

        let mut drive_stream = stream_proc_source.with_db(&arch_db);
        drive_stream.open_transfer()?;
        drive_stream.transfer(transfer_1.clone(), false, "test message drive 1")?;
        drive_stream.transfer(transfer_2.clone(), false, "test message drive 2")?;
        drive_stream.transfer(transfer_3.clone(), false, "test message drive 3")?;
        drive_stream.close_transfer()?;
        let proc = drive_stream.get();
        assert_eq!(
            r#"process is
begin
  \a__x_valid\ <= '1';
  \a__x_data\(1 downto 0) <= "11";
  \a__x_data\(5 downto 4) <= "11";
  \a__x_last\(2 downto 0) <= (others => '0');
  \a__x_last\(5 downto 3) <= (others => '0');
  \a__x_last\(8 downto 6) <= (others => '0');
  \a__x_strb\ <= "101";
  \a__x_user\(2 downto 0) <= "101";
  wait until rising_edge(clk) and \a__x_ready\ = '1';
  \a__x_data\(1 downto 0) <= "10";
  \a__x_data\(3 downto 2) <= "01";
  \a__x_data\(5 downto 4) <= "00";
  \a__x_last\(2 downto 0) <= "001";
  \a__x_last\(5 downto 3) <= (others => '0');
  \a__x_last\(8 downto 6) <= (others => '0');
  \a__x_strb\ <= "111";
  \a__x_stai\ <= std_logic_vector(to_unsigned(0, 2));
  \a__x_endi\ <= std_logic_vector(to_unsigned(2, 2));
  wait until rising_edge(clk) and \a__x_ready\ = '1';
  \a__x_data\(1 downto 0) <= "10";
  \a__x_last\(2 downto 0) <= "011";
  \a__x_last\(5 downto 3) <= "100";
  \a__x_last\(8 downto 6) <= (others => '0');
  \a__x_strb\ <= "100";
  wait until rising_edge(clk) and \a__x_ready\ = '1';
  \a__x_valid\ <= '0';
  wait until rising_edge(clk);
end process \a__x\;"#,
            proc.process().declare(&arch_db)?
        );

        let mut compare_stream = stream_proc_sink.with_db(&arch_db);
        compare_stream.open_transfer()?;
        compare_stream.transfer(transfer_1.clone(), false, "test message compare 1")?;
        compare_stream.transfer(transfer_2.clone(), false, "test message compare 2")?;
        compare_stream.transfer(transfer_3.clone(), false, "test message compare 3")?;
        compare_stream.close_transfer()?;
        let proc = compare_stream.get();
        assert_eq!(
            r#"process is
begin
  wait until rising_edge(clk) and \a__y_valid\ = '1';
  assert \a__y_data(1 downto 0) = "11" report "test message compare 1";
  assert \a__y_data(5 downto 4) = "11" report "test message compare 1";
  assert \a__y_last(2 downto 0) = (others => '0') report "test message compare 1";
  assert \a__y_last(5 downto 3) = (others => '0') report "test message compare 1";
  assert \a__y_last(8 downto 6) = (others => '0') report "test message compare 1";
  assert \a__y_strb = "101" report "test message compare 1";
  assert \a__y_user(2 downto 0) = "101" report "test message compare 1";
  \a__y_ready\ <= '1';
  wait until rising_edge(clk) and \a__y_valid\ = '1';
  assert \a__y_data\(1 downto 0) = "10" report "test message compare 2";
  assert \a__y_data\(3 downto 2) = "01" report "test message compare 2";
  assert \a__y_data\(5 downto 4) = "00" report "test message compare 2";
  assert \a__y_last\(2 downto 0) = "001" report "test message compare 2";
  assert \a__y_last\(5 downto 3) = (others => '0') report "test message compare 2";
  assert \a__y_last\(8 downto 6) = (others => '0') report "test message compare 2";
  assert \a__y_strb\ = "111" report "test message compare 2";
  assert \a__y_stai\ = std_logic_vector(to_unsigned(0, 2)) report "test message compare 2";
  assert \a__y_endi\ = std_logic_vector(to_unsigned(2, 2)) report "test message compare 2";
  \a__y_ready\ <= '1';
  wait until rising_edge(clk) and \a__y_valid\ = '1';
  assert \a__y_data\(1 downto 0) = "10" report "test message compare 3";
  assert \a__y_last\(2 downto 0) = "011" report "test message compare 3";
  assert \a__y_last\(5 downto 3) = "100" report "test message compare 3";
  assert \a__y_last\(8 downto 6) = (others => '0') report "test message compare 3";
  assert \a__y_strb\ = "100" report "test message compare 3";
  \a__y_ready\ <= '1';
  wait until rising_edge(clk) and \a__y_valid\ = '1';
  \a__y_ready\ <= '0';
  wait until rising_edge(clk);
end process \a__y\;"#,
            proc.process().declare(&arch_db)?
        );
    } else {
        assert!(false, "Expected generated arch");
    }

    Ok(())
}
