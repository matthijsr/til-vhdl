use std::{
    any::type_name,
    collections::{HashMap, HashSet},
    convert::TryFrom,
    str::FromStr,
};

use til_query::common::{
    logical::logicaltype::stream::{Direction, Synchronicity, Throughput},
    physical::complexity::Complexity,
};
use tydi_common::{
    name::{Name, PathName},
    numbers::{NonNegative, Positive},
};

use crate::{
    expr::{Expr, LogicalTypeExpr, Value},
    ident_expr::IdentExpr,
    Span, Spanned,
};

pub mod eval_decl;
pub mod eval_implementation;
pub mod eval_interface;
pub mod eval_streamlet;
pub mod eval_type;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EvalError {
    span: Span,
    msg: String,
}

impl EvalError {
    pub fn new(span: &Span, msg: impl Into<String>) -> Self {
        EvalError {
            span: span.clone(),
            msg: msg.into(),
        }
    }

    pub fn span(&self) -> Span {
        self.span.clone()
    }

    pub fn msg(&self) -> &str {
        self.msg.as_str()
    }
}

pub fn eval_common_error<T>(
    res: Result<T, tydi_common::error::Error>,
    span: &Span,
) -> Result<T, EvalError> {
    match res {
        Ok(val) => Ok(val),
        Err(err) => Err(EvalError {
            span: span.clone(),
            msg: err.to_string(),
        }),
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Def<T> {
    Import(PathName),
    Ident(Name),
    Def(T),
}

pub fn eval_ident<T: Clone>(
    ident: &IdentExpr,
    span: &Span,
    defs: &HashMap<Name, T>,
    imports: &HashMap<PathName, T>,
) -> Result<T, EvalError> {
    match ident {
        IdentExpr::Name((n, s)) => {
            let name = eval_name(n, s)?;
            if let Some(val) = defs.get(&name) {
                Ok(val.clone())
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
            if let Some(val) = imports.get(&pthn) {
                Ok(val.clone())
            } else {
                Err(EvalError {
                    span: span.clone(),
                    msg: format!("No imported {} with identity {}", type_name::<T>(), &pthn),
                })
            }
        }
    }
}

pub fn eval_name(n: &String, s: &Span) -> Result<Name, EvalError> {
    match Name::try_new(n) {
        Ok(name) => Ok(name),
        Err(err) => Err(EvalError {
            span: s.clone(),
            msg: format!("Invalid identity {}. {}", n, err),
        }),
    }
}
