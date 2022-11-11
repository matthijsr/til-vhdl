use std::collections::BTreeMap;

use dependency_graph::Node;
use tydi_common::name::PathName;
use tydi_common::name::PathNameSelf;
use tydi_common::traits::Identify;

use crate::namespace::Namespace;
use crate::Spanned;

use super::import_stat::ImportStatement;

pub trait GetNamespace {
    fn namespace(&self) -> &Namespace;
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NamespaceNode {
    pub name: PathName,
    pub dependencies: Vec<Spanned<PathName>>,
    pub imports: Vec<Spanned<ImportStatement>>,
    pub namespace: Namespace,
}

impl GetNamespace for NamespaceNode {
    fn namespace(&self) -> &Namespace {
        &self.namespace
    }
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

impl Node for NamespaceNode {
    type DependencyType = Spanned<PathName>;

    fn dependencies(&self) -> &[Self::DependencyType] {
        &self.dependencies.as_slice()
    }

    fn matches(&self, dependency: &Self::DependencyType) -> bool {
        self.path_name() == &dependency.0
    }
}
