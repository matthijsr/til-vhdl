use std::collections::BTreeMap;

use tydi_common::{
    error::{Result, TryResult},
    name::{Name, PathName, PathNameSelf},
    traits::Identify,
};
use tydi_intern::Id;

use crate::{common::logical::logicaltype::LogicalType, ir::streamlet::Streamlet};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Namespace {
    /// The name of the Namespace within its parent project
    name: PathName,
    /// The types declared within the namespace.
    /// Names are purely for tracking, and do not affect type equivalence.
    types: BTreeMap<Name, Id<LogicalType>>,
    /// The streamlets declared within the namespace.
    streamlets: BTreeMap<Name, Id<Streamlet>>,
    /// The implementations declared within the namespace.
    implementations: BTreeMap<Name, Id<Streamlet>>,
    /// Imported names, structured as:
    /// Key: Name of the imported type, streamlet or implementation.
    ///      Allows for PathNames to disambiguate between overlapping names.
    /// Value: Source of the external dependency.
    imports: BTreeMap<PathName, PathName>,
}

impl Namespace {
    pub fn new(name: impl TryResult<PathName>) -> Result<Self> {
        Ok(Namespace {
            name: name.try_result()?,
            types: BTreeMap::new(),
            streamlets: BTreeMap::new(),
            implementations: BTreeMap::new(),
            imports: BTreeMap::new(),
        })
    }
}

impl Identify for Namespace {
    fn identifier(&self) -> String {
        self.path_name().to_string()
    }
}

impl PathNameSelf for Namespace {
    fn path_name(&self) -> &PathName {
        &self.name
    }
}
