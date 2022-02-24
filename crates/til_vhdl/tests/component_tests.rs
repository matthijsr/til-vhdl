use std::{
    convert::{TryFrom, TryInto},
    sync::Arc,
};

use til_parser::query::into_query_storage;
use til_query::{
    common::logical::logicaltype::{
        stream::{Direction, Stream, Synchronicity},
        LogicalType,
    },
    ir::{
        db::Database,
        implementation::{structure::Structure, Implementation},
        physical_properties::InterfaceDirection,
        streamlet::Streamlet,
        traits::{GetSelf, InternSelf},
        Ir,
    },
    test_utils::{test_stream_id, test_stream_id_custom},
};
use til_vhdl::IntoVhdl;
use tydi_common::{error::Result, name::Name, numbers::NonNegative};
use tydi_vhdl::{
    architecture::{arch_storage::Arch, Architecture},
    common::vhdl_name::VhdlNameSelf,
    component::Component,
    declaration::Declare,
    package::Package,
};

extern crate til_vhdl;

#[test]
fn streamlet_to_vhdl() -> Result<()> {
    let _db = Database::default();
    let db = &_db;
    let mut _arch_db = tydi_vhdl::architecture::arch_storage::db::Database::default();
    let arch_db = &mut _arch_db;
    let stream = test_stream_id(db, 4)?;
    let streamlet = Streamlet::new()
        .try_with_name("test")?
        .with_ports(db, vec![("a", stream, InterfaceDirection::In)])?;
    let component = Arc::new(streamlet.canonical(db, arch_db, "")?);
    let mut package = Package::new_default_empty();
    package.add_component(component);

    assert_eq!(
        r#"library ieee;
use ieee.std_logic_1164.all;

package default is

  component test_com
    port (
      clk : in std_logic;
      rst : in std_logic;
      a_valid : in std_logic;
      a_ready : out std_logic;
      a_data : in std_logic_vector(3 downto 0);
      a_last : in std_logic;
      a_strb : in std_logic
    );
  end component;

end default;"#,
        package.declare(arch_db)?
    );

    let architecture = Architecture::new_default(&package, Name::try_new("test_com")?)?;
    assert_eq!(
        r#"library ieee;
use ieee.std_logic_1164.all;

library work;
use work.default.all;

entity test_com is
  port (
    clk : in std_logic;
    rst : in std_logic;
    a_valid : in std_logic;
    a_ready : out std_logic;
    a_data : in std_logic_vector(3 downto 0);
    a_last : in std_logic;
    a_strb : in std_logic
  );
end test_com;

architecture Behavioral of test_com is
begin
end Behavioral;"#,
        architecture.declare(arch_db)?
    );

    Ok(())
}

// Validate the output signals at each complexity
#[test]
fn streamlet_to_vhdl_complexities() -> Result<()> {
    let _db = Database::default();
    let db = &_db;
    let complexity_decls = (1..8)
        .map(|c: NonNegative| {
            let mut _arch_db = tydi_vhdl::architecture::arch_storage::db::Database::default();
            let arch_db = &mut _arch_db;
            let stream = test_stream_id_custom(db, 4, 2.0, 0, c)?;
            let streamlet = Streamlet::new()
                .try_with_name("test")?
                .with_ports(db, vec![("a", stream, InterfaceDirection::In)])?;
            let component: Component = streamlet.canonical(db, arch_db, "")?;
            component.declare(arch_db)
        })
        .collect::<Result<Vec<_>>>()?;

    assert_eq!(
        r#"component test_com
  port (
    clk : in std_logic;
    rst : in std_logic;
    a_valid : in std_logic;
    a_ready : out std_logic;
    a_data : in std_logic_vector(7 downto 0)
  );
end component;"#,
        complexity_decls[0],
        "Complexity 1"
    );
    assert_eq!(
        r#"component test_com
  port (
    clk : in std_logic;
    rst : in std_logic;
    a_valid : in std_logic;
    a_ready : out std_logic;
    a_data : in std_logic_vector(7 downto 0)
  );
end component;"#,
        complexity_decls[1],
        "Complexity 2"
    );
    assert_eq!(
        r#"component test_com
  port (
    clk : in std_logic;
    rst : in std_logic;
    a_valid : in std_logic;
    a_ready : out std_logic;
    a_data : in std_logic_vector(7 downto 0)
  );
end component;"#,
        complexity_decls[2],
        "Complexity 3"
    );
    assert_eq!(
        r#"component test_com
  port (
    clk : in std_logic;
    rst : in std_logic;
    a_valid : in std_logic;
    a_ready : out std_logic;
    a_data : in std_logic_vector(7 downto 0)
  );
end component;"#,
        complexity_decls[3],
        "Complexity 4"
    );
    assert_eq!(
        r#"component test_com
  port (
    clk : in std_logic;
    rst : in std_logic;
    a_valid : in std_logic;
    a_ready : out std_logic;
    a_data : in std_logic_vector(7 downto 0);
    a_endi : in std_logic
  );
end component;"#,
        complexity_decls[4],
        "Complexity 5"
    );
    assert_eq!(
        r#"component test_com
  port (
    clk : in std_logic;
    rst : in std_logic;
    a_valid : in std_logic;
    a_ready : out std_logic;
    a_data : in std_logic_vector(7 downto 0);
    a_stai : in std_logic;
    a_endi : in std_logic
  );
end component;"#,
        complexity_decls[5],
        "Complexity 6"
    );
    assert_eq!(
        r#"component test_com
  port (
    clk : in std_logic;
    rst : in std_logic;
    a_valid : in std_logic;
    a_ready : out std_logic;
    a_data : in std_logic_vector(7 downto 0);
    a_stai : in std_logic;
    a_endi : in std_logic;
    a_strb : in std_logic_vector(1 downto 0)
  );
end component;"#,
        complexity_decls[6],
        "Complexity 7"
    );

    Ok(())
}

#[test]
fn split_streams_path_names() -> Result<()> {
    let db = into_query_storage(
        "
namespace my::test::space {
    type stream_base1 = Stream(
        data: Bits(8),
        dimensionality: 0,
        synchronicity: Sync,
        complexity: 4,
        direction: Forward,
    );
    type stream_base2 = Stream(
        data: Bits(8),
        dimensionality: 0,
        synchronicity: Sync,
        complexity: 4,
        direction: Reverse,
    );
    type multi_stream_group = Group(a: stream_base1, b: stream_base1, c: stream_base2);
    type multi_stream = Stream(
        data: multi_stream_group,
        throughput: 3.0,
        dimensionality: 0,
        synchronicity: Sync,
        complexity: 4,
        direction: Forward,
    );

    streamlet multi_streamlet = (x: in multi_stream, y: out multi_stream);
}
    ",
    )?;

    let proj = db.project();
    let streamlet = proj
        .namespaces()
        .get(&("my::test::space".try_into()?))
        .unwrap()
        .get(&db)
        .get_streamlet(&db, "multi_streamlet")?;

    let mut arch_db = tydi_vhdl::architecture::arch_storage::db::Database::default();

    let comp: Component = streamlet.canonical(&db, &mut arch_db, None)?;
    assert_eq!(
        r#"component my__test__space__multi_streamlet_com
  port (
    clk : in std_logic;
    rst : in std_logic;
    x__a_valid : in std_logic;
    x__a_ready : out std_logic;
    x__a_data : in std_logic_vector(23 downto 0);
    x__b_valid : in std_logic;
    x__b_ready : out std_logic;
    x__b_data : in std_logic_vector(23 downto 0);
    x__c_valid : in std_logic;
    x__c_ready : out std_logic;
    x__c_data : in std_logic_vector(23 downto 0);
    y__a_valid : out std_logic;
    y__a_ready : in std_logic;
    y__a_data : out std_logic_vector(23 downto 0);
    y__b_valid : out std_logic;
    y__b_ready : in std_logic;
    y__b_data : out std_logic_vector(23 downto 0);
    y__c_valid : out std_logic;
    y__c_ready : in std_logic;
    y__c_data : out std_logic_vector(23 downto 0)
  );
end component;"#,
        comp.declare(&arch_db)?
    );

    Ok(())
}

#[test]
fn playground() -> Result<()> {
    let mut _db = Database::default();
    let db = &mut _db;

    let mut _arch_db = tydi_vhdl::architecture::arch_storage::db::Database::default();
    let arch_db = &mut _arch_db;

    let bits = LogicalType::try_new_bits(4)?.intern(db);
    let data_type = LogicalType::try_new_union(None, vec![("a", bits), ("b", bits)])?.intern(db);
    let null_type = LogicalType::null_id(db);
    let stream = Stream::try_new(
        db,
        data_type,
        1.0,
        1,
        Synchronicity::Sync,
        4,
        Direction::Forward,
        null_type,
        false,
    )?;

    let streamlet = Streamlet::new().try_with_name("test")?.with_ports(
        db,
        vec![
            ("a", stream, InterfaceDirection::In),
            ("b", stream, InterfaceDirection::Out),
        ],
    )?;

    let mut structure = Structure::try_from(&streamlet)?;
    structure.try_add_connection(db, "a", "b")?;
    let implementation = Implementation::structural(structure)?
        .try_with_name("structural")?
        .intern(db);
    let streamlet = streamlet.with_implementation(Some(implementation));

    let mut package = Package::new_default_empty();
    let component: Arc<Component> = Arc::new(streamlet.canonical(db, arch_db, "")?);
    arch_db.set_subject_component_name(Arc::new(component.vhdl_name().clone()));
    package.add_component(component);
    let package = Arc::new(package);

    arch_db.set_default_package(package);

    let declaration: String = streamlet.canonical(db, arch_db, "")?;

    println!("{}", arch_db.default_package().declare(arch_db)?);

    println!("{}", declaration);

    Ok(())
}
