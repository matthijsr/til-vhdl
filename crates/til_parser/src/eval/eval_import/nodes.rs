use std::collections::BTreeMap;

use tydi_common::name::PathName;
use tydi_common::name::PathNameSelf;
use tydi_common::traits::Identify;

use crate::namespace::Namespace;

pub trait GetNamespace {
    fn namespace(&self) -> &Namespace;
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum NamespaceNodeState {
    /// Initial state of the node
    Incomplete(IncompleteNamespaceNode),
    /// Full node (no dependency issues)
    Node(NamespaceNode),
    /// This Namespace had dependency errors (prevents any dependents from going deeper)
    ErrorNode(IncompleteNamespaceNode),
}

impl GetNamespace for NamespaceNodeState {
    fn namespace(&self) -> &Namespace {
        match &self {
            NamespaceNodeState::Incomplete(node) => node.namespace(),
            NamespaceNodeState::Node(node) => node.namespace(),
            NamespaceNodeState::ErrorNode(node) => node.namespace(),
        }
    }
}

impl PathNameSelf for NamespaceNodeState {
    fn path_name(&self) -> &PathName {
        match &self {
            NamespaceNodeState::Incomplete(node) => node.path_name(),
            NamespaceNodeState::Node(node) => node.path_name(),
            NamespaceNodeState::ErrorNode(node) => node.path_name(),
        }
    }
}

impl Identify for NamespaceNodeState {
    fn identifier(&self) -> String {
        self.path_name().to_string()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct IncompleteNamespaceNode {
    pub name: PathName,
    pub namespace: Namespace,
}

impl GetNamespace for IncompleteNamespaceNode {
    fn namespace(&self) -> &Namespace {
        &self.namespace
    }
}

impl PathNameSelf for IncompleteNamespaceNode {
    fn path_name(&self) -> &PathName {
        &self.name
    }
}

impl Identify for IncompleteNamespaceNode {
    fn identifier(&self) -> String {
        self.path_name().to_string()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NamespaceNode {
    pub name: PathName,
    pub imports: BTreeMap<PathName, NamespaceNode>,
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
