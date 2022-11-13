use std::collections::BTreeMap;

use tydi_common::name::PathName;
use tydi_common::name::PathNameSelf;
use tydi_common::traits::Identify;

use crate::namespace::Namespace;
use crate::Spanned;

use super::import_stat::ImportStatement;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NamespaceNode {
    pub name: PathName,
    pub imports: BTreeMap<PathName, Vec<Spanned<ImportStatement>>>,
    pub namespace: Namespace,
}

impl PathNameSelf for NamespaceNode {
    fn path_name(&self) -> &PathName {
        &self.name
    }
}

impl Identify for NamespaceNode {
    fn identifier(&self) -> String {
        self.path_name().to_string()
    }
}
