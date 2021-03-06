use std::{collections::HashMap, sync::Arc};

use chumsky::{Parser, Stream};
use til_query::ir::{
    db::Database,
    project::{namespace::Namespace, Project},
    Ir,
};
use tydi_common::{error::Error, name::PathName};

use crate::{
    eval::{eval_decl::eval_declaration, EvalError},
    lex::lexer,
    namespace::{namespaces_parser, Statement},
    report::{report_errors, report_eval_errors},
};

pub fn into_query_storage(src: impl Into<String>) -> tydi_common::error::Result<Database> {
    let mut _db = Database::default();
    let db = &mut _db;

    let src = src.into();
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
    let mut project = Project::new("proj", ".")?;
    if let Some(ast) = ast {
        for (name_vec, parsed_namespace) in ast.into_iter() {
            match PathName::try_new(name_vec) {
                Ok(namespace_name) => {
                    // TODO: Imports currently left immutable as they are unused.
                    let mut types = HashMap::new();
                    let type_imports = HashMap::new();
                    let mut interfaces = HashMap::new();
                    let interface_imports = HashMap::new();
                    let mut implementations = HashMap::new();
                    let implementation_imports = HashMap::new();
                    let mut streamlets = HashMap::new();
                    let streamlet_imports = HashMap::new();
                    for stat in parsed_namespace.stats().iter() {
                        match &stat.0 {
                            Statement::Import => {
                                // TODO
                                eval_errors.push(EvalError::new(
                                    &stat.1,
                                    "Imports are not currently supported",
                                ));
                            }
                            Statement::Decl(decl) => {
                                let eval_result = eval_declaration(
                                    db,
                                    decl,
                                    &namespace_name,
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
                    }

                    // Don't bother doing more work if evaluation failed at any point, just use the errors to provide a useful report.
                    if eval_errors.len() == 0 {
                        let mut namespace = Namespace::new(namespace_name)?;

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

                        project.add_namespace(db, namespace)?;
                    }
                }
                Err(err) => eval_errors.push(EvalError::new(
                    parsed_namespace.name_span(),
                    err.to_string(),
                )),
            }
        }
    }

    if eval_errors.len() > 0 {
        report_eval_errors(&src, eval_errors.clone());
        return Err(Error::ProjectError(
            "Errors during evaluation, see report.".to_string(),
        ));
    }

    db.set_project(Arc::new(project));

    Ok(_db)
}
