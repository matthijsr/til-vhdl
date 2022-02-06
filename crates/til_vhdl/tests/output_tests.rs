use std::{sync::Arc, convert::TryInto};

use til_query::{
    common::logical::logicaltype::{
        stream::{Direction, Stream, Synchronicity},
        LogicalType,
    },
    ir::{
        db::Database,
        implementation::structure::Structure,
        physical_properties::InterfaceDirection,
        project::{namespace::Namespace, Project},
        streamlet::Streamlet,
        traits::GetSelf,
        Ir,
    },
};
use til_vhdl::canonical;
use tydi_common::error::Result;

extern crate til_vhdl;

#[test]
fn playground() -> Result<()> {
    let mut _db = Database::default();
    let db = &mut _db;

    let mut project = Project::new("proj", ".")?;
    let mut namespace = Namespace::new("root.sub")?;
    namespace.define_type(db, "bits", 4)?;
    namespace.define_type(db, "null", LogicalType::Null)?;
    namespace.define_type(
        db,
        "stream",
        Stream::try_new(
            db,
            namespace.get_type_id("bits")?,
            1.0,
            1,
            Synchronicity::Sync,
            4,
            Direction::Forward,
            namespace.get_type_id("null")?,
            false,
        )?,
    )?;
    namespace.define_streamlet(
        db,
        "streamlet",
        Streamlet::new().with_ports(
            db,
            vec![
                (
                    "a",
                    namespace.get_stream_id(db, "stream")?,
                    InterfaceDirection::In,
                ),
                (
                    "b",
                    namespace.get_stream_id(db, "stream")?,
                    InterfaceDirection::Out,
                ),
            ],
        )?,
    )?;

    let streamlet_id = namespace.get_streamlet_id("streamlet")?;
    let mut structure: Structure = (&streamlet_id.get(db)).try_into()?;
    structure.try_add_streamlet_instance("a", streamlet_id)?;
    structure.try_add_connection(db, ("a", "a"), "a")?;
    structure.try_add_connection(db, ("a", "b"), "b")?;
    namespace.define_implementation(db, "implementation", structure)?;

    namespace.define_streamlet(
        db,
        "implemented_streamlet",
        namespace
            .get_streamlet(db, "streamlet")?
            .with_implementation(Some(namespace.get_implementation_id("implementation")?)),
    )?;
    project.add_namespace(db, namespace)?;
    db.set_project(Arc::new(project));

    canonical(db, "../../test_output/")?;

    Ok(())
}
