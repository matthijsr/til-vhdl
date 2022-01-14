use std::{borrow::Borrow, collections::BTreeMap, convert::TryInto};

use tydi_common::{
    error::{Error, Result, TryResult},
    name::Name,
};
use tydi_intern::Id;
use tydi_vhdl::{
    architecture::{arch_storage::Arch, ArchitectureBody},
    assignment::Assign,
    declaration::ObjectDeclaration,
    port::Port,
};

use super::{
    connection::InterfaceReference, physical_properties::InterfaceDirection, Connection, GetSelf,
    Implementation, Interface, InternSelf, IntoVhdl, Ir, Streamlet,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Context {
    ports: BTreeMap<Name, Id<Interface>>,
    streamlet_instances: BTreeMap<Name, Id<Streamlet>>,
    connections: Vec<Connection>,
    implementations: BTreeMap<Name, Id<Implementation>>,
}

impl Context {
    pub fn new(ports: impl Into<BTreeMap<Name, Id<Interface>>>) -> Self {
        Context {
            ports: ports.into(),
            streamlet_instances: BTreeMap::new(),
            connections: vec![],
            implementations: BTreeMap::new(),
        }
    }

    pub fn port_ids(&self) -> &BTreeMap<Name, Id<Interface>> {
        &self.ports
    }

    pub fn ports(&self, db: &dyn Ir) -> BTreeMap<Name, Interface> {
        self.port_ids()
            .iter()
            .map(|(name, id)| (name.clone(), id.get(db)))
            .collect()
    }

    pub fn streamlet_instance_ids(&self) -> &BTreeMap<Name, Id<Streamlet>> {
        &self.streamlet_instances
    }

    pub fn streamlet_instances(&self, db: &dyn Ir) -> BTreeMap<Name, Streamlet> {
        self.streamlet_instance_ids()
            .iter()
            .map(|(name, id)| (name.clone(), id.get(db)))
            .collect()
    }

    pub fn try_add_connection(
        &mut self,
        db: &dyn Ir,
        left: impl TryResult<InterfaceReference>,
        right: impl TryResult<InterfaceReference>,
    ) -> Result<()> {
        let left = left.try_result()?;
        let right = right.try_result()?;

        struct InterfaceAndContext {
            on_streamlet: bool,
            interface: Interface,
        }

        let get_port = |i: &InterfaceReference| match i.streamlet_instance() {
            Some(streamlet_instance) => self
                .try_get_streamlet_instance(streamlet_instance)
                .and_then(|x| {
                    Ok(InterfaceAndContext {
                        on_streamlet: true,
                        interface: x.get(db).try_get_port(db, i.port())?,
                    })
                }),
            None => match self.port_ids().get(i.port()) {
                Some(port) => Ok(InterfaceAndContext {
                    on_streamlet: false,
                    interface: port.get(db),
                }),
                None => Err(Error::InvalidArgument(format!(
                    "No port with name {} exists within this context",
                    i.port()
                ))),
            },
        };
        let left_i = get_port(&left)?;
        let right_i = get_port(&right)?;
        // Interfaces are on the same layer if they both either belong to the context or to a streamlet instance
        let same_layer = left_i.on_streamlet == right_i.on_streamlet;

        if left_i.interface.stream_id() == right_i.interface.stream_id()
            // If the interfaces are on the same layer, their directions should be opposite.
            // If they are not on the same layer, their directions should be the same.
            && same_layer == (left_i.interface.direction() != right_i.interface.direction())
        {
            let (source, sink) = match left_i.interface.direction() {
                // If left_interface belongs to a streamlet instance, Out means it's a Source
                InterfaceDirection::Out if left_i.on_streamlet => (left, right),
                // Otherwise, it belongs to the context, and is a Sink
                InterfaceDirection::Out => (right, left),
                // Likewise, In means it is a Sink if left_interface is a streamlet instance
                InterfaceDirection::In if left_i.on_streamlet => (right, left),
                // But it is a Source if it belongs to the context
                InterfaceDirection::In => (left, right),
            };

            self.connections.push(Connection::new(source, sink));
            Ok(())
        } else {
            Err(Error::InvalidTarget(format!(
                "The ports {} and {} are incompatible",
                left, right
            )))
        }
    }

    pub fn try_add_streamlet_instance(
        &mut self,
        name: impl TryResult<Name>,
        streamlet: Id<Streamlet>,
    ) -> Result<()> {
        let name = name.try_result()?;
        if self.streamlet_instance_ids().contains_key(&name) {
            Err(Error::InvalidArgument(format!(
                "A streamlet instance with name {} already exists in this context",
                name
            )))
        } else {
            self.streamlet_instances.insert(name, streamlet);
            Ok(())
        }
    }

    pub fn try_get_streamlet_instance(&self, name: &Name) -> Result<Id<Streamlet>> {
        match self.streamlet_instance_ids().get(name) {
            Some(streamlet) => Ok(*streamlet),
            None => Err(Error::InvalidArgument(format!(
                "A streamlet instance with name {} does not exist in this context",
                name
            ))),
        }
    }

    pub fn connections(&self) -> &Vec<Connection> {
        &self.connections
    }

    pub fn try_into_arch_body(
        &self,
        ir_db: &dyn Ir,
        vhdl_db: &mut dyn Arch,
    ) -> Result<ArchitectureBody> {
        let mut declarations = vec![];
        let mut statements = vec![];
        let own_ports = self
            .ports(ir_db)
            .iter()
            .map(|(name, port)| Ok((name.clone(), port.canonical(ir_db, vhdl_db, "")?)))
            .collect::<Result<BTreeMap<Name, Vec<Port>>>>()?;
        let mut streamlet_ports = BTreeMap::new();
        let mut streamlet_components = BTreeMap::new();
        for (instance_name, streamlet) in self.streamlet_instances(ir_db) {
            streamlet_components.insert(
                instance_name.clone(),
                streamlet.canonical(ir_db, vhdl_db, "")?,
            );
            streamlet_ports.insert(
                instance_name,
                streamlet
                    .port_ids()
                    .iter()
                    .map(|(name, id)| {
                        Ok((name.clone(), id.get(ir_db).canonical(ir_db, vhdl_db, "")?))
                    })
                    .collect::<Result<BTreeMap<Name, Vec<Port>>>>()?,
            );
        }
        for connection in self.connections() {
            if connection.is_local_to_local() {
                let mut sink_objs = vec![];
                let mut source_objs = vec![];
                for port in own_ports.get(connection.sink().port()).unwrap() {
                    sink_objs.push(ObjectDeclaration::from_port(vhdl_db, port, true));
                }
                for port in own_ports.get(connection.source().port()).unwrap() {
                    source_objs.push(ObjectDeclaration::from_port(vhdl_db, port, true));
                }
                let sink_source: Vec<(Id<ObjectDeclaration>, Id<ObjectDeclaration>)> =
                    sink_objs.into_iter().zip(source_objs.into_iter()).collect();
                for (sink, source) in sink_source {
                    statements.push(sink.assign(vhdl_db, &source)?.into());
                }
            } else {
            }
        }

        Ok(ArchitectureBody::new(declarations, statements))
    }
}

impl From<&Streamlet> for Context {
    fn from(streamlet: &Streamlet) -> Self {
        Context::new(streamlet.port_ids().clone())
    }
}

#[cfg(test)]
mod tests {
    use tydi_vhdl::architecture::ArchitectureDeclare;

    use crate::{
        common::logical::logicaltype::{LogicalType, Stream},
        ir::{Database, TryIntern},
        test_utils::test_stream_id,
    };

    use super::*;

    #[test]
    fn try_add_connection() -> Result<()> {
        let _db = Database::default();
        let db = &_db;
        let stream = test_stream_id(db, 4)?;
        let streamlet = Streamlet::try_new(
            db,
            "a",
            vec![
                ("a", stream, InterfaceDirection::In),
                ("b", stream, InterfaceDirection::Out),
            ],
        )?;
        let mut context = Context::from(&streamlet);
        context.try_add_streamlet_instance("instance", streamlet.with_implementation(db, None))?;
        context.try_add_connection(db, "a", ("instance", "a"))?;

        Ok(())
    }

    #[test]
    fn try_get_streamlet_instance() -> Result<()> {
        let _db = Database::default();
        let db = &_db;
        let stream = test_stream_id(db, 4)?;
        let streamlet = Streamlet::try_new(
            db,
            "a",
            vec![
                ("a", stream, InterfaceDirection::In),
                ("b", stream, InterfaceDirection::Out),
            ],
        )?;
        let mut context = Context::from(&streamlet);
        context.try_add_streamlet_instance("instance", streamlet.with_implementation(db, None))?;
        context.try_get_streamlet_instance(&("instance".try_result()?))?;

        Ok(())
    }

    // Demonstrates how a Context can be used to create a structural architecture for an entity
    #[test]
    fn try_into_arch_body() -> Result<()> {
        // ...
        // Databases (this is boilerplate)
        // ...
        let _ir_db = Database::default();
        let ir_db = &_ir_db;
        let mut _vhdl_db = tydi_vhdl::architecture::arch_storage::db::Database::default();
        let vhdl_db = &mut _vhdl_db;

        // ...
        // IR
        // ...
        // Create a Union type with fields a: Bits(16) and b: Bits(7)
        let union = LogicalType::try_new_union(
            None,
            vec![("a", 16.try_intern(ir_db)?), ("b", 7.try_intern(ir_db)?)],
        )?;
        // Create a Stream node with data: Bits(4)
        let stream = test_stream_id(ir_db, 4)?;
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
        )?;
        // Create a Context from the Streamlet definition (this creates a Context with ports matching the Streamlet)
        let mut context = Context::from(&streamlet);
        // Add an instance (called "instance") of the Streamlet to the Context
        context
            .try_add_streamlet_instance("instance", streamlet.with_implementation(ir_db, None))?;
        // Connect the Context's "a" port with the instance's "a" port
        context.try_add_connection(ir_db, "a", ("instance", "a"))?;
        // Connect the Context's "b" port with the instance's "b" port
        context.try_add_connection(ir_db, "b", ("instance", "b"))?;
        // Connect the Context's "c" port with the Context's "d" port
        context.try_add_connection(ir_db, "c", "d")?;
        // Connect the instance's "c" port with the instance's "d" port
        context.try_add_connection(ir_db, ("instance", "c"), ("instance", "d"))?;

        // ..
        // Back-end
        // ..
        // Convert the Context into a VHDL architecture body
        let result = context.try_into_arch_body(ir_db, vhdl_db)?;

        // ..
        // Print the declarations and statements within the body to demonstrate the result
        println!("declarations:");
        for declaration in result.declarations(vhdl_db) {
            println!("{}", declaration.declare(vhdl_db, "", ";")?);
        }
        println!("\nstatements:");
        for statement in result.statements() {
            println!("{}", statement.declare(vhdl_db, "", ";")?);
        }

        Ok(())
    }
}
