use tydi_common::{error::Result, name::PathName, numbers::{NonNegative, Positive}};
use tydi_intern::Id;

use crate::{
    common::{
        logical::logicaltype::stream::{Direction, Stream, Synchronicity},
        physical::complexity::Complexity,
    },
    ir::{traits::GetSelf, Ir},
};

use super::{transfer_scope::TransferScope, ElementManipulatingReference};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StreamReference {
    pub physical_stream: PathName,
    pub direction: Direction,
    pub complexity: Complexity,
    pub dimensionality: NonNegative,
    pub transfer_scope: TransferScope,
    pub element_lanes: Positive,
    pub user: ElementManipulatingReference,
}

impl StreamReference {
    pub fn from_stream_id(
        db: &dyn Ir,
        stream_id: Id<Stream>,
        path_name: &PathName,
    ) -> Result<Self> {
        let stream = stream_id.get(db);
        let physical_stream = path_name.clone();
        let direction = stream.direction();
        let complexity = stream.complexity();
        let dimensionality = stream.dimensionality();
        let transfer_scope = if path_name.is_empty() {
            TransferScope::Root
        } else {
            match stream.synchronicity() {
                Synchronicity::Sync | Synchronicity::Flatten => {
                    TransferScope::Sync(path_name.root())
                }
                Synchronicity::Desync | Synchronicity::FlatDesync => TransferScope::Root,
            }
        };
        let element_lanes = stream.throughput().positive();
        let user = ElementManipulatingReference::from_logical_type_id(db, stream.user_id())?;

        Ok(StreamReference {
            physical_stream,
            direction,
            complexity,
            dimensionality,
            transfer_scope,
            element_lanes,
            user,
        })
    }
}
