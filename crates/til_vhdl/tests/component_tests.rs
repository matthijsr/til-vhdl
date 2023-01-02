use std::{
    convert::{TryFrom, TryInto},
    sync::Arc,
};

use til_parser::query::into_query_storage_default;
use til_query::{
    common::{
        logical::{
            logical_stream::SynthesizeLogicalStream,
            logicaltype::{
                stream::{Stream, Synchronicity},
                LogicalType,
            },
        },
        stream_direction::StreamDirection,
    },
    ir::{
        db::Database,
        generics::param_value::{combination::GenericParamValueOps, GenericParamValue},
        implementation::{structure::Structure, Implementation},
        physical_properties::InterfaceDirection,
        streamlet::Streamlet,
        traits::{GetSelf, InternArc, InternSelf},
        Ir,
    },
    test_utils::{
        simple_streamlet_with_interface_params, simple_structural_streamlet,
        simple_structural_streamlet_with_behav_params, streamlet_without_impl,
        streamlet_without_impl_with_behav_params,
        structural_streamlet_with_interface_params_and_instances, test_stream_id,
        test_stream_id_custom,
    },
};
use til_vhdl::IntoVhdl;
use tydi_common::{error::Result, name::Name, numbers::NonNegative};
use tydi_vhdl::{
    architecture::{arch_storage::Arch, Architecture},
    common::vhdl_name::VhdlNameSelf,
    declaration::Declare,
    package::Package,
};

use crate::common::ir_streamlet_to_vhdl;

mod common;

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
    let component = streamlet.canonical(db, arch_db, "")?.to_component();
    let mut package = Package::new_default_empty();
    package.add_component(component);

    assert_eq!(
        r#"library ieee;
use ieee.std_logic_1164.all;

package default is

  component test_com is
    port (
      clk : in std_logic;
      rst : in std_logic;
      a_valid : in std_logic;
      a_ready : out std_logic;
      a_data : in std_logic_vector(3 downto 0);
      a_last : in std_logic;
      a_strb : in std_logic
    );
  end component test_com;

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
            let component = streamlet.canonical(db, arch_db, "")?.to_component();
            component.declare(arch_db)
        })
        .collect::<Result<Vec<_>>>()?;

    assert_eq!(
        r#"component test_com is
  port (
    clk : in std_logic;
    rst : in std_logic;
    a_valid : in std_logic;
    a_ready : out std_logic;
    a_data : in std_logic_vector(7 downto 0)
  );
end component test_com;"#,
        complexity_decls[0],
        "Complexity 1"
    );
    assert_eq!(
        r#"component test_com is
  port (
    clk : in std_logic;
    rst : in std_logic;
    a_valid : in std_logic;
    a_ready : out std_logic;
    a_data : in std_logic_vector(7 downto 0)
  );
end component test_com;"#,
        complexity_decls[1],
        "Complexity 2"
    );
    assert_eq!(
        r#"component test_com is
  port (
    clk : in std_logic;
    rst : in std_logic;
    a_valid : in std_logic;
    a_ready : out std_logic;
    a_data : in std_logic_vector(7 downto 0)
  );
end component test_com;"#,
        complexity_decls[2],
        "Complexity 3"
    );
    assert_eq!(
        r#"component test_com is
  port (
    clk : in std_logic;
    rst : in std_logic;
    a_valid : in std_logic;
    a_ready : out std_logic;
    a_data : in std_logic_vector(7 downto 0)
  );
end component test_com;"#,
        complexity_decls[3],
        "Complexity 4"
    );
    assert_eq!(
        r#"component test_com is
  port (
    clk : in std_logic;
    rst : in std_logic;
    a_valid : in std_logic;
    a_ready : out std_logic;
    a_data : in std_logic_vector(7 downto 0);
    a_endi : in std_logic
  );
end component test_com;"#,
        complexity_decls[4],
        "Complexity 5"
    );
    assert_eq!(
        r#"component test_com is
  port (
    clk : in std_logic;
    rst : in std_logic;
    a_valid : in std_logic;
    a_ready : out std_logic;
    a_data : in std_logic_vector(7 downto 0);
    a_stai : in std_logic;
    a_endi : in std_logic
  );
end component test_com;"#,
        complexity_decls[5],
        "Complexity 6"
    );
    assert_eq!(
        r#"component test_com is
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
end component test_com;"#,
        complexity_decls[6],
        "Complexity 7"
    );

    Ok(())
}

#[test]
fn split_streams_path_names() -> Result<()> {
    let db = into_query_storage_default(
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

    let proj = db.project_ref();
    let streamlet = proj
        .namespaces()
        .get(&("my::test::space".try_into()?))
        .unwrap()
        .get(&db)
        .get_streamlet(&db, "multi_streamlet")?;

    let mut arch_db = tydi_vhdl::architecture::arch_storage::db::Database::default();

    let comp = streamlet.canonical(&db, &mut arch_db, None)?.to_component();
    assert_eq!(
        r#"component my_0_test_0_space_0_multi_streamlet_com is
  port (
    clk : in std_logic;
    rst : in std_logic;
    x_0_a_valid : in std_logic;
    x_0_a_ready : out std_logic;
    x_0_a_data : in std_logic_vector(23 downto 0);
    x_0_b_valid : in std_logic;
    x_0_b_ready : out std_logic;
    x_0_b_data : in std_logic_vector(23 downto 0);
    x_0_c_valid : out std_logic;
    x_0_c_ready : in std_logic;
    x_0_c_data : out std_logic_vector(23 downto 0);
    y_0_a_valid : out std_logic;
    y_0_a_ready : in std_logic;
    y_0_a_data : out std_logic_vector(23 downto 0);
    y_0_b_valid : out std_logic;
    y_0_b_ready : in std_logic;
    y_0_b_data : out std_logic_vector(23 downto 0);
    y_0_c_valid : in std_logic;
    y_0_c_ready : out std_logic;
    y_0_c_data : in std_logic_vector(23 downto 0)
  );
end component my_0_test_0_space_0_multi_streamlet_com;"#,
        comp.declare(&arch_db)?
    );

    Ok(())
}

#[test]
fn type_reference() -> Result<()> {
    let db = into_query_storage_default(
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
    type multi_stream_group2 = Group(a: stream_base1, b: stream_base1, c: stream_base2, d: Bits(2));
    type multi_stream2 = Stream(
        data: multi_stream_group2,
        throughput: 3.0,
        dimensionality: 0,
        synchronicity: Sync,
        complexity: 4,
        direction: Forward,
    );
}
    ",
    )?;

    let proj = db.project_ref();
    let namespace = proj
        .namespaces()
        .get(&("my::test::space".try_into()?))
        .unwrap()
        .get(&db);
    let stream1 = namespace.get_stream_id_no_assignments(&db, "multi_stream")?;
    let stream2 = namespace.get_stream_id_no_assignments(&db, "multi_stream2")?;

    let result1 = stream1.synthesize(&db)?;
    assert_eq!(
        r#"Scope (
  name:  ,
  child: Group (
    a: Stream (
      physical_stream: a,
      data: Bits(8),
      direction: Forward,
      complexity: 4,
      dimensionality: Fixed(0),
      transfer_scope: Sync(),
      element_lanes: 3,
      user: Null
    )
    b: Stream (
      physical_stream: b,
      data: Bits(8),
      direction: Forward,
      complexity: 4,
      dimensionality: Fixed(0),
      transfer_scope: Sync(),
      element_lanes: 3,
      user: Null
    )
    c: Stream (
      physical_stream: c,
      data: Bits(8),
      direction: Reverse,
      complexity: 4,
      dimensionality: Fixed(0),
      transfer_scope: Sync(),
      element_lanes: 3,
      user: Null
    )
  )
)"#,
        result1.type_reference().to_string()
    );

    let result2 = stream2.synthesize(&db)?;
    assert_eq!(
        r#"Stream (
  physical_stream: ,
  data: Group (
    a: Stream (
      physical_stream: a,
      data: Bits(8),
      direction: Forward,
      complexity: 4,
      dimensionality: Fixed(0),
      transfer_scope: Sync(),
      element_lanes: 3,
      user: Null
    )
    b: Stream (
      physical_stream: b,
      data: Bits(8),
      direction: Forward,
      complexity: 4,
      dimensionality: Fixed(0),
      transfer_scope: Sync(),
      element_lanes: 3,
      user: Null
    )
    c: Stream (
      physical_stream: c,
      data: Bits(8),
      direction: Reverse,
      complexity: 4,
      dimensionality: Fixed(0),
      transfer_scope: Sync(),
      element_lanes: 3,
      user: Null
    )
    d: Bits(2)
  ),
  direction: Forward,
  complexity: 4,
  dimensionality: Fixed(0),
  transfer_scope: Root,
  element_lanes: 3,
  user: Null
)"#,
        result2.type_reference().to_string()
    );

    Ok(())
}

#[test]
fn component_and_port_documentation() -> Result<()> {
    let db = into_query_storage_default(
        "
namespace my::test::space {
    type stream1 = Stream(
        data: Bits(8),
        dimensionality: 0,
        synchronicity: Sync,
        complexity: 4,
        direction: Forward,
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
        .get_streamlet(&db, "doc_streamlet")?;

    let mut arch_db = tydi_vhdl::architecture::arch_storage::db::Database::default();

    let comp = streamlet.canonical(&db, &mut arch_db, None)?.to_component();
    assert_eq!(
        r#"-- streamlet documentation is multi-line but can act as a split string
component my_0_test_0_space_0_doc_streamlet_com is
  port (
    clk : in std_logic;
    rst : in std_logic;
    -- interface documentation
    -- is also
    -- multiline
    x_valid : in std_logic;
    x_ready : out std_logic;
    x_data : in std_logic_vector(7 downto 0)
  );
end component my_0_test_0_space_0_doc_streamlet_com;"#,
        comp.declare(&arch_db)?
    );

    Ok(())
}

#[test]
fn basic_comp_arch() -> Result<()> {
    let mut _db = Database::default();
    let db = &mut _db;

    let streamlet = simple_structural_streamlet(db, "test")?;

    let mut _arch_db = tydi_vhdl::architecture::arch_storage::db::Database::default();
    let arch_db = &mut _arch_db;

    let package = Package::new_default_empty();

    let mut streamlet = ir_streamlet_to_vhdl(streamlet, db, arch_db, package)?;

    let streamlet_arch = streamlet.to_architecture(db, arch_db)?;

    assert_eq!(
        r#"component test_com is
  port (
    clk : in std_logic;
    rst : in std_logic;
    a_valid : in std_logic;
    a_ready : out std_logic;
    a_data : in std_logic_vector(4 downto 0);
    a_last : in std_logic;
    a_strb : in std_logic;
    b_valid : out std_logic;
    b_ready : in std_logic;
    b_data : out std_logic_vector(4 downto 0);
    b_last : out std_logic;
    b_strb : out std_logic
  );
end component test_com;"#,
        streamlet.to_component().declare(arch_db)?
    );

    assert_eq!(
        r#"library ieee;
use ieee.std_logic_1164.all;

package default is

  component test_com is
    port (
      clk : in std_logic;
      rst : in std_logic;
      a_valid : in std_logic;
      a_ready : out std_logic;
      a_data : in std_logic_vector(4 downto 0);
      a_last : in std_logic;
      a_strb : in std_logic;
      b_valid : out std_logic;
      b_ready : in std_logic;
      b_data : out std_logic_vector(4 downto 0);
      b_last : out std_logic;
      b_strb : out std_logic
    );
  end component test_com;

end default;"#,
        arch_db.default_package().declare(arch_db)?
    );

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
    a_data : in std_logic_vector(4 downto 0);
    a_last : in std_logic;
    a_strb : in std_logic;
    b_valid : out std_logic;
    b_ready : in std_logic;
    b_data : out std_logic_vector(4 downto 0);
    b_last : out std_logic;
    b_strb : out std_logic
  );
end test_com;

architecture structural of test_com is
begin
  b_valid <= a_valid;
  a_ready <= b_ready;
  b_data <= a_data;
  b_last <= a_last;
  b_strb <= a_strb;
end structural;"#,
        streamlet_arch.declare(arch_db)?
    );

    Ok(())
}

#[test]
fn basic_comp_arch_with_behav_params() -> Result<()> {
    let mut _db = Database::default();
    let db = &mut _db;

    let streamlet = simple_structural_streamlet_with_behav_params(db, "test")?;

    let mut _arch_db = tydi_vhdl::architecture::arch_storage::db::Database::default();
    let arch_db = &mut _arch_db;

    let package = Package::new_default_empty();

    let mut streamlet = ir_streamlet_to_vhdl(streamlet, db, arch_db, package)?;

    let streamlet_arch = streamlet.to_architecture(db, arch_db)?;

    assert_eq!(
        r#"component test_com is
  generic (
    pa : natural := 0;
    pb : positive := 2;
    pc : integer := -2
  );
  port (
    clk : in std_logic;
    rst : in std_logic;
    a_valid : in std_logic;
    a_ready : out std_logic;
    a_data : in std_logic_vector(4 downto 0);
    a_last : in std_logic;
    a_strb : in std_logic;
    b_valid : out std_logic;
    b_ready : in std_logic;
    b_data : out std_logic_vector(4 downto 0);
    b_last : out std_logic;
    b_strb : out std_logic
  );
end component test_com;"#,
        streamlet.to_component().declare(arch_db)?
    );

    assert_eq!(
        r#"library ieee;
use ieee.std_logic_1164.all;

package default is

  component test_com is
    generic (
      pa : natural := 0;
      pb : positive := 2;
      pc : integer := -2
    );
    port (
      clk : in std_logic;
      rst : in std_logic;
      a_valid : in std_logic;
      a_ready : out std_logic;
      a_data : in std_logic_vector(4 downto 0);
      a_last : in std_logic;
      a_strb : in std_logic;
      b_valid : out std_logic;
      b_ready : in std_logic;
      b_data : out std_logic_vector(4 downto 0);
      b_last : out std_logic;
      b_strb : out std_logic
    );
  end component test_com;

end default;"#,
        arch_db.default_package().declare(arch_db)?
    );

    assert_eq!(
        r#"library ieee;
use ieee.std_logic_1164.all;

library work;
use work.default.all;

entity test_com is
  generic (
    pa : natural := 0;
    pb : positive := 2;
    pc : integer := -2
  );
  port (
    clk : in std_logic;
    rst : in std_logic;
    a_valid : in std_logic;
    a_ready : out std_logic;
    a_data : in std_logic_vector(4 downto 0);
    a_last : in std_logic;
    a_strb : in std_logic;
    b_valid : out std_logic;
    b_ready : in std_logic;
    b_data : out std_logic_vector(4 downto 0);
    b_last : out std_logic;
    b_strb : out std_logic
  );
end test_com;

architecture structural of test_com is
begin
  b_valid <= a_valid;
  a_ready <= b_ready;
  b_data <= a_data;
  b_last <= a_last;
  b_strb <= a_strb;
end structural;"#,
        streamlet_arch.declare(arch_db)?
    );

    Ok(())
}

#[test]
fn basic_comp_arch_with_interface_params() -> Result<()> {
    let mut _db = Database::default();
    let db = &mut _db;

    let streamlet = simple_streamlet_with_interface_params(db, "test")?;

    let mut _arch_db = tydi_vhdl::architecture::arch_storage::db::Database::default();
    let arch_db = &mut _arch_db;

    let package = Package::new_default_empty();

    let mut streamlet = ir_streamlet_to_vhdl(streamlet, db, arch_db, package)?;

    let streamlet_arch = streamlet.to_architecture(db, arch_db)?;

    assert_eq!(
        r#"component test_com is
  generic (
    pa : positive := 5
  );
  port (
    clk : in std_logic;
    rst : in std_logic;
    a_valid : in std_logic;
    a_ready : out std_logic;
    a_data : in std_logic_vector(4 downto 0);
    a_last : in std_logic_vector((pa + 1) - 1 downto 0);
    a_strb : in std_logic;
    b_valid : out std_logic;
    b_ready : in std_logic;
    b_data : out std_logic_vector(4 downto 0);
    b_last : out std_logic_vector((pa + 1) - 1 downto 0);
    b_strb : out std_logic
  );
end component test_com;"#,
        streamlet.to_component().declare(arch_db)?
    );

    assert_eq!(
        r#"library ieee;
use ieee.std_logic_1164.all;

package default is

  component test_com is
    generic (
      pa : positive := 5
    );
    port (
      clk : in std_logic;
      rst : in std_logic;
      a_valid : in std_logic;
      a_ready : out std_logic;
      a_data : in std_logic_vector(4 downto 0);
      a_last : in std_logic_vector((pa + 1) - 1 downto 0);
      a_strb : in std_logic;
      b_valid : out std_logic;
      b_ready : in std_logic;
      b_data : out std_logic_vector(4 downto 0);
      b_last : out std_logic_vector((pa + 1) - 1 downto 0);
      b_strb : out std_logic
    );
  end component test_com;

end default;"#,
        arch_db.default_package().declare(arch_db)?
    );

    assert_eq!(
        r#"library ieee;
use ieee.std_logic_1164.all;

library work;
use work.default.all;

entity test_com is
  generic (
    pa : positive := 5
  );
  port (
    clk : in std_logic;
    rst : in std_logic;
    a_valid : in std_logic;
    a_ready : out std_logic;
    a_data : in std_logic_vector(4 downto 0);
    a_last : in std_logic_vector((pa + 1) - 1 downto 0);
    a_strb : in std_logic;
    b_valid : out std_logic;
    b_ready : in std_logic;
    b_data : out std_logic_vector(4 downto 0);
    b_last : out std_logic_vector((pa + 1) - 1 downto 0);
    b_strb : out std_logic
  );
end test_com;

architecture Behavioral of test_com is
begin
end Behavioral;"#,
        streamlet_arch.declare(arch_db)?
    );

    Ok(())
}

#[test]
fn basic_comp_arch_with_instance() -> Result<()> {
    let mut _db = Database::default();
    let db = &mut _db;

    let instance_streamlet = streamlet_without_impl(db, "inner")?;
    let parent_streamlet = streamlet_without_impl(db, "parent")?;
    let mut structure = Structure::try_from(&parent_streamlet)?;
    structure.try_add_streamlet_instance_default(db, "a", instance_streamlet.intern_arc(db))?;
    structure.try_add_connection(db, "a", ("a", "a"))?;
    structure.try_add_connection(db, "b", ("a", "b"))?;
    let implementation = Implementation::structural(structure)?
        .try_with_name("structural")?
        .intern(db);
    let parent_streamlet = parent_streamlet.with_implementation(Some(implementation));

    let mut _arch_db = tydi_vhdl::architecture::arch_storage::db::Database::default();
    let arch_db = &mut _arch_db;

    let package = Package::new_default_empty();

    let streamlet = ir_streamlet_to_vhdl(parent_streamlet, db, arch_db, package)?;

    let streamlet_arch = streamlet.to_architecture(db, arch_db)?;

    assert_eq!(
        r#"library ieee;
use ieee.std_logic_1164.all;

library work;
use work.default.all;

entity parent_com is
  port (
    clk : in std_logic;
    rst : in std_logic;
    a_valid : in std_logic;
    a_ready : out std_logic;
    a_data : in std_logic_vector(4 downto 0);
    a_last : in std_logic;
    a_strb : in std_logic;
    b_valid : out std_logic;
    b_ready : in std_logic;
    b_data : out std_logic_vector(4 downto 0);
    b_last : out std_logic;
    b_strb : out std_logic
  );
end parent_com;

architecture structural of parent_com is
  signal a_0_a_valid : std_logic;
  signal a_0_a_ready : std_logic;
  signal a_0_a_data : std_logic_vector(4 downto 0);
  signal a_0_a_last : std_logic;
  signal a_0_a_strb : std_logic;
  signal a_0_b_valid : std_logic;
  signal a_0_b_ready : std_logic;
  signal a_0_b_data : std_logic_vector(4 downto 0);
  signal a_0_b_last : std_logic;
  signal a_0_b_strb : std_logic;
begin
  a: inner_com port map(
    clk => clk,
    rst => rst,
    a_valid => a_0_a_valid,
    a_ready => a_0_a_ready,
    a_data => a_0_a_data,
    a_last => a_0_a_last,
    a_strb => a_0_a_strb,
    b_valid => a_0_b_valid,
    b_ready => a_0_b_ready,
    b_data => a_0_b_data,
    b_last => a_0_b_last,
    b_strb => a_0_b_strb
  );
  a_0_a_valid <= a_valid;
  a_ready <= a_0_a_ready;
  a_0_a_data <= a_data;
  a_0_a_last <= a_last;
  a_0_a_strb <= a_strb;
  b_valid <= a_0_b_valid;
  a_0_b_ready <= b_ready;
  b_data <= a_0_b_data;
  b_last <= a_0_b_last;
  b_strb <= a_0_b_strb;
end structural;"#,
        streamlet_arch.declare(arch_db)?
    );

    Ok(())
}

#[test]
fn basic_comp_arch_with_instance_and_behav_params() -> Result<()> {
    let mut _db = Database::default();
    let db = &mut _db;

    let instance_streamlet = streamlet_without_impl_with_behav_params(db, "inner")?;
    let parent_streamlet = streamlet_without_impl_with_behav_params(db, "parent")?;
    let mut structure = Structure::try_from(&parent_streamlet)?;
    structure.try_add_streamlet_instance_domains_default(
        db,
        "a",
        instance_streamlet.intern_arc(db),
        vec![
            ("pa", GenericParamValue::from(20)),
            (
                "pb",
                parent_streamlet
                    .try_get_parameter(db, &Name::try_new("pb")?)?
                    .into(),
            ),
            (
                "pc",
                parent_streamlet
                    .try_get_parameter(db, &Name::try_new("pa")?)?
                    .g_add(1)?
                    .into(),
            ),
        ],
    )?;
    structure.try_add_connection(db, "a", ("a", "a"))?;
    structure.try_add_connection(db, "b", ("a", "b"))?;
    let implementation = Implementation::structural(structure)?
        .try_with_name("structural")?
        .intern(db);
    let parent_streamlet = parent_streamlet.with_implementation(Some(implementation));

    let mut _arch_db = tydi_vhdl::architecture::arch_storage::db::Database::default();
    let arch_db = &mut _arch_db;

    let package = Package::new_default_empty();

    let streamlet = ir_streamlet_to_vhdl(parent_streamlet, db, arch_db, package)?;

    let streamlet_arch = streamlet.to_architecture(db, arch_db)?;

    assert_eq!(
        r#"library ieee;
use ieee.std_logic_1164.all;

library work;
use work.default.all;

entity parent_com is
  generic (
    pa : natural := 0;
    pb : positive := 2;
    pc : integer := -2
  );
  port (
    clk : in std_logic;
    rst : in std_logic;
    a_valid : in std_logic;
    a_ready : out std_logic;
    a_data : in std_logic_vector(4 downto 0);
    a_last : in std_logic;
    a_strb : in std_logic;
    b_valid : out std_logic;
    b_ready : in std_logic;
    b_data : out std_logic_vector(4 downto 0);
    b_last : out std_logic;
    b_strb : out std_logic
  );
end parent_com;

architecture structural of parent_com is
  signal a_0_a_valid : std_logic;
  signal a_0_a_ready : std_logic;
  signal a_0_a_data : std_logic_vector(4 downto 0);
  signal a_0_a_last : std_logic;
  signal a_0_a_strb : std_logic;
  signal a_0_b_valid : std_logic;
  signal a_0_b_ready : std_logic;
  signal a_0_b_data : std_logic_vector(4 downto 0);
  signal a_0_b_last : std_logic;
  signal a_0_b_strb : std_logic;
begin
  a: inner_com generic map(
    pa => 20,
    pb => pb,
    pc => pa + 1
  ) port map(
    clk => clk,
    rst => rst,
    a_valid => a_0_a_valid,
    a_ready => a_0_a_ready,
    a_data => a_0_a_data,
    a_last => a_0_a_last,
    a_strb => a_0_a_strb,
    b_valid => a_0_b_valid,
    b_ready => a_0_b_ready,
    b_data => a_0_b_data,
    b_last => a_0_b_last,
    b_strb => a_0_b_strb
  );
  a_0_a_valid <= a_valid;
  a_ready <= a_0_a_ready;
  a_0_a_data <= a_data;
  a_0_a_last <= a_last;
  a_0_a_strb <= a_strb;
  b_valid <= a_0_b_valid;
  a_0_b_ready <= b_ready;
  b_data <= a_0_b_data;
  b_last <= a_0_b_last;
  b_strb <= a_0_b_strb;
end structural;"#,
        streamlet_arch.declare(arch_db)?
    );

    Ok(())
}

#[test]
fn basic_comp_arch_with_instance_and_interface_params() -> Result<()> {
    let mut _db = Database::default();
    let db = &mut _db;

    let instance_streamlet = simple_streamlet_with_interface_params(db, "inner")?;
    let parent_streamlet = simple_streamlet_with_interface_params(db, "parent")?;
    let mut structure = Structure::try_from(&parent_streamlet)?;
    structure.try_add_streamlet_instance_domains_default(
        db,
        "a",
        instance_streamlet.intern_arc(db),
        vec![(
            "pa",
            parent_streamlet
                .try_get_parameter(db, &Name::try_new("pa")?)?
                .g_add(1)?,
        )],
    )?;

    structure.try_add_connection(db, "a", "b")?;
    structure.try_add_connection(db, ("a", "a"), ("a", "b"))?;
    let implementation = Implementation::structural(structure)?
        .try_with_name("structural")?
        .intern(db);
    let parent_streamlet = parent_streamlet.with_implementation(Some(implementation));

    let mut _arch_db = tydi_vhdl::architecture::arch_storage::db::Database::default();
    let arch_db = &mut _arch_db;

    let package = Package::new_default_empty();

    let streamlet = ir_streamlet_to_vhdl(parent_streamlet, db, arch_db, package)?;

    let streamlet_arch = streamlet.to_architecture(db, arch_db)?;

    assert_eq!(
        r#"library ieee;
use ieee.std_logic_1164.all;

library work;
use work.default.all;

entity parent_com is
  generic (
    pa : positive := 5
  );
  port (
    clk : in std_logic;
    rst : in std_logic;
    a_valid : in std_logic;
    a_ready : out std_logic;
    a_data : in std_logic_vector(4 downto 0);
    a_last : in std_logic_vector((pa + 1) - 1 downto 0);
    a_strb : in std_logic;
    b_valid : out std_logic;
    b_ready : in std_logic;
    b_data : out std_logic_vector(4 downto 0);
    b_last : out std_logic_vector((pa + 1) - 1 downto 0);
    b_strb : out std_logic
  );
end parent_com;

architecture structural of parent_com is
  signal a_0_a_valid : std_logic;
  signal a_0_a_ready : std_logic;
  signal a_0_a_data : std_logic_vector(4 downto 0);
  signal a_0_a_last : std_logic_vector((pa + 2) - 1 downto 0);
  signal a_0_a_strb : std_logic;
  signal a_0_b_valid : std_logic;
  signal a_0_b_ready : std_logic;
  signal a_0_b_data : std_logic_vector(4 downto 0);
  signal a_0_b_last : std_logic_vector((pa + 2) - 1 downto 0);
  signal a_0_b_strb : std_logic;
begin
  a: inner_com generic map(
    pa => pa + 1
  ) port map(
    clk => clk,
    rst => rst,
    a_valid => a_0_a_valid,
    a_ready => a_0_a_ready,
    a_data => a_0_a_data,
    a_last => a_0_a_last,
    a_strb => a_0_a_strb,
    b_valid => a_0_b_valid,
    b_ready => a_0_b_ready,
    b_data => a_0_b_data,
    b_last => a_0_b_last,
    b_strb => a_0_b_strb
  );
  b_valid <= a_valid;
  a_ready <= b_ready;
  b_data <= a_data;
  b_last <= a_last;
  b_strb <= a_strb;
  a_0_a_valid <= a_0_b_valid;
  a_0_b_ready <= a_0_a_ready;
  a_0_a_data <= a_0_b_data;
  a_0_a_last <= a_0_b_last;
  a_0_a_strb <= a_0_b_strb;
end structural;"#,
        streamlet_arch.declare(arch_db)?
    );

    Ok(())
}

#[test]
fn structural_comp_arch_with_instances_and_interface_params() -> Result<()> {
    let mut _db = Database::default();
    let db = &mut _db;

    let streamlet =
        structural_streamlet_with_interface_params_and_instances(db, "test", "instance")?;

    let mut _arch_db = tydi_vhdl::architecture::arch_storage::db::Database::default();
    let arch_db = &mut _arch_db;

    let package = Package::new_default_empty();

    let streamlet = ir_streamlet_to_vhdl(streamlet, db, arch_db, package)?;

    let streamlet_arch = streamlet.to_architecture(db, arch_db)?;

    assert_eq!(
        r#"library ieee;
use ieee.std_logic_1164.all;

library work;
use work.default.all;

entity test_com is
  generic (
    pa : positive := 5
  );
  port (
    clk : in std_logic;
    rst : in std_logic;
    a_valid : in std_logic;
    a_ready : out std_logic;
    a_data : in std_logic_vector(4 downto 0);
    a_last : in std_logic_vector(pa - 1 downto 0);
    a_strb : in std_logic;
    b_valid : out std_logic;
    b_ready : in std_logic;
    b_data : out std_logic_vector(4 downto 0);
    b_last : out std_logic_vector((pa + 2) - 1 downto 0);
    b_strb : out std_logic
  );
end test_com;

architecture Behaviour of test_com is
  signal first_0_a_valid : std_logic;
  signal first_0_a_ready : std_logic;
  signal first_0_a_data : std_logic_vector(4 downto 0);
  signal first_0_a_last : std_logic_vector(pa - 1 downto 0);
  signal first_0_a_strb : std_logic;
  signal first_0_b_valid : std_logic;
  signal first_0_b_ready : std_logic;
  signal first_0_b_data : std_logic_vector(4 downto 0);
  signal first_0_b_last : std_logic_vector((pa + 1) - 1 downto 0);
  signal first_0_b_strb : std_logic;
  signal second_0_a_valid : std_logic;
  signal second_0_a_ready : std_logic;
  signal second_0_a_data : std_logic_vector(4 downto 0);
  signal second_0_a_last : std_logic_vector((pa + 1) - 1 downto 0);
  signal second_0_a_strb : std_logic;
  signal second_0_b_valid : std_logic;
  signal second_0_b_ready : std_logic;
  signal second_0_b_data : std_logic_vector(4 downto 0);
  signal second_0_b_last : std_logic_vector((pa + 2) - 1 downto 0);
  signal second_0_b_strb : std_logic;
begin
  first: instance_com generic map(
    pa => pa
  ) port map(
    clk => clk,
    rst => rst,
    a_valid => first_0_a_valid,
    a_ready => first_0_a_ready,
    a_data => first_0_a_data,
    a_last => first_0_a_last,
    a_strb => first_0_a_strb,
    b_valid => first_0_b_valid,
    b_ready => first_0_b_ready,
    b_data => first_0_b_data,
    b_last => first_0_b_last,
    b_strb => first_0_b_strb
  );
  second: instance_com generic map(
    pa => pa + 1
  ) port map(
    clk => clk,
    rst => rst,
    a_valid => second_0_a_valid,
    a_ready => second_0_a_ready,
    a_data => second_0_a_data,
    a_last => second_0_a_last,
    a_strb => second_0_a_strb,
    b_valid => second_0_b_valid,
    b_ready => second_0_b_ready,
    b_data => second_0_b_data,
    b_last => second_0_b_last,
    b_strb => second_0_b_strb
  );
  first_0_a_valid <= a_valid;
  a_ready <= first_0_a_ready;
  first_0_a_data <= a_data;
  first_0_a_last <= a_last;
  first_0_a_strb <= a_strb;
  second_0_a_valid <= first_0_b_valid;
  first_0_b_ready <= second_0_a_ready;
  second_0_a_data <= first_0_b_data;
  second_0_a_last <= first_0_b_last;
  second_0_a_strb <= first_0_b_strb;
  b_valid <= second_0_b_valid;
  second_0_b_ready <= b_ready;
  b_data <= second_0_b_data;
  b_last <= second_0_b_last;
  b_strb <= second_0_b_strb;
end Behaviour;"#,
        streamlet_arch.declare(arch_db)?
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
        StreamDirection::Forward,
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
    let mut streamlet = streamlet.canonical(db, arch_db, None)?;
    let component = streamlet.to_component();
    arch_db.set_subject_component_name(Arc::new(component.vhdl_name().clone()));
    package.add_component(component);
    let package = Arc::new(package);

    arch_db.set_default_package(package);

    let streamlet_arch = streamlet.to_architecture(db, arch_db)?;

    println!("{}", arch_db.default_package().declare(arch_db)?);

    println!("{}", streamlet_arch.declare(arch_db)?);

    Ok(())
}
