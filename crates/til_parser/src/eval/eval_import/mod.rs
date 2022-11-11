pub mod import_stat;
pub mod nodes;

use std::collections::{BTreeMap, HashMap};

use dependency_graph::DependencyGraph;
use tydi_common::{
    error::Result,
    name::{PathName, PathNameSelf},
    traits::Identify,
};

use crate::{
    namespace::{Import, Namespace, Statement},
    Span, Spanned,
};

use self::{
    import_stat::ImportStatement,
    nodes::{GetNamespace, NamespaceNode},
};

use super::EvalError;

pub fn resolve_dependency_graph(namespaces: Vec<Namespace>) -> Result<((), Vec<EvalError>)> {
    let mut unique_namespaces = HashMap::new();
    let mut eval_errors = vec![];

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
        return Ok(((), eval_errors));
    }

    let mut namespace_nodes = vec![];

    for (name, namespace) in unique_namespaces.iter() {
        let mut dependencies = vec![];
        let mut imports = vec![];
        for stat in namespace.stats() {
            if let (Statement::Import(parsed_import), span) = stat {
                match eval_import_stat((parsed_import, span)) {
                    Ok(import_stat) => {
                        if import_stat.0.path_name() == name {
                            eval_errors.push(EvalError::new(
                                &import_stat.1,
                                format!("Namespace {} is trying to import itself", name),
                            ))
                        } else {
                            dependencies
                                .push((import_stat.0.path_name().clone(), import_stat.1.clone()));
                            imports.push(import_stat);
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
            dependencies,
            imports,
            namespace: namespace.clone(),
        });
    }

    if eval_errors.len() > 0 {
        return Ok(((), eval_errors));
    }

    let graph = DependencyGraph::from(namespace_nodes.as_slice());
    for unresolved in graph.unresolved_dependencies() {
        println!("Unable to resolve {}", unresolved.0);
    }

    if !graph.is_internally_resolvable() {
        println!("Not internally resolvable");
    } else {
        println!("Is internally resolvable");
    }

    for namespace_node in graph {
        match namespace_node {
            dependency_graph::Step::Resolved(resolved_node) => {
                println!("Resolved {}", resolved_node.path_name())
            }
            dependency_graph::Step::Unresolved(unresolved_dep) => {
                println!("Unable to resolve {}", unresolved_dep.0)
            }
        }
    }

    Ok(((), eval_errors))
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
