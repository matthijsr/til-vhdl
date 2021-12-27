use til_vhdl::{
    common::logical::logicaltype::{Direction, Synchronicity},
    ir::{
        physical_properties::PortDirection, Database, Implementation, InternSelf, Ir, LogicalType,
        PhysicalProperties, Port, Stream, Streamlet,
    },
};
use tydi_common::error::{Error, Result};

extern crate til_vhdl;

#[test]
fn streamlet_new() -> Result<()> {
    let db = Database::default();
    let imple = Implementation::Link;
    let implid = db.intern_implementation(imple.clone());
    let streamlet = Streamlet::try_new(&db, vec![])?.with_implementation(&db, Some(implid));
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
    let _vhdl_db = tydi_vhdl::architecture::arch_storage::db::Database::default();
    let vhdl_db = &_vhdl_db;
    let data_type = LogicalType::try_new_bits(4)?.intern(db);
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
    let port = Port::try_new("a", stream, PhysicalProperties::new(PortDirection::Sink))?;
    let streamlet = Streamlet::try_new(db, vec![port]);

    Ok(())
}
