use core::fmt;

use textwrap::indent;
use tydi_common::{
    error::Result,
    name::PathName,
    numbers::{NonNegative, Positive},
};
use tydi_intern::Id;

use crate::{
    common::{
        logical::{
            logicaltype::stream::{Direction, Stream, Synchronicity},
            split_streams::SplitStreams,
            type_hierarchy::TypeHierarchy,
        },
        physical::complexity::Complexity,
    },
    ir::{traits::GetSelf, Ir},
};

use super::{transfer_scope::TransferScope, ElementManipulatingReference, TypeReference};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StreamReference {
    physical_stream: PathName,
    data: Box<TypeReference>,
    direction: Direction,
    complexity: Complexity,
    dimensionality: NonNegative,
    transfer_scope: TransferScope,
    element_lanes: Positive,
    user: ElementManipulatingReference,
}

impl StreamReference {
    pub fn from_stream_id(
        db: &dyn Ir,
        stream_id: Id<Stream>,
        path_name: &PathName,
        split_streams: &SplitStreams,
        hierarchy: &TypeHierarchy,
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
            data: Box::new(TypeReference::collect_reference_from_split_streams(
                db,
                split_streams,
                hierarchy,
                path_name,
            )?),
            direction,
            complexity,
            dimensionality,
            transfer_scope,
            element_lanes,
            user,
        })
    }

    pub fn physical_stream(&self) -> &PathName {
        &self.physical_stream
    }
    pub fn data(&self) -> &Box<TypeReference> {
        &self.data
    }
    pub fn direction(&self) -> &Direction {
        &self.direction
    }
    pub fn complexity(&self) -> &Complexity {
        &self.complexity
    }
    pub fn dimensionality(&self) -> &NonNegative {
        &self.dimensionality
    }
    pub fn transfer_scope(&self) -> &TransferScope {
        &self.transfer_scope
    }
    pub fn element_lanes(&self) -> &Positive {
        &self.element_lanes
    }
    pub fn user(&self) -> &ElementManipulatingReference {
        &self.user
    }
}

impl fmt::Display for StreamReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Stream (\n{}\n)",
            indent(
                &format!(
                    r#"physical_stream: {},
data: {},
direction: {},
complexity: {},
dimensionality: {},
transfer_scope: {},
element_lanes: {},
user: {}"#,
                    self.physical_stream(),
                    self.data(),
                    self.direction(),
                    self.complexity(),
                    self.dimensionality(),
                    self.transfer_scope(),
                    self.element_lanes(),
                    self.user()
                ),
                "  "
            )
        )
    }
}
