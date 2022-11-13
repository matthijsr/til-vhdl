use std::{collections::HashMap, path::PathBuf};

use chumsky::{Parser, Stream};
use petgraph::algo::toposort;
use til_query::ir::{
    db::Database,
    project::{namespace::Namespace, Project},
    traits::GetSelf,
    Ir,
};
use tydi_common::{
    error::{Error, TryResult},
    name::{PathName, PathNameSelf},
};

use crate::{
    eval::{eval_decl::eval_declaration, eval_import::build_dependency_graph, EvalError},
    lex::lexer,
    namespace::{namespaces_parser, Statement},
    report::{report_errors, report_eval_errors},
};

pub fn into_query_storage_default(src: impl Into<String>) -> tydi_common::error::Result<Database> {
    let mut db = Database::default();
    db.set_project(Project::new("proj", ".", None::<&str>)?);

    file_to_project(src, &mut db, ".")?;

    Ok(db)
}

pub fn into_query_storage_default_with_output(
    src: impl Into<String>,
    output_path: impl TryResult<PathBuf>,
) -> tydi_common::error::Result<Database> {
    let mut db = Database::default();
    db.set_project(Project::new("proj", ".", Some(output_path))?);

    file_to_project(src, &mut db, ".")?;

    Ok(db)
}

pub fn into_query_storage(
    src: impl Into<String>,
    project: impl TryResult<Project>,
    link_root: impl TryResult<PathBuf>,
) -> tydi_common::error::Result<Database> {
    let mut db = Database::default();
    db.set_project(project.try_result()?);

    file_to_project(src, &mut db, link_root)?;

    Ok(db)
}

pub fn file_to_project(
    src: impl Into<String>,
    db: &mut Database,
    link_root: impl TryResult<PathBuf>,
) -> tydi_common::error::Result<()> {
    let src = src.into();
    let link_root = link_root.try_result()?;
    let (tokens, errs) = lexer().parse_recovery(src.as_str());
    let (ast, parse_errs) = if let Some(tokens) = tokens {
        let len = src.chars().count();
        let (ast, parse_errs) =
            namespaces_parser().parse_recovery(Stream::from_iter(len..len + 1, tokens.into_iter()));

        (ast, parse_errs)
    } else {
        (None, Vec::new())
    };
    if errs.len() > 0 || parse_errs.len() > 0 {
        report_errors(&src, errs, parse_errs);
        return Err(Error::ParsingError(
            "Errors during parsing, see report.".to_string(),
        ));
    }
    let mut eval_errors = vec![];

    if let Some(ast) = ast {
        let di_graph = build_dependency_graph(ast, &mut eval_errors)?;

        if eval_errors.len() > 0 {
            report_eval_errors(&src, eval_errors.clone());
            return Err(Error::ProjectError(
                "Errors while attempting to resolve imports, see report.".to_string(),
            ));
        }

        let namespace_nodes = match toposort(&di_graph, None) {
            Ok(sorted_nodes) => Ok(sorted_nodes),
            Err(cycle) => Err(Error::ProjectError(format!(
                "Import error, namespace {} has a cyclical dependency.",
                di_graph[cycle.node_id()].path_name()
            ))),
        }?
        .into_iter()
        .rev()
        .map(|idx| di_graph.node_weight(idx).unwrap());

        for namespace_node in namespace_nodes {
            let mut type_imports = HashMap::new();
            let mut interface_imports = HashMap::new();
            let mut implementation_imports = HashMap::new();
            let mut streamlet_imports = HashMap::new();
            for (imported_space, _) in namespace_node.imports() {
                let imported_space = db.project().namespaces().try_get(imported_space)?.get(db);
                for (name, id) in imported_space.type_ids() {
                    type_imports.insert(imported_space.path_name().with_child(name), *id);
                }
                for (name, id) in imported_space.interface_ids() {
                    interface_imports.insert(imported_space.path_name().with_child(name), *id);
                }
                for (name, id) in imported_space.implementation_ids() {
                    implementation_imports.insert(imported_space.path_name().with_child(name), *id);
                }
                for (name, id) in imported_space.streamlet_ids() {
                    streamlet_imports.insert(imported_space.path_name().with_child(name), *id);
                }
            }
            // TODO: Imports currently left immutable as they are unused.
            let mut types = HashMap::new();
            let mut interfaces = HashMap::new();
            let mut implementations = HashMap::new();
            let mut streamlets = HashMap::new();
            for stat in namespace_node.namespace.stats().iter() {
                if let Statement::Decl(decl) = &stat.0 {
                    let eval_result = eval_declaration(
                        db,
                        &link_root,
                        decl,
                        namespace_node.path_name(),
                        &mut streamlets,
                        &streamlet_imports,
                        &mut implementations,
                        &implementation_imports,
                        &mut interfaces,
                        &interface_imports,
                        &mut types,
                        &type_imports,
                    );

                    if let Err(err) = eval_result {
                        eval_errors.push(err);
                    }
                }
            }

            // Don't bother doing more work if evaluation failed at any point, just use the errors to provide a useful report.
            if eval_errors.len() == 0 {
                let mut namespace = Namespace::new(namespace_node.path_name().clone())?;

                for (name, type_id) in types {
                    namespace.import_type(name, type_id)?;
                }
                for (name, interface_id) in interfaces {
                    namespace.import_interface(name, interface_id)?;
                }
                for (name, implementation_id) in implementations {
                    namespace.import_implementation(name, implementation_id)?;
                }
                for (name, streamlet_id) in streamlets {
                    namespace.import_streamlet(name, streamlet_id)?;
                }

                let mut project = db.project();
                project.add_namespace(db, namespace)?;
                db.set_project(project);
            }
        }
    }
    if eval_errors.len() > 0 {
        report_eval_errors(&src, eval_errors.clone());
        return Err(Error::ProjectError(
            "Errors during evaluation, see report.".to_string(),
        ));
    }

    Ok(())
}
