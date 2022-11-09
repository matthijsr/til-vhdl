use std::convert::TryInto;

use til_parser::project::into_query_storage;
use til_query::{
    common::{
        logical::logicaltype::{
            stream::{Stream, Synchronicity},
            LogicalType,
        },
        stream_direction::StreamDirection,
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

fn parse_to_output(file_path: &str, name: &str) -> Result<()> {
    let db = into_query_storage(
        format!(
            r#"name = "{}"

files = ["{}"]
 
output_path = "../../test_output/""#,
            name, file_path
        ),
        ".",
    )?;

    canonical(&db)
}

#[test]
fn from_til_parse() -> Result<()> {
    parse_to_output("tests/til_files/test_nspace.til", "test_nspace")
}

#[test]
fn test_import() -> Result<()> {
    parse_to_output("tests/til_files/test_import.til", "test_import")
}

#[test]
fn evaluation_parse() -> Result<()> {
    parse_to_output("tests/til_files/evaluation.til", "evaluation")
}

#[test]
fn playground() -> Result<()> {
    let mut _db = Database::default();
    let db = &mut _db;

    let mut project = Project::new("playground", ".", Some("../../test_output/"))?;
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
            StreamDirection::Forward,
            namespace.get_type_id("null")?,
            false,
        )?,
    )?;
    namespace.define_streamlet(
        db,
        "streamlet",
        Streamlet::new().with_domains_ports(
            db,
            vec!["domain1", "domain2", "domain3"],
            vec![
                (
                    "a",
                    namespace.get_stream_id(db, "stream")?,
                    ("domain1", InterfaceDirection::In),
                ),
                (
                    "b",
                    namespace.get_stream_id(db, "stream")?,
                    ("domain2", InterfaceDirection::Out),
                ),
            ],
        )?,
    )?;

    let streamlet_id = namespace.get_streamlet_id("streamlet")?;
    let mut structure: Structure = streamlet_id.get(db).as_ref().try_into()?;
    structure.try_add_streamlet_instance(
        db,
        "a",
        streamlet_id,
        vec![
            ("", "domain1"),
            ("domain2", "domain2"),
            ("domain3", "domain1"),
        ],
    )?;
    structure.try_add_connection(db, ("a", "a"), "a")?;
    structure.try_add_connection(db, ("a", "b"), "b")?;
    namespace.define_implementation(db, "implementation", structure)?;

    namespace.define_streamlet(
        db,
        "implemented_streamlet",
        namespace
            .get_streamlet(db, "streamlet")?
            .as_ref()
            .clone()
            .with_implementation(Some(namespace.get_implementation_id("implementation")?)),
    )?;
    project.add_namespace(db, namespace)?;
    db.set_project(project);

    canonical(db)?;

    Ok(())
}
