use std::convert::TryInto;

use til_vhdl::{
    common::{
        logical::logicaltype::{Direction, Synchronicity},
        physical::{fields::Fields, stream::PhysicalStream},
    },
    ir::{
        physical_properties::InterfaceDirection, Database, Implementation, Interface, InternSelf,
        IntoVhdl, Ir, LogicalType, PhysicalProperties, Stream, Streamlet,
    },
    test_utils::test_stream_id,
};
use tydi_common::{
    error::{Error, Result},
    name::Name,
    numbers::{BitCount, Positive},
};
use tydi_vhdl::{
    architecture::{arch_storage::Arch, Architecture},
    declaration::Declare,
    package::Package,
};

extern crate til_vhdl;

#[test]
fn streamlet_new() -> Result<()> {
    let db = Database::default();
    let imple = Implementation::Link;
    let implid = db.intern_implementation(imple.clone());
    let streamlet = Streamlet::try_portless("test")?.with_implementation(&db, Some(implid));
    assert_eq!(
        db.lookup_intern_streamlet(streamlet)
            .implementation(&db)
            .unwrap(),
        imple
    );
    Ok(())
}

#[test]
fn streamlet_to_vhdl() -> Result<()> {
    let _db = Database::default();
    let db = &_db;
    let mut _vhdl_db = tydi_vhdl::architecture::arch_storage::db::Database::default();
    let vhdl_db = &_vhdl_db;
    let stream = test_stream_id(db)?;
    let streamlet = Streamlet::try_new(db, "test", vec![("a", stream, InterfaceDirection::In)])?;
    let component = streamlet.canonical(db, vhdl_db, "")?;
    let mut package = Package::new_default_empty();
    package.add_component(component);

    assert_eq!(
        r#"library ieee;
use ieee.std_logic_1164.all;

package work is

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

end work;"#,
        package.declare(vhdl_db)?
    );

    let architecture = Architecture::new_default(&package, Name::try_new("test_com")?)?;
    assert_eq!(
        r#"library ieee;
use ieee.std_logic_1164.all;

library work;
use work.work.all;

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
        architecture.declare(vhdl_db)?
    );

    Ok(())
}

#[test]
fn playground() -> Result<()> {
    let _db = Database::default();
    let db = &_db;
    let mut _vhdl_db = tydi_vhdl::architecture::arch_storage::db::Database::default();
    let vhdl_db = &_vhdl_db;
    let bits = LogicalType::try_new_bits(4)?.intern(db);
    let data_type =
        LogicalType::try_new_union(db, None, vec![("a", bits), ("b", bits)])?.intern(db);
    //let data_type = LogicalType::try_new_bits(4)?.intern(db);let null_type = LogicalType::Null.intern(db);
    let null_type = LogicalType::Null.intern(db);
    let stream = Stream::try_new(
        db,
        data_type,
        "1.0",
        1,
        Synchronicity::Sync,
        4,
        Direction::Forward,
        null_type,
        false,
    )?;
    let streamlet = Streamlet::try_new(
        db,
        "test",
        vec![
            ("a", stream, InterfaceDirection::In),
            ("b", stream, InterfaceDirection::Out),
        ],
    )?;
    let component = streamlet.canonical(db, vhdl_db, "")?;
    let mut package = Package::new_default_empty();
    package.add_component(component);

    println!("{}", package.declare(vhdl_db)?);

    let architecture = Architecture::new_default(&package, Name::try_new("test_com")?)?;
    println!("{}", architecture.declare(vhdl_db)?);

    Ok(())
}
