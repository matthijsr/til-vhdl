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

use super::{Def, EvalError, eval_name, eval_ident};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum LogicalTypeDef {
    Null,
    Bits(Positive),
    Group(Vec<(Name, Box<Def<Self>>)>),
    Union(Vec<(Name, Box<Def<Self>>)>),
    Stream(StreamTypeDef),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StreamTypeDef {
    data: Option<Box<Def<LogicalTypeDef>>>,
    throughput: Option<Throughput>,
    dimensionality: Option<NonNegative>,
    synchronicity: Option<Synchronicity>,
    complexity: Option<Complexity>,
    direction: Option<Direction>,
    user: Option<Box<Def<LogicalTypeDef>>>,
    keep: Option<bool>,
}

fn eval_type_expr(
    expr: &Spanned<Expr>,
    types: &HashMap<Name, Def<LogicalTypeDef>>,
    type_imports: &HashMap<PathName, Def<LogicalTypeDef>>,
) -> Result<Def<LogicalTypeDef>, EvalError> {
    let eval_group = |group: &Vec<(Spanned<String>, Spanned<Expr>)>| -> Result<Vec<(Name, Box<Def<LogicalTypeDef>>)>, EvalError> {
        let mut dups = HashSet::new();
        let mut result = vec![];
        for ((name_string, name_span), el_expr) in group {
            let name = eval_name(name_string, name_span)?;
            if dups.contains(&name) {
                return Err(EvalError {
                    span: name_span.clone(),
                    msg: format!("Duplicate label in Group or Union, {}", name)
                })
            } else {
                dups.insert(name.clone());
                result.push((name, Box::new(eval_type_expr(el_expr, types, type_imports)?)));
            }
        }
        Ok(result)
    };

    let eval_stream =
        |stream_props: &Vec<(Spanned<String>, Spanned<Expr>)>| -> Result<StreamTypeDef, EvalError> {
            let mut stream = StreamTypeDef {
                data: None,
                throughput: None,
                dimensionality: None,
                synchronicity: None,
                complexity: None,
                direction: None,
                user: None,
                keep: None,
            };
            for ((name_string, name_span), el_expr) in stream_props {
                let duplicate_error =
                    |label: &String, label_span: &Span| -> Result<StreamTypeDef, EvalError> {
                        Err(EvalError {
                            span: label_span.clone(),
                            msg: format!("Invalid Stream property {}", label),
                        })
                    };

                let invalid_expr =
                    |label: &str, prop_expr: &Spanned<Expr>| -> Result<StreamTypeDef, EvalError> {
                        Err(EvalError {
                            span: prop_expr.1.clone(),
                            msg: format!(
                                "Invalid expression {:#?} for property {}",
                                prop_expr.0, label
                            ),
                        })
                    };

                let custom_error = |label: &str,
                                    err_span: &Spanned<Expr>,
                                    err: String|
                 -> Result<StreamTypeDef, EvalError> {
                    Err(EvalError {
                        span: err_span.1.clone(),
                        msg: format!("Error assigning {}: {}", label, err),
                    })
                };

                match name_string.as_str() {
                    "data" => {
                        if stream.data == None {
                            stream.data =
                                Some(Box::new(eval_type_expr(el_expr, types, type_imports)?));
                        } else {
                            return duplicate_error(name_string, name_span);
                        }
                    }
                    "throughput" => {
                        if stream.throughput == None {
                            match &el_expr.0 {
                                Expr::Value(Value::Int(i)) => match Throughput::try_new(*i) {
                                    Ok(t) => {
                                        stream.throughput = Some(t);
                                    }
                                    Err(err) => {
                                        return custom_error("throughput", el_expr, err.to_string())
                                    }
                                },
                                Expr::Value(Value::Float(f)) => {
                                    stream.throughput = Some(f.positive_real().into());
                                }
                                _ => return invalid_expr("throughput", el_expr),
                            }
                        } else {
                            return duplicate_error(name_string, name_span);
                        }
                    }
                    "dimensionality" => {
                        if stream.dimensionality == None {
                            match &el_expr.0 {
                                Expr::Value(Value::Int(i)) => {
                                    stream.dimensionality = Some(*i);
                                }
                                _ => return invalid_expr("dimensionality", el_expr),
                            }
                        } else {
                            return duplicate_error(name_string, name_span);
                        }
                    }
                    "synchronicity" => {
                        if stream.synchronicity == None {
                            match &el_expr.0 {
                                Expr::Value(Value::Synchronicity(s)) => {
                                    stream.synchronicity = Some(*s);
                                }
                                _ => return invalid_expr("synchronicity", el_expr),
                            }
                        } else {
                            return duplicate_error(name_string, name_span);
                        }
                    }
                    "complexity" => {
                        if stream.complexity == None {
                            match &el_expr.0 {
                                Expr::Value(Value::Int(i)) => {
                                    stream.complexity = Some(Complexity::from(*i));
                                }
                                Expr::Value(Value::Float(f)) => {
                                    match Complexity::try_from(f.positive_real()) {
                                        Ok(c) => {
                                            stream.complexity = Some(c);
                                        }
                                        Err(err) => {
                                            return custom_error(
                                                "complexity",
                                                el_expr,
                                                err.to_string(),
                                            )
                                        }
                                    }
                                }
                                Expr::Value(Value::Version(ver)) => {
                                    match Complexity::from_str(ver.as_str()) {
                                        Ok(c) => {
                                            stream.complexity = Some(c);
                                        }
                                        Err(err) => {
                                            return custom_error(
                                                "complexity",
                                                el_expr,
                                                err.to_string(),
                                            )
                                        }
                                    }
                                }
                                _ => return invalid_expr("complexity", el_expr),
                            }
                        } else {
                            return duplicate_error(name_string, name_span);
                        }
                    }
                    "direction" => {
                        if stream.direction == None {
                            match &el_expr.0 {
                                Expr::Value(Value::Direction(d)) => {
                                    stream.direction = Some(*d);
                                }
                                _ => return invalid_expr("direction", el_expr),
                            }
                        } else {
                            return duplicate_error(name_string, name_span);
                        }
                    }
                    "user" => {
                        if stream.user == None {
                            stream.user =
                                Some(Box::new(eval_type_expr(el_expr, types, type_imports)?));
                        } else {
                            return duplicate_error(name_string, name_span);
                        }
                    }
                    "keep" => {
                        if stream.keep == None {
                            match &el_expr.0 {
                                Expr::Value(Value::Boolean(b)) => {
                                    stream.keep = Some(*b);
                                }
                                _ => return invalid_expr("keep", el_expr),
                            }
                        } else {
                            return duplicate_error(name_string, name_span);
                        }
                    }
                    _ => {
                        return Err(EvalError {
                            span: name_span.clone(),
                            msg: format!("Invalid Stream property {}", name_string),
                        })
                    }
                }
            }

            Ok(stream)
        };

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
            LogicalTypeExpr::Group(group) => {
                Ok(Def::Def(LogicalTypeDef::Group(eval_group(group)?)))
            }
            LogicalTypeExpr::Union(group) => {
                Ok(Def::Def(LogicalTypeDef::Union(eval_group(group)?)))
            }
            LogicalTypeExpr::Stream(props) => {
                Ok(Def::Def(LogicalTypeDef::Stream(eval_stream(props)?)))
            }
        },
        _ => Err(EvalError {
            span: expr.1.clone(),
            msg: format!("Invalid expression {:#?} for type definition", &expr.0),
        }),
    }
}
