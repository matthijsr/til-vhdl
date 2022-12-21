use tydi_intern::Id;

use crate::{
    common::{
        logical::logicaltype::{
            genericproperty::GenericProperty,
            stream::{Stream, Synchronicity, Throughput},
            LogicalType,
        },
        stream_direction::StreamDirection,
    },
    ir::{
        db::Database,
        generics::{
            behavioral::integer::IntegerGeneric, interface::dimensionality::DimensionalityGeneric,
            GenericParameter,
        },
        implementation::{structure::Structure, Implementation},
        physical_properties::InterfaceDirection,
        streamlet::Streamlet,
        traits::{InternSelf, TryIntern},
        Ir,
    },
};

use tydi_common::{
    error::{Result, TryResult},
    name::{Name, PathName},
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

pub fn simple_structural_streamlet(
    db: &mut Database,
    name: impl TryResult<PathName>,
) -> Result<Streamlet> {
    let streamlet = streamlet_without_impl(db, name)?;
    let mut structure = Structure::try_from(&streamlet)?;
    structure.try_add_connection(db, "a", "b")?;
    let implementation = Implementation::structural(structure)?
        .try_with_name("structural")?
        .intern(db);
    let streamlet = streamlet.with_implementation(Some(implementation));
    Ok(streamlet)
}

pub fn streamlet_without_impl(
    db: &mut Database,
    name: impl TryResult<PathName>,
) -> Result<Streamlet> {
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
    let streamlet = Streamlet::new().try_with_name(name)?.with_ports(
        db,
        vec![
            ("a", stream, InterfaceDirection::In),
            ("b", stream, InterfaceDirection::Out),
        ],
    )?;
    Ok(streamlet)
}

pub fn streamlet_without_impl_with_behav_params(
    db: &mut Database,
    name: impl TryResult<PathName>,
) -> Result<Streamlet> {
    let streamlet = streamlet_without_impl(db, name)?;
    streamlet.with_parameters(
        db,
        vec![
            GenericParameter::try_new("pa", IntegerGeneric::natural(), 0)?,
            GenericParameter::try_new("pb", IntegerGeneric::positive(), 2)?,
            GenericParameter::try_new("pc", IntegerGeneric::integer(), -2)?,
        ],
    )
}

pub fn simple_structural_streamlet_with_behav_params(
    db: &mut Database,
    name: impl TryResult<PathName>,
) -> Result<Streamlet> {
    let streamlet = simple_structural_streamlet(db, name)?;
    streamlet.with_parameters(
        db,
        vec![
            GenericParameter::try_new("pa", IntegerGeneric::natural(), 0)?,
            GenericParameter::try_new("pb", IntegerGeneric::positive(), 2)?,
            GenericParameter::try_new("pc", IntegerGeneric::integer(), -2)?,
        ],
    )
}

pub fn simple_streamlet_with_interface_params(
    db: &mut Database,
    name: impl TryResult<PathName>,
) -> Result<Streamlet> {
    let bits = LogicalType::try_new_bits(4)?.intern(db);
    let data_type = LogicalType::try_new_union(None, vec![("a", bits), ("b", bits)])?.intern(db);
    let null_type = LogicalType::null_id(db);
    let generic_dim_param: GenericProperty<NonNegative> = Name::try_new("pa")?.into();
    let stream = Stream::try_new(
        db,
        data_type,
        1.0,
        generic_dim_param + GenericProperty::Fixed(1),
        Synchronicity::Sync,
        4,
        StreamDirection::Forward,
        null_type,
        false,
    )?;
    let streamlet = Streamlet::new().try_with_name(name)?.with_ports(
        db,
        vec![
            ("a", stream, InterfaceDirection::In),
            ("b", stream, InterfaceDirection::Out),
        ],
    )?;
    streamlet.with_parameters(
        db,
        vec![GenericParameter::try_new(
            "pa",
            DimensionalityGeneric::new(),
            5,
        )?],
    )
}
