use std::collections::BTreeMap;

use til_query::ir::{connection::InterfaceReference, GetSelf, Ir};
use tydi_common::{
    error::{Result, TryOptional},
    name::Name,
    traits::Identify,
};
use tydi_intern::Id;
use tydi_vhdl::{
    architecture::{arch_storage::Arch, ArchitectureBody},
    assignment::Assign,
    common::vhdl_name::{VhdlName, VhdlNameSelf},
    declaration::ObjectDeclaration,
    statement::PortMapping,
};

use crate::IntoVhdl;

pub(crate) type Structure = til_query::ir::implementation::structure::Structure;

impl IntoVhdl<ArchitectureBody> for Structure {
    fn canonical(
        &self,
        ir_db: &dyn Ir,
        arch_db: &mut dyn Arch,
        prefix: impl TryOptional<VhdlName>,
    ) -> Result<ArchitectureBody> {
        self.validate_connections(ir_db)?;
        let prefix = prefix.try_optional()?;

        let mut declarations = vec![];
        let mut statements = vec![];
        let own_ports = self
            .ports(ir_db)
            .iter()
            .map(|(name, port)| {
                Ok((
                    name.clone(),
                    port.canonical(ir_db, arch_db, prefix.clone())?
                        .iter()
                        .map(|vhdl_port| ObjectDeclaration::from_port(arch_db, vhdl_port, true))
                        .collect(),
                ))
            })
            .collect::<Result<BTreeMap<Name, Vec<Id<ObjectDeclaration>>>>>()?;
        let clk = ObjectDeclaration::entity_clk(arch_db);
        let rst = ObjectDeclaration::entity_rst(arch_db);
        let mut streamlet_ports = BTreeMap::new();
        let mut streamlet_components = BTreeMap::new();
        for (instance_name, streamlet) in self.streamlet_instances(ir_db) {
            let component = streamlet.canonical(ir_db, arch_db, prefix.clone())?;
            let mut port_mapping =
                PortMapping::from_component(arch_db, &component, instance_name.clone())?;
            streamlet_components.insert(instance_name.clone(), component);
            let port_map_signals = streamlet
                .port_ids()
                .iter()
                .map(|(name, id)| {
                    let ports = id.get(ir_db).canonical(ir_db, arch_db, prefix.clone())?;
                    let mut signals = vec![];
                    for port in ports {
                        let signal = ObjectDeclaration::signal(
                            arch_db,
                            format!("{}__{}", instance_name, port.identifier()),
                            port.typ().clone(),
                            None,
                        )?;
                        port_mapping.map_port(arch_db, port.vhdl_name().clone(), &signal)?;
                        signals.push(signal);
                        declarations.push(signal.into());
                    }

                    Ok((name.clone(), signals))
                })
                .collect::<Result<BTreeMap<Name, Vec<Id<ObjectDeclaration>>>>>()?;
            streamlet_ports.insert(instance_name, port_map_signals);
            port_mapping.map_port(arch_db, "clk", &clk)?;
            port_mapping.map_port(arch_db, "rst", &rst)?;
            statements.push(port_mapping.finish()?.into());
        }
        for connection in self.connections() {
            let get_objs = |interface: &InterfaceReference| -> &Vec<Id<ObjectDeclaration>> {
                match interface.streamlet_instance() {
                    Some(streamlet_instance) => streamlet_ports
                        .get(streamlet_instance)
                        .unwrap()
                        .get(interface.port())
                        .unwrap(),
                    None => own_ports.get(interface.port()).unwrap(),
                }
            };
            let sink_objs = get_objs(connection.sink()).clone();
            let source_objs = get_objs(connection.source()).clone();
            let sink_source: Vec<(Id<ObjectDeclaration>, Id<ObjectDeclaration>)> =
                sink_objs.into_iter().zip(source_objs.into_iter()).collect();
            for (sink, source) in sink_source {
                statements.push(sink.assign(arch_db, &source)?.into());
            }
        }

        Ok(ArchitectureBody::new(declarations, statements))
    }
}

#[cfg(test)]
mod tests {
    use til_query::{
        common::logical::logicaltype::LogicalType,
        ir::{
            db::Database, physical_properties::InterfaceDirection, streamlet::Streamlet, TryIntern,
        },
        test_utils::{test_stream_id, test_stream_id_custom},
    };
    use tydi_vhdl::architecture::ArchitectureDeclare;

    use super::*;

    // Demonstrates how a Structure can be used to create a structural architecture for an entity
    #[test]
    fn try_into_arch_body() -> Result<()> {
        // ...
        // Databases (this is boilerplate)
        // ...
        let mut _ir_db = Database::default();
        let ir_db = &mut _ir_db;
        let mut _arch_db = tydi_vhdl::architecture::arch_storage::db::Database::default();
        let arch_db = &mut _arch_db;

        // ...
        // IR
        // ...
        // Create a Union type with fields a: Bits(16) and b: Bits(7)
        let union = LogicalType::try_new_union(
            None,
            vec![("a", 16.try_intern(ir_db)?), ("b", 7.try_intern(ir_db)?)],
        )?;
        // Create a Stream node with data: Bits(4)
        let stream = test_stream_id_custom(ir_db, 4, 3.0, 0, 7)?;
        // Create another Stream node with data: Union(a: Bits(16), b: Bits(7))
        let stream2 = test_stream_id(ir_db, union)?;
        // Create a Streamlet
        let streamlet = Streamlet::try_new(
            ir_db,
            // The streamlet is called "a"
            "a",
            // It has ports "in: a" and "out: b", both using the Stream node as their type
            // As well as ports "in: c" and "out: d", using the other (Union) Stream node
            vec![
                ("a", stream, InterfaceDirection::In),
                ("b", stream, InterfaceDirection::Out),
                ("c", stream2, InterfaceDirection::In),
                ("d", stream2, InterfaceDirection::Out),
            ],
        )?
        .with_implementation(ir_db, None); // Streamlet does not have an implementation

        // Create a Structure from the Streamlet definition (this creates a Structure with ports matching the Streamlet)
        let mut structure = Structure::from(&streamlet.get(ir_db));
        // Add an instance (called "instance") of the Streamlet to the Structure
        structure.try_add_streamlet_instance("instance", streamlet)?;
        // Connect the Structure's "a" port with the instance's "a" port
        structure.try_add_connection(ir_db, "a", ("instance", "a"))?;
        // Connect the Structure's "b" port with the instance's "b" port
        structure.try_add_connection(ir_db, "b", ("instance", "b"))?;
        // Connect the Structure's "c" port with the Structure's "d" port
        structure.try_add_connection(ir_db, "c", "d")?;
        // Connect the instance's "c" port with the instance's "d" port
        structure.try_add_connection(ir_db, ("instance", "c"), ("instance", "d"))?;

        structure.try_add_streamlet_instance("b", streamlet)?;
        structure.try_add_connection(ir_db, ("b", "a"), ("b", "b"))?;
        structure.try_add_connection(ir_db, ("b", "c"), ("b", "d"))?;

        // ..
        // Back-end
        // ..
        // Convert the Structure into a VHDL architecture body
        let result = structure.canonical(ir_db, arch_db, None)?;

        let mut declarations = String::new();
        for declaration in result.declarations() {
            declarations.push_str(&format!("{}\n", declaration.declare(arch_db)?));
        }
        assert_eq!(
            r#"signal b__a_valid : std_logic
signal b__a_ready : std_logic
signal b__a_data : std_logic_vector(11 downto 0)
signal b__a_stai : std_logic_vector(1 downto 0)
signal b__a_endi : std_logic_vector(1 downto 0)
signal b__a_strb : std_logic_vector(2 downto 0)
signal b__b_valid : std_logic
signal b__b_ready : std_logic
signal b__b_data : std_logic_vector(11 downto 0)
signal b__b_stai : std_logic_vector(1 downto 0)
signal b__b_endi : std_logic_vector(1 downto 0)
signal b__b_strb : std_logic_vector(2 downto 0)
signal b__c_valid : std_logic
signal b__c_ready : std_logic
signal b__c_data : std_logic_vector(16 downto 0)
signal b__c_last : std_logic
signal b__c_strb : std_logic
signal b__d_valid : std_logic
signal b__d_ready : std_logic
signal b__d_data : std_logic_vector(16 downto 0)
signal b__d_last : std_logic
signal b__d_strb : std_logic
signal instance__a_valid : std_logic
signal instance__a_ready : std_logic
signal instance__a_data : std_logic_vector(11 downto 0)
signal instance__a_stai : std_logic_vector(1 downto 0)
signal instance__a_endi : std_logic_vector(1 downto 0)
signal instance__a_strb : std_logic_vector(2 downto 0)
signal instance__b_valid : std_logic
signal instance__b_ready : std_logic
signal instance__b_data : std_logic_vector(11 downto 0)
signal instance__b_stai : std_logic_vector(1 downto 0)
signal instance__b_endi : std_logic_vector(1 downto 0)
signal instance__b_strb : std_logic_vector(2 downto 0)
signal instance__c_valid : std_logic
signal instance__c_ready : std_logic
signal instance__c_data : std_logic_vector(16 downto 0)
signal instance__c_last : std_logic
signal instance__c_strb : std_logic
signal instance__d_valid : std_logic
signal instance__d_ready : std_logic
signal instance__d_data : std_logic_vector(16 downto 0)
signal instance__d_last : std_logic
signal instance__d_strb : std_logic
"#,
            declarations,
            "declarations"
        );

        let mut statements = String::new();
        for statement in result.statements() {
            statements.push_str(&format!("{}\n", statement.declare(arch_db)?));
        }
        assert_eq!(
            r#"b: a_com port map(
  clk => clk,
  rst => rst,
  a_valid => b__a_valid,
  a_ready => b__a_ready,
  a_data => b__a_data,
  a_stai => b__a_stai,
  a_endi => b__a_endi,
  a_strb => b__a_strb,
  c_valid => b__c_valid,
  c_ready => b__c_ready,
  c_data => b__c_data,
  c_last => b__c_last,
  c_strb => b__c_strb,
  b_valid => b__b_valid,
  b_ready => b__b_ready,
  b_data => b__b_data,
  b_stai => b__b_stai,
  b_endi => b__b_endi,
  b_strb => b__b_strb,
  d_valid => b__d_valid,
  d_ready => b__d_ready,
  d_data => b__d_data,
  d_last => b__d_last,
  d_strb => b__d_strb
)
instance: a_com port map(
  clk => clk,
  rst => rst,
  a_valid => instance__a_valid,
  a_ready => instance__a_ready,
  a_data => instance__a_data,
  a_stai => instance__a_stai,
  a_endi => instance__a_endi,
  a_strb => instance__a_strb,
  c_valid => instance__c_valid,
  c_ready => instance__c_ready,
  c_data => instance__c_data,
  c_last => instance__c_last,
  c_strb => instance__c_strb,
  b_valid => instance__b_valid,
  b_ready => instance__b_ready,
  b_data => instance__b_data,
  b_stai => instance__b_stai,
  b_endi => instance__b_endi,
  b_strb => instance__b_strb,
  d_valid => instance__d_valid,
  d_ready => instance__d_ready,
  d_data => instance__d_data,
  d_last => instance__d_last,
  d_strb => instance__d_strb
)
instance__a_valid <= a_valid
instance__a_ready <= a_ready
instance__a_data <= a_data
instance__a_stai <= a_stai
instance__a_endi <= a_endi
instance__a_strb <= a_strb
b_valid <= instance__b_valid
b_ready <= instance__b_ready
b_data <= instance__b_data
b_stai <= instance__b_stai
b_endi <= instance__b_endi
b_strb <= instance__b_strb
d_valid <= c_valid
d_ready <= c_ready
d_data <= c_data
d_last <= c_last
d_strb <= c_strb
instance__c_valid <= instance__d_valid
instance__c_ready <= instance__d_ready
instance__c_data <= instance__d_data
instance__c_last <= instance__d_last
instance__c_strb <= instance__d_strb
b__a_valid <= b__b_valid
b__a_ready <= b__b_ready
b__a_data <= b__b_data
b__a_stai <= b__b_stai
b__a_endi <= b__b_endi
b__a_strb <= b__b_strb
b__c_valid <= b__d_valid
b__c_ready <= b__d_ready
b__c_data <= b__d_data
b__c_last <= b__d_last
b__c_strb <= b__d_strb
"#,
            statements,
            "statements"
        );

        Ok(())
    }
}