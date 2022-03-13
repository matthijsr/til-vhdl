use tydi_common::{name::PathName, numbers::NonNegative};

use crate::common::{logical::logicaltype::stream::Direction, physical::complexity::Complexity};

use super::{
    elements::{ElementActivityReference, StreamElementReference},
    transfer_scope::TransferScope,
    ElementManipulatingReference,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StreamReference<F: Clone + PartialEq> {
    pub physical_stream: PathName,
    pub valid: F,
    pub ready: F,
    pub direction: Direction,
    pub complexity: Complexity,
    pub dimensionality: NonNegative,
    pub transfer_scope: TransferScope,
    pub activity: ElementActivityReference<F>,
    pub elements: Vec<StreamElementReference<F>>,
    pub user: ElementManipulatingReference<F>,
}
