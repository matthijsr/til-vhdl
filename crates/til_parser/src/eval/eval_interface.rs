use std::collections::{HashMap, HashSet};

use til_query::ir::physical_properties::InterfaceDirection;
use tydi_common::name::{Name, PathName};

use crate::{eval::eval_ident, expr::Expr, Span, Spanned};

use super::{
    eval_name,
    eval_type::{eval_type_expr, LogicalTypeDef},
    Def, EvalError,
};

pub type InterfaceDef = Vec<(Name, InterfaceDirection, Def<LogicalTypeDef>)>;

pub fn eval_interface_expr(
    expr: &Spanned<Expr>,
    interfaces: &HashMap<Name, Def<InterfaceDef>>,
    interface_imports: &HashMap<PathName, Def<InterfaceDef>>,
    types: &HashMap<Name, Def<LogicalTypeDef>>,
    type_imports: &HashMap<PathName, Def<LogicalTypeDef>>,
) -> Result<Def<InterfaceDef>, EvalError> {
    match &expr.0 {
        Expr::Ident(ident) => eval_ident(ident, &expr.1, interfaces, interface_imports),
        Expr::InterfaceDef(iface) => {
            let mut dups = HashSet::new();
            let mut result = vec![];
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
                        result.push((
                            name,
                            props.mode.0,
                            eval_type_expr(props.typ.as_ref(), types, type_imports)?,
                        ));
                    }
                } else {
                    return Err(EvalError {
                        span: port_def_expr.1.clone(),
                        msg: format!("{:#?} is not a port definition", port_def_expr.0),
                    });
                }
            }
            Ok(Def::Def(result))
        }
        _ => Err(EvalError {
            span: expr.1.clone(),
            msg: format!("Invalid expression {:#?} for type definition", &expr.0),
        }),
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use chumsky::{prelude::Simple, Parser, Stream};
    use tydi_common::error::TryResult;

    use crate::{
        eval::{eval_type::tests::test_expr_parse_type, get_base_def},
        expr::expr_parser,
        lex::lexer,
        report::report_errors,
    };

    use super::*;

    pub(crate) fn test_expr_parse_interface(
        src: impl Into<String>,
        name: impl TryResult<Name>,
        types: &HashMap<Name, Def<LogicalTypeDef>>,
        interfaces: &mut HashMap<Name, Def<InterfaceDef>>,
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
                    &expr,
                    interfaces,
                    &HashMap::new(),
                    types,
                    &HashMap::new(),
                ) {
                    Ok(def) => {
                        interfaces.insert(name.try_result().unwrap(), def.clone());
                        println!("{:#?}", def);
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
        let mut types = HashMap::new();
        let mut interfaces = HashMap::new();
        test_expr_parse_type("Null", "a", &mut types);
        test_expr_parse_interface("(a: in Null, b: out a)", "a", &types, &mut interfaces);
    }

    #[test]
    fn test_interface_ref() {
        let types = HashMap::new();
        let mut interfaces = HashMap::new();
        test_expr_parse_interface("(a: in Null)", "a", &types, &mut interfaces);
        test_expr_parse_interface("a", "b", &types, &mut interfaces);
    }

    #[test]
    fn test_invalid_interface_def_duplicate() {
        let types = HashMap::new();
        let mut interfaces = HashMap::new();
        test_expr_parse_interface("(a: in Null, a: out Null)", "a", &types, &mut interfaces);
    }

    #[test]
    fn test_interface_indirection() {
        let types = HashMap::new();
        let mut interfaces = HashMap::new();
        test_expr_parse_interface("(a: in Null)", "a", &types, &mut interfaces);
        test_expr_parse_interface("a", "b", &types, &mut interfaces);
        println!(
            "{:#?}",
            get_base_def(
                &Def::Ident(Name::try_new("b").unwrap()),
                &(0..0),
                &interfaces,
                &mut HashSet::new()
            )
        )
    }
}
