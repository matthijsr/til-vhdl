use tydi_intern::Id;

use crate::{
    common::{
        logical::logicaltype::{
            stream::{Stream, Synchronicity, Throughput},
            LogicalType,
        },
        stream_direction::StreamDirection,
    },
    ir::{
        db::Database,
        generics::{behavioral::integer::IntegerGeneric, GenericParameter},
        implementation::{structure::Structure, Implementation},
        physical_properties::InterfaceDirection,
        streamlet::Streamlet,
        traits::{InternSelf, TryIntern},
        Ir,
    },
};

use tydi_common::{
    error::{Result, TryResult},
    numbers::NonNegative,
};

pub fn test_stream_id(db: &dyn Ir, data_type: impl TryIntern<LogicalType>) -> Result<Id<Stream>> {
    let data_type = data_type.try_intern(db)?;
    let null_type = LogicalType::Null.intern(db);
    Stream::try_new(
        db,
        data_type,
        1.0,
        1,
        Synchronicity::Sync,
        4,
        StreamDirection::Forward,
        null_type,
        false,
    )
}

pub fn test_stream_id_custom(
    db: &dyn Ir,
    data_type: impl TryIntern<LogicalType>,
    throughput: impl TryResult<Throughput>,
    dimensionality: NonNegative,
    complexity: NonNegative,
) -> Result<Id<Stream>> {
    let data_type = data_type.try_intern(db)?;
    let null_type = LogicalType::Null.intern(db);
    Stream::try_new(
        db,
        data_type,
        throughput,
        dimensionality,
        Synchronicity::Sync,
        complexity,
        StreamDirection::Forward,
        null_type,
        false,
    )
}

pub fn simple_structural_streamlet(db: &mut Database) -> Result<Streamlet> {
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
    Ok(streamlet)
}

pub fn simple_structural_streamlet_with_behav_params(db: &mut Database) -> Result<Streamlet> {
    let streamlet = simple_structural_streamlet(db)?;
    streamlet.with_parameters(
        db,
        vec![
            GenericParameter::try_new("pa", IntegerGeneric::natural(), 0)?,
            GenericParameter::try_new("pb", IntegerGeneric::positive(), 2)?,
            GenericParameter::try_new("pc", IntegerGeneric::integer(), -2)?,
        ],
    )
}
