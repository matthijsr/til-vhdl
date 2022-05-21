use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
    sync::Arc,
};

use til_query::{
    common::logical::logicaltype::{stream::Stream, LogicalType},
    ir::{
        interface_port::InterfacePort,
        project::interface::Interface,
        traits::{GetSelf, InternArc},
        Ir,
    },
};
use tydi_common::{
    error::TryResult,
    map::InsertionOrderedSet,
    name::{Name, PathName},
    traits::Documents,
};
use tydi_intern::Id;

use crate::{
    eval::eval_ident,
    interface_expr::{DomainList, InterfaceDef, InterfaceExpr, PortsDef},
    Spanned,
};

use super::{eval_common_error, eval_name, eval_type::eval_type_expr, EvalError};

pub fn eval_interface_expr(
    db: &dyn Ir,
    expr: &Spanned<InterfaceExpr>,
    interfaces: &HashMap<Name, Id<Arc<Interface>>>,
    interface_imports: &HashMap<PathName, Id<Arc<Interface>>>,
    types: &HashMap<Name, Id<LogicalType>>,
    type_imports: &HashMap<PathName, Id<LogicalType>>,
) -> Result<Id<Arc<Interface>>, EvalError> {
    match &expr.0 {
        InterfaceExpr::Identifier(ident) => {
            eval_ident(ident, &expr.1, interfaces, interface_imports, "interface")
        }
        InterfaceExpr::Definition((iface_def, span)) => match iface_def {
            InterfaceDef::Error => Err(EvalError {
                span: span.clone(),
                msg: "Invalid expression for interface definition".to_string(),
            }),
            InterfaceDef::Def(domain_list, ports) => match &ports.0 {
                PortsDef::Error => Err(EvalError {
                    span: ports.1.clone(),
                    msg: "Invalid expression for ports definition".to_string(),
                }),
                PortsDef::Def(ports_def) => {
                    let mut result = if let Some(domain_list) = domain_list {
                        match &domain_list.0 {
                            DomainList::Error => Err(EvalError {
                                span: domain_list.1.clone(),
                                msg: "Domain list error".to_string(),
                            }),
                            DomainList::List(list) => {
                                let mut doms = InsertionOrderedSet::new();
                                for dom in list {
                                    if !doms.insert(eval_name(&dom.0, &dom.1)?) {
                                        return Err(EvalError {
                                            span: dom.1.clone(),
                                            msg: format!(
                                                "Duplicate domain in Interface, \"{}\"",
                                                dom.0
                                            ),
                                        });
                                    }
                                }
                                eval_common_error(
                                    Interface::new_domains(doms.iter()),
                                    &domain_list.1,
                                )
                            }
                        }
                    } else {
                        Ok(Interface::new_empty())
                    }?;
                    let mut dups = HashSet::new();
                    for (port_def, port_span) in ports_def {
                        let name = eval_name(&port_def.name.0, &port_def.name.1)?;
                        if dups.contains(&name) {
                            return Err(EvalError {
                                span: port_def.name.1.clone(),
                                msg: format!("Duplicate label in Interface, \"{}\"", name),
                            });
                        } else {
                            dups.insert(name.clone());
                            let stream_id: Id<Stream> = eval_common_error(
                                eval_type_expr(
                                    db,
                                    (&port_def.props.0.typ.0, &port_def.props.0.typ.1),
                                    types,
                                    type_imports,
                                )?
                                .get(db)
                                .try_result(),
                                &port_def.props.0.typ.1,
                            )?;
                            let port_dom = if let Some(domain) = &port_def.props.0.domain {
                                Some(eval_name(&domain.0, &domain.1)?)
                            } else {
                                None
                            };
                            let mut port = eval_common_error(
                                InterfacePort::try_from((
                                    name,
                                    stream_id,
                                    (port_dom, port_def.props.0.mode.0),
                                )),
                                port_span,
                            )?;
                            if let Some(doc) = &port_def.doc {
                                port.set_doc(&doc.0);
                            }
                            eval_common_error(result.push_port(port), port_span)?;
                        }
                    }
                    Ok(result.intern_arc(db))
                }
            },
        },
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use chumsky::{prelude::Simple, Parser, Stream};
    use til_query::ir::db::Database;
    use tydi_common::error::TryResult;

    use crate::{
        eval::eval_type::tests::test_expr_parse_type, interface_expr::interface_expr, lex::lexer,
        report::report_errors,
    };

    use super::*;

    pub(crate) fn test_expr_parse_interface(
        src: impl Into<String>,
        name: impl TryResult<Name>,
        db: &dyn Ir,
        types: &HashMap<Name, Id<LogicalType>>,
        interfaces: &mut HashMap<Name, Id<Arc<Interface>>>,
    ) {
        let src = src.into();
        let (tokens, mut errs) = lexer().parse_recovery(src.as_str());

        // println!("{:#?}", tokens);

        let parse_errs = if let Some(tokens) = tokens {
            // println!("Tokens = {:?}", tokens);
            let len = src.chars().count();
            let (ast, parse_errs) = interface_expr()
                .parse_recovery(Stream::from_iter(len..len + 1, tokens.into_iter()));

            if let Some(expr) = ast {
                match eval_interface_expr(
                    db,
                    &expr,
                    interfaces,
                    &HashMap::new(),
                    types,
                    &HashMap::new(),
                ) {
                    Ok(def) => {
                        interfaces.insert(name.try_result().unwrap(), def.clone());
                        println!("{}", def.get(db));
                    }
                    Err(e) => errs.push(Simple::custom(e.span, e.msg)),
                };
            }

            parse_errs
        } else {
            Vec::new()
        };

        report_errors(&src, errs, parse_errs);
    }

    #[test]
    fn test_interface_def() {
        let db = &Database::default();
        let mut types = HashMap::new();
        let mut interfaces = HashMap::new();
        test_expr_parse_type(
            "Stream (
        data: Bits(4),
        throughput: 2.0,
        dimensionality: 0,
        synchronicity: Sync,
        complexity: 4.3,
        direction: Forward,
        user: Null,
        keep: false,
    )",
            "a",
            db,
            &mut types,
        );
        test_expr_parse_interface("(a: in a, b: out a)", "a", db, &types, &mut interfaces);
    }

    #[test]
    fn test_interface_ref() {
        let db = &Database::default();
        let types = HashMap::new();
        let mut interfaces = HashMap::new();
        test_expr_parse_interface(
            "(a: in Stream (
        data: Bits(4),
        throughput: 2.0,
        dimensionality: 0,
        synchronicity: Sync,
        complexity: 4.3,
        direction: Forward,
        user: Null,
        keep: false,
    ))",
            "a",
            db,
            &types,
            &mut interfaces,
        );
        test_expr_parse_interface("a", "b", db, &types, &mut interfaces);
    }

    #[test]
    fn test_invalid_interface_def_duplicate() {
        let db = &Database::default();
        let mut types = HashMap::new();
        let mut interfaces = HashMap::new();
        test_expr_parse_type(
            "Stream (
        data: Bits(4),
        throughput: 2.0,
        dimensionality: 0,
        synchronicity: Sync,
        complexity: 4.3,
        direction: Forward,
        user: Null,
        keep: false,
    )",
            "a",
            db,
            &mut types,
        );
        test_expr_parse_interface("(a: in a, a: out a)", "a", db, &types, &mut interfaces);
    }

    #[test]
    fn test_interface_indirection() {
        let db = &Database::default();
        let mut types = HashMap::new();
        let mut interfaces = HashMap::new();
        test_expr_parse_type(
            "Stream (
        data: Bits(4),
        throughput: 2.0,
        dimensionality: 0,
        synchronicity: Sync,
        complexity: 4.3,
        direction: Forward,
        user: Null,
        keep: false,
    )",
            "a",
            db,
            &mut types,
        );
        test_expr_parse_interface("(a: in a)", "a", db, &types, &mut interfaces);
        test_expr_parse_interface("a", "b", db, &types, &mut interfaces);
        assert_eq!(
            interfaces.get(&Name::try_new("a").unwrap()),
            interfaces.get(&Name::try_new("b").unwrap()),
        )
    }
}
