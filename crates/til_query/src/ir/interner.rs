use std::sync::Arc;

use tydi_common::{map::InsertionOrderedMap, name::Name};
use tydi_intern::Id;

use crate::common::logical::logicaltype::{stream::Stream, LogicalType};

use super::{
    implementation::{structure::streamlet_instance::GenericParameterAssignment, Implementation},
    project::{interface::Interface, namespace::Namespace},
    streamlet::Streamlet,
};

#[salsa::query_group(InternerStorage)]
pub trait Interner {
    #[salsa::interned]
    fn intern_namespace(&self, namespace: Namespace) -> Id<Namespace>;
    #[salsa::interned]
    fn intern_implementation(&self, implementation: Implementation) -> Id<Implementation>;
    #[salsa::interned]
    fn intern_type(&self, logical_type: LogicalType) -> Id<LogicalType>;
    #[salsa::interned]
    fn intern_stream(&self, stream: Stream) -> Id<Stream>;
    #[salsa::interned]
    fn intern_streamlet(&self, streamlet: Arc<Streamlet>) -> Id<Arc<Streamlet>>;
    #[salsa::interned]
    fn intern_interface(&self, interface: Arc<Interface>) -> Id<Arc<Interface>>;
    #[salsa::interned]
    fn intern_param_assignments(
        &self,
        param_assignments: Arc<InsertionOrderedMap<Name, GenericParameterAssignment>>,
    ) -> Id<Arc<InsertionOrderedMap<Name, GenericParameterAssignment>>>;
}
