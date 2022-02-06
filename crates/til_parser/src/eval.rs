use std::{any::type_name, collections::HashMap};

use til_query::common::{
    logical::logicaltype::stream::{Direction, Synchronicity, Throughput},
    physical::complexity::Complexity,
};
use tydi_common::{
    name::{Name, PathName},
    numbers::{NonNegative, Positive},
};

use crate::{
    expr::{Expr, LogicalTypeExpr},
    ident_expr::IdentExpr,
    Span, Spanned,
};

struct EvalError {
    span: Span,
    msg: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Def<T> {
    Import(PathName),
    Ident(Name),
    Def(T),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum LogicalTypeDef {
    Null,
    Bits(Positive),
    Group(Vec<(Spanned<Name>, Box<Spanned<Def<Self>>>)>),
    Union(Vec<(Spanned<Name>, Box<Spanned<Def<Self>>>)>),
    Stream(StreamTypeDef),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StreamTypeDef {
    data: Box<Spanned<Def<LogicalTypeDef>>>,
    throughput: Spanned<Throughput>,
    dimensionality: Spanned<NonNegative>,
    synchronicity: Spanned<Synchronicity>,
    complexity: Spanned<Complexity>,
    direction: Spanned<Direction>,
    user: Box<Spanned<Def<LogicalTypeDef>>>,
    keep: Spanned<bool>,
}

fn eval_ident<T>(
    ident: &IdentExpr,
    span: &Span,
    defs: &HashMap<Name, Def<T>>,
    imports: &HashMap<PathName, Def<T>>,
) -> Result<Def<T>, EvalError> {
    let eval_name = |n: &String, s: &Span| -> Result<Name, EvalError> {
        match Name::try_new(n) {
            Ok(name) => Ok(name),
            Err(err) => Err(EvalError {
                span: s.clone(),
                msg: format!("Invalid identity {}. {}", n, err),
            }),
        }
    };

    match ident {
        IdentExpr::Name((n, s)) => {
            let name = eval_name(n, s)?;
            if defs.contains_key(&name) {
                Ok(Def::Ident(name))
            } else {
                Err(EvalError {
                    span: s.clone(),
                    msg: format!("No {} with identity {}", type_name::<T>(), &name),
                })
            }
        }
        IdentExpr::PathName(pth) => {
            let mut pthn = vec![];
            for (n, s) in pth {
                let name_span = eval_name(n, s)?;
                pthn.push(name_span);
            }
            let pthn = PathName::new(pthn.into_iter());
            if imports.contains_key(&pthn) {
                Ok(Def::Import(pthn))
            } else {
                Err(EvalError {
                    span: span.clone(),
                    msg: format!("No imported {} with identity {}", type_name::<T>(), &pthn),
                })
            }
        }
    }
}

fn eval_type_expr(
    expr: &Spanned<Expr>,
    types: &HashMap<Name, Def<LogicalTypeDef>>,
    type_imports: &HashMap<PathName, Def<LogicalTypeDef>>,
) -> Result<Def<LogicalTypeDef>, EvalError> {
    match &expr.0 {
        Expr::Error => unreachable!(),
        Expr::Ident(ident) => eval_ident(ident, &expr.1, types, type_imports),
        Expr::TypeDef(typ_expr) => match typ_expr {
            LogicalTypeExpr::Null => Ok(Def::Def(LogicalTypeDef::Null)),
            LogicalTypeExpr::Bits((num, num_span)) => {
                if let Ok(p) = num.parse() {
                    Ok(Def::Def(LogicalTypeDef::Bits(p)))
                } else {
                    Err(EvalError {
                        span: num_span.clone(),
                        msg: format!("{} is not a positive integer", num),
                    })
                }
            }
            LogicalTypeExpr::Group(_) => todo!(),
            LogicalTypeExpr::Union(_) => todo!(),
            LogicalTypeExpr::Stream(_) => todo!(),
        },
        _ => Err(EvalError {
            span: expr.1.clone(),
            msg: format!("Invalid expression {:#?} for type definition", &expr.0),
        }),
    }
}
