use std::collections::{HashMap, HashSet};

use til_query::{
    common::logical::logicaltype::{stream::Stream, LogicalType},
    ir::{
        physical_properties::InterfaceDirection,
        project::interface::InterfaceCollection,
        traits::{GetSelf, InternSelf},
        Ir,
    },
};
use tydi_common::{
    error::TryResult,
    name::{Name, PathName},
};
use tydi_intern::Id;

use crate::{eval::eval_ident, expr::Expr, Span, Spanned};

use super::{eval_common_error, eval_name, eval_type::eval_type_expr, Def, EvalError};

pub fn eval_interface_expr(
    db: &dyn Ir,
    expr: &Spanned<Expr>,
    interfaces: &HashMap<Name, Id<InterfaceCollection>>,
    interface_imports: &HashMap<PathName, Id<InterfaceCollection>>,
    types: &HashMap<Name, Id<LogicalType>>,
    type_imports: &HashMap<PathName, Id<LogicalType>>,
) -> Result<Id<InterfaceCollection>, EvalError> {
    match &expr.0 {
        Expr::Ident(ident) => {
            eval_ident(ident, &expr.1, interfaces, interface_imports, "interface")
        }
        Expr::InterfaceDef(iface) => {
            let mut dups = HashSet::new();
            let mut result = InterfaceCollection::new_empty();
            for port_def_expr in iface {
                if let Expr::PortDef((name_string, name_span), (props, props_span)) =
                    &port_def_expr.0
                {
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
                        eval_common_error(
                            result.push(db, (name, stream_id, props.mode.0)),
                            name_span,
                        )?;
                    }
                } else {
                    return Err(EvalError {
                        span: port_def_expr.1.clone(),
                        msg: format!("{:#?} is not a port definition", port_def_expr.0),
                    });
                }
            }
            Ok(result.intern(db))
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
        interfaces: &mut HashMap<Name, Id<InterfaceCollection>>,
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
