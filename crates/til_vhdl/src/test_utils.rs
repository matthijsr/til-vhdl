use tydi_intern::Id;

use crate::{
    common::logical::logicaltype::{Direction, LogicalType, Stream, Synchronicity},
    ir::{InternSelf, Ir},
};

use tydi_common::error::Result;

pub fn test_stream_id(db: &dyn Ir) -> Result<Id<Stream>> {
    let data_type = LogicalType::try_new_bits(4)?.intern(db);
    let null_type = LogicalType::Null.intern(db);
    Stream::try_new(
        db,
        data_type,
        "1.0",
        1,
        Synchronicity::Sync,
        4,
        Direction::Forward,
        null_type,
        false,
    )
}
