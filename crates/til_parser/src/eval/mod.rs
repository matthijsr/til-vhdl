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

pub mod eval_implementation;
pub mod eval_interface;
pub mod eval_streamlet;
pub mod eval_type;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EvalError {
    span: Span,
    msg: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Def<T> {
    Import(PathName),
    Ident(Name),
    Def(T),
}

fn eval_ident<T>(
    ident: &IdentExpr,
    span: &Span,
    defs: &HashMap<Name, Def<T>>,
    imports: &HashMap<PathName, Def<T>>,
) -> Result<Def<T>, EvalError> {
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

fn eval_name(n: &String, s: &Span) -> Result<Name, EvalError> {
    match Name::try_new(n) {
        Ok(name) => Ok(name),
        Err(err) => Err(EvalError {
            span: s.clone(),
            msg: format!("Invalid identity {}. {}", n, err),
        }),
    }
}

pub fn get_base_def<T: Clone>(
    sel: &Def<T>,
    span: &Span,
    declarations: &HashMap<Name, Def<T>>,
    previous: &mut HashSet<Name>,
) -> Result<T, EvalError> {
    match sel {
        Def::Import(_) => todo!(),
        Def::Ident(sel) => {
            if let Some(def) = declarations.get(sel) {
                match def {
                    Def::Import(_) => todo!(),
                    Def::Ident(name) => {
                        if !previous.insert(name.clone()) {
                            // This shouldn't normally be possible, but it doesn't hurt to be careful.
                            return Err(EvalError {
                                span: span.clone(),
                                msg: format!(
                                    "Circular dependency! {} -> {}",
                                    previous
                                        .iter()
                                        .map(|n| n.to_string())
                                        .collect::<Vec<String>>()
                                        .join(" ->\n"),
                                    sel
                                ),
                            });
                        }
                        get_base_def(def, span, declarations, previous)
                    }
                    Def::Def(result) => Ok(result.clone()),
                }
            } else {
                Err(EvalError {
                    span: span.clone(),
                    msg: format!("No such interface {}", sel),
                })
            }
        }
        Def::Def(def) => Ok(def.clone()),
    }
}
