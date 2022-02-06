use tydi_intern::Id;

use crate::common::logical::logicaltype::{stream::Stream, LogicalType};

use super::{
    implementation::Implementation,
    interface::Interface,
    project::{interface_collection::InterfaceCollection, namespace::Namespace},
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
    fn intern_port(&self, logical_type: Interface) -> Id<Interface>;
    #[salsa::interned]
    fn intern_stream(&self, stream: Stream) -> Id<Stream>;
    #[salsa::interned]
    fn intern_streamlet(&self, streamlet: Streamlet) -> Id<Streamlet>;
    #[salsa::interned]
    fn intern_interface_collection(
        &self,
        interface: InterfaceCollection,
    ) -> Id<InterfaceCollection>;
}
