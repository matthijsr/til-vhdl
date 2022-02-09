use std::{collections::HashMap, sync::Arc};

use chumsky::{Parser, Stream};
use til_query::{
    common::logical::logicaltype::LogicalType,
    ir::{
        db::Database,
        project::{
            namespace::{self, Namespace},
            Project,
        },
        Ir,
    },
};
use tydi_common::{error::Error, name::PathName};
use tydi_intern::Id;

use crate::{
    eval::{eval_decl::eval_declaration, eval_name, EvalError},
    expr::Expr,
    ident_expr::IdentExpr,
    lex::lexer,
    namespace::{namespaces_parser, Decl, Statement},
    report::report_errors,
    Spanned,
};

pub fn into_query_storage(src: impl Into<String>) -> tydi_common::error::Result<Database> {
    let mut _db = Database::default();
    let db = &mut _db;

    let src = src.into();
    let (tokens, mut errs) = lexer().parse_recovery(src.as_str());

    let (ast, parse_errs) = if let Some(tokens) = tokens {
        let len = src.chars().count();
        let (ast, parse_errs) =
            namespaces_parser().parse_recovery(Stream::from_iter(len..len + 1, tokens.into_iter()));

        println!("{:#?}", ast);

        (ast, parse_errs)
    } else {
        (None, Vec::new())
    };

    report_errors(&src, errs.clone(), parse_errs.clone());

    if errs.len() > 1 || parse_errs.len() > 1 {
        return Err(Error::ParsingError(
            "Errors during parsing, see report.".to_string(),
        ));
    }

    let mut eval_errors = vec![];
    let mut project = Project::new("proj", ".")?;
    if let Some(ast) = ast {
        for (name, parsed_namespace) in ast.into_iter() {
            match PathName::try_new(name) {
                Ok(name) => {
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
                        let mut namespace = Namespace::new(name)?;

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

    db.set_project(Arc::new(project));

    Ok(_db)
}
