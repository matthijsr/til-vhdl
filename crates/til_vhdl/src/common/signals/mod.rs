use std::sync::Arc;

use crate::ir::streamlet::PhysicalStreamObject;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PhysicalStreamProcess {
    stream_object: Arc<PhysicalStreamObject>,
}
