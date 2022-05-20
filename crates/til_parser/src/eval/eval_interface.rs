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
    name::{Name, PathName},
    traits::Documents,
};
use tydi_intern::Id;

use crate::{eval::eval_ident, expr::Expr, Spanned};

use super::{eval_common_error, eval_name, eval_type::eval_type_expr, EvalError};

pub fn eval_interface_expr(
    db: &dyn Ir,
    expr: &Spanned<Expr>,
    interfaces: &HashMap<Name, Id<Arc<Interface>>>,
    interface_imports: &HashMap<PathName, Id<Arc<Interface>>>,
    types: &HashMap<Name, Id<LogicalType>>,
    type_imports: &HashMap<PathName, Id<LogicalType>>,
) -> Result<Id<Arc<Interface>>, EvalError> {
    match &expr.0 {
        Expr::Ident(ident) => {
            eval_ident(ident, &expr.1, interfaces, interface_imports, "interface")
        }
        Expr::InterfaceDef(domain_list, iface) => {
            let mut dups = HashSet::new();
            let mut result = Interface::new_empty();
            for port_def_expr in iface {
                let mut port_def_expr = port_def_expr;
                let mut doc = None;
                if let (Expr::Documentation((doc_str, _), sub_expr), _) = port_def_expr {
                    port_def_expr = sub_expr;
                    doc = Some(doc_str);
                };

                if let Expr::PortDef((name_string, name_span), (props, _)) = &port_def_expr.0 {
                    let name = eval_name(name_string, name_span)?;
                    if dups.contains(&name) {
                        return Err(EvalError {
                            span: name_span.clone(),
                            msg: format!("Duplicate label in Interface, \"{}\"", name),
                        });
                    } else {
                        dups.insert(name.clone());
                        let stream_id: Id<Stream> = eval_common_error(
                            eval_type_expr(db, &props.typ, types, type_imports)?
                                .get(db)
                                .try_result(),
                            &props.typ.1,
                        )?;
                        let mut port = eval_common_error(
                            InterfacePort::try_from((name, stream_id, props.mode.0)),
                            name_span,
                        )?;
                        if let Some(doc) = doc {
                            port.set_doc(doc);
                        }
                        eval_common_error(result.push_port(port), name_span)?;
                    }
                } else {
                    return Err(EvalError {
                        span: port_def_expr.1.clone(),
                        msg: format!("{:#?} is not a port definition", port_def_expr.0),
                    });
                }
            }
            Ok(result.intern_arc(db))
        }
        _ => Err(EvalError {
            span: expr.1.clone(),
            msg: format!("Invalid expression {:#?} for interface definition", &expr.0),
        }),
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use chumsky::{prelude::Simple, Parser, Stream};
    use til_query::ir::db::Database;
    use tydi_common::error::TryResult;

    use crate::{
        eval::eval_type::tests::test_expr_parse_type, expr::expr_parser, lex::lexer,
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
            let (ast, parse_errs) =
                expr_parser().parse_recovery(Stream::from_iter(len..len + 1, tokens.into_iter()));

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
