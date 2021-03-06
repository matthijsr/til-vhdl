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
