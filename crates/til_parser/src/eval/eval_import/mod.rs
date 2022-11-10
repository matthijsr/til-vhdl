pub mod nodes;

use std::collections::HashMap;

use tydi_common::{error::Result, name::PathName, traits::Identify};

use crate::namespace::Namespace;

use self::nodes::{GetNamespace, IncompleteNamespaceNode, NamespaceNode, NamespaceNodeState};

use super::EvalError;

pub fn build_dependency_graph(
    namespaces: HashMap<Vec<String>, Namespace>,
) -> Result<(HashMap<PathName, NamespaceNode>, Vec<EvalError>)> {
    let mut incomplete_nodes = HashMap::new();
    let mut complete_nodes = HashMap::new();
    let mut eval_errors = vec![];

    for (name_vec, parsed_namespace) in namespaces.into_iter() {
        match PathName::try_new(name_vec) {
            Ok(namespace_name) => {
                let curr_span = parsed_namespace.name_span().clone();
                if let Some(existing_name) = incomplete_nodes.insert(
                    namespace_name.clone(),
                    IncompleteNamespaceNode {
                        name: namespace_name,
                        namespace: parsed_namespace,
                    },
                ) {
                    eval_errors.push(EvalError::new(
                        &curr_span,
                        format!(
                            "Namespace with name {} was already defined",
                            existing_name.identifier()
                        ),
                    ));
                    eval_errors.push(EvalError::new(
                        existing_name.namespace().name_span(),
                        format!("Previous definition of {}", existing_name.identifier()),
                    ));
                }
            }
            Err(err) => eval_errors.push(EvalError::new(
                parsed_namespace.name_span(),
                err.to_string(),
            )),
        }
    }

    if eval_errors.len() > 0 {
        return Ok((complete_nodes, eval_errors));
    }

    let mut node_states: HashMap<PathName, NamespaceNodeState> = HashMap::new();

    for (name, node) in incomplete_nodes.into_iter() {
        if !node_states.contains_key(&name) {
            // TODO: Grab imports, recursively resolve each dependency first
        }
    }

    Ok((complete_nodes, eval_errors))
}
