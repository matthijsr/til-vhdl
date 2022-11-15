pub mod import_stat;
pub mod nodes;

use std::collections::{BTreeMap, HashMap};

use petgraph::{prelude::DiGraph, Graph};
use tydi_common::{
    error::Result,
    name::{PathName, PathNameSelf},
};

use crate::{
    namespace::{Import, Namespace, Statement},
    Span, Spanned,
};

use self::{import_stat::ImportStatement, nodes::NamespaceNode};

use super::EvalError;

pub fn build_dependency_graph(
    namespaces: Vec<Namespace>,
    eval_errors: &mut Vec<EvalError>,
) -> Result<DiGraph<NamespaceNode, ()>> {
    let mut di_graph: DiGraph<NamespaceNode, ()> = Graph::new();
    let mut unique_namespaces = HashMap::new();

    for parsed_namespace in namespaces.into_iter() {
        match PathName::try_new(parsed_namespace.name()) {
            Ok(namespace_name) => {
                let curr_span = parsed_namespace.name_span().clone();
                if let Some(existing_name) =
                    unique_namespaces.insert(namespace_name.clone(), parsed_namespace)
                {
                    eval_errors.push(EvalError::new(
                        &curr_span,
                        format!(
                            "Namespace with name {} was already defined",
                            &namespace_name,
                        ),
                    ));
                    eval_errors.push(EvalError::new(
                        existing_name.name_span(),
                        format!("Previous definition of {}", &namespace_name),
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
        return Ok(di_graph);
    }

    let mut namespace_nodes = vec![];

    for (name, namespace) in unique_namespaces.iter() {
        let mut imports: BTreeMap<PathName, Vec<Spanned<ImportStatement>>> = BTreeMap::new();
        for stat in namespace.stats() {
            if let (Statement::Import(parsed_import), span) = stat {
                match eval_import_stat((parsed_import, span)) {
                    Ok(import_stat) => {
                        if import_stat.0.path_name() == name {
                            eval_errors.push(EvalError::new(
                                &import_stat.1,
                                format!("Namespace {} is trying to import itself", name),
                            ))
                        } else if !unique_namespaces.contains_key(import_stat.0.path_name()) {
                            eval_errors.push(EvalError::new(
                                &import_stat.1,
                                format!("Namespace {} does not exist", import_stat.0.path_name()),
                            ))
                        } else {
                            match imports.get_mut(import_stat.0.path_name()) {
                                Some(import_stats) => import_stats.push(import_stat),
                                None => {
                                    imports.insert(
                                        import_stat.0.path_name().clone(),
                                        vec![import_stat],
                                    );
                                }
                            }
                        }
                    }
                    Err(eval_error) => {
                        eval_errors.push(eval_error);
                    }
                }
            }
        }
        namespace_nodes.push(NamespaceNode {
            name: name.clone(),
            imports,
            namespace: namespace.clone(),
        });
    }

    if eval_errors.len() > 0 {
        return Ok(di_graph);
    }

    let mut node_ids = HashMap::new();

    // Add initial nodes
    for node in namespace_nodes.into_iter() {
        let key = node.path_name().clone();
        let node_id = di_graph.add_node(node);
        node_ids.insert(key, node_id);
    }

    // Create edges between nodes (no weight)
    for node_idx in di_graph.node_indices() {
        let deps = di_graph[node_idx]
            .imports
            .keys()
            .map(|dep| *node_ids.get(dep).unwrap())
            .collect::<Vec<_>>();
        for dep in deps {
            di_graph.add_edge(node_idx, dep, ());
        }
    }

    Ok(di_graph)
}

pub fn eval_import_stat(
    import_stat: (&Import, &Span),
) -> std::result::Result<Spanned<ImportStatement>, EvalError> {
    match &import_stat.0 {
        Import::FullImport((import_name, import_span)) => {
            match PathName::try_new(import_name.iter().map(|(n, _)| n)) {
                Ok(name) => Ok((ImportStatement::Full(name), import_span.clone())),
                Err(err) => Err(EvalError::new(import_span, err.to_string())),
            }
        }
    }
}
