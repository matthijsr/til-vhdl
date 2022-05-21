use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
    str::FromStr,
};

use til_query::{
    common::{
        logical::logicaltype::{
            stream::{Stream, Synchronicity, Throughput},
            LogicalType,
        },
        physical::complexity::Complexity,
        stream_direction::StreamDirection,
    },
    ir::{traits::InternSelf, Ir},
};
use tydi_common::{
    name::{Name, PathName},
    numbers::NonNegative,
};
use tydi_intern::Id;

use crate::{
    expr::{Expr, Value},
    type_expr::{FieldsDef, LogicalTypeDef, StreamProp, StreamProps, TypeExpr},
    Span, Spanned,
};

use super::{eval_common_error, eval_ident, eval_name, EvalError};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StreamTypeDef {
    data: Option<Id<LogicalType>>,
    throughput: Option<Throughput>,
    dimensionality: Option<NonNegative>,
    synchronicity: Option<Synchronicity>,
    complexity: Option<Complexity>,
    direction: Option<StreamDirection>,
    user: Option<Id<LogicalType>>,
    keep: Option<bool>,
}

pub fn eval_type_expr(
    db: &dyn Ir,
    expr: (&TypeExpr, &Span),
    types: &HashMap<Name, Id<LogicalType>>,
    type_imports: &HashMap<PathName, Id<LogicalType>>,
) -> Result<Id<LogicalType>, EvalError> {
    let eval_fields =
        |fields: &Spanned<FieldsDef>| -> Result<Vec<(Name, Id<LogicalType>)>, EvalError> {
            match &fields.0 {
                FieldsDef::Error => Err(EvalError {
                    span: fields.1.clone(),
                    msg: "Invalid fields expression.".to_string(),
                }),
                FieldsDef::Fields(field_list) => {
                    let mut dups = HashSet::new();
                    let mut result = vec![];
                    for ((name_string, name_span), el_expr) in field_list {
                        let name = eval_name(name_string, name_span)?;
                        if dups.contains(&name) {
                            return Err(EvalError {
                                span: name_span.clone(),
                                msg: format!("Duplicate label in Group or Union, \"{}\"", name),
                            });
                        } else {
                            dups.insert(name.clone());
                            result.push((
                                name,
                                eval_type_expr(db, (&el_expr.0, &el_expr.1), types, type_imports)?,
                            ));
                        }
                    }
                    Ok(result)
                }
            }
        };

    let eval_stream = |span: &Span, stream_props: &StreamProps| -> Result<Id<Stream>, EvalError> {
        match &stream_props {
            StreamProps::Error => Err(EvalError {
                span: span.clone(),
                msg: "Invalid Stream properties".to_string(),
            }),
            StreamProps::Props(props) => {
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
                for ((name_string, name_span), prop) in props {
                    let duplicate_error =
                        |label: &String, label_span: &Span| -> Result<Id<Stream>, EvalError> {
                            Err(EvalError {
                                span: label_span.clone(),
                                msg: format!("Duplicate Stream property, \"{}\"", label),
                            })
                        };

                    let invalid_prop = |label: &str,
                                        prop_expr: &Spanned<StreamProp>|
                     -> Result<Id<Stream>, EvalError> {
                        Err(EvalError {
                            span: prop_expr.1.clone(),
                            msg: format!(
                                "Invalid expression {:#?} for property \"{}\"",
                                prop_expr.0, label
                            ),
                        })
                    };

                    let custom_error = |label: &str,
                                        err_span: &Span,
                                        err: String|
                     -> Result<Id<Stream>, EvalError> {
                        Err(EvalError {
                            span: err_span.clone(),
                            msg: format!("Error assigning {}: {}", label, err),
                        })
                    };

                    match name_string.as_str() {
                        "data" => {
                            if stream.data == None {
                                match &prop.0 {
                                    StreamProp::Type(typ) => {
                                        stream.data = Some(eval_type_expr(
                                            db,
                                            (typ, &prop.1),
                                            types,
                                            type_imports,
                                        )?);
                                    }
                                    _ => {
                                        return custom_error(
                                            "data",
                                            &prop.1,
                                            "Expected a type expression".to_string(),
                                        )
                                    }
                                }
                            } else {
                                return duplicate_error(name_string, name_span);
                            }
                        }
                        "throughput" => {
                            if stream.throughput == None {
                                match &prop.0 {
                                    StreamProp::Value(Value::Int(i)) => {
                                        match Throughput::try_new(*i) {
                                            Ok(t) => {
                                                stream.throughput = Some(t);
                                            }
                                            Err(err) => {
                                                return custom_error(
                                                    "throughput",
                                                    &prop.1,
                                                    err.to_string(),
                                                )
                                            }
                                        }
                                    }
                                    StreamProp::Value(Value::Float(f)) => {
                                        stream.throughput = Some(f.positive_real().into());
                                    }
                                    _ => return invalid_prop("throughput", prop),
                                }
                            } else {
                                return duplicate_error(name_string, name_span);
                            }
                        }
                        "dimensionality" => {
                            if stream.dimensionality == None {
                                match &prop.0 {
                                    StreamProp::Value(Value::Int(i)) => {
                                        stream.dimensionality = Some(*i);
                                    }
                                    _ => return invalid_prop("dimensionality", prop),
                                }
                            } else {
                                return duplicate_error(name_string, name_span);
                            }
                        }
                        "synchronicity" => {
                            if stream.synchronicity == None {
                                match &prop.0 {
                                    StreamProp::Value(Value::Synchronicity(s)) => {
                                        stream.synchronicity = Some(*s);
                                    }
                                    _ => return invalid_prop("synchronicity", prop),
                                }
                            } else {
                                return duplicate_error(name_string, name_span);
                            }
                        }
                        "complexity" => {
                            if stream.complexity == None {
                                match &prop.0 {
                                    StreamProp::Value(Value::Int(i)) => {
                                        stream.complexity = Some(Complexity::from(*i));
                                    }
                                    StreamProp::Value(Value::Float(f)) => {
                                        match Complexity::try_from(f.positive_real()) {
                                            Ok(c) => {
                                                stream.complexity = Some(c);
                                            }
                                            Err(err) => {
                                                return custom_error(
                                                    "complexity",
                                                    &prop.1,
                                                    err.to_string(),
                                                )
                                            }
                                        }
                                    }
                                    StreamProp::Value(Value::Version(ver)) => {
                                        match Complexity::from_str(ver.as_str()) {
                                            Ok(c) => {
                                                stream.complexity = Some(c);
                                            }
                                            Err(err) => {
                                                return custom_error(
                                                    "complexity",
                                                    &prop.1,
                                                    err.to_string(),
                                                )
                                            }
                                        }
                                    }
                                    _ => return invalid_prop("complexity", prop),
                                }
                            } else {
                                return duplicate_error(name_string, name_span);
                            }
                        }
                        "direction" => {
                            if stream.direction == None {
                                match &prop.0 {
                                    StreamProp::Value(Value::Direction(d)) => {
                                        stream.direction = Some(*d);
                                    }
                                    _ => return invalid_prop("direction", prop),
                                }
                            } else {
                                return duplicate_error(name_string, name_span);
                            }
                        }
                        "user" => {
                            if stream.user == None {
                                match &prop.0 {
                                    StreamProp::Type(typ) => {
                                        stream.user = Some(eval_type_expr(
                                            db,
                                            (typ, &prop.1),
                                            types,
                                            type_imports,
                                        )?);
                                    }
                                    _ => {
                                        return custom_error(
                                            "user",
                                            &prop.1,
                                            "Expected a type expression".to_string(),
                                        )
                                    }
                                }
                            } else {
                                return duplicate_error(name_string, name_span);
                            }
                        }
                        "keep" => {
                            if stream.keep == None {
                                match &prop.0 {
                                    StreamProp::Value(Value::Boolean(b)) => {
                                        stream.keep = Some(*b);
                                    }
                                    _ => return invalid_prop("keep", prop),
                                }
                            } else {
                                return duplicate_error(name_string, name_span);
                            }
                        }
                        _ => {
                            return Err(EvalError {
                                span: name_span.clone(),
                                msg: format!("Invalid Stream property, \"{}\"", name_string),
                            })
                        }
                    }
                }

                let missing_err = |f: &str| -> EvalError {
                    EvalError {
                        span: span.clone(),
                        msg: format!("Missing \"{}\" field.", f),
                    }
                };

                eval_common_error(
                    Stream::try_new(
                        db,
                        stream.data.ok_or(missing_err("data"))?,
                        stream.throughput.unwrap_or_default(),
                        stream.dimensionality.ok_or(missing_err("dimensionality"))?,
                        stream.synchronicity.ok_or(missing_err("synchronicity"))?,
                        stream.complexity.ok_or(missing_err("complexity"))?,
                        stream.direction.ok_or(missing_err("direction"))?,
                        stream.user.unwrap_or(LogicalType::null_id(db)),
                        stream.keep.unwrap_or(false),
                    ),
                    span,
                )
            }
        }
    };

    match &expr.0 {
        TypeExpr::Error => Err(EvalError {
            span: expr.1.clone(),
            msg: format!("Invalid expression {:#?} for type definition", &expr.0),
        }),
        TypeExpr::Identifier(ident) => eval_ident(ident, &expr.1, types, type_imports, "type"),
        TypeExpr::Definition(typ_def) => match &typ_def.as_ref().0 {
            LogicalTypeDef::Null => Ok(LogicalType::null_id(db)),
            LogicalTypeDef::Bits((num, num_span)) => {
                if let Ok(p) = num.parse() {
                    Ok(LogicalType::Bits(p).intern(db))
                } else {
                    Err(EvalError {
                        span: num_span.clone(),
                        msg: format!("{} is not a positive integer", num),
                    })
                }
            }
            LogicalTypeDef::Group(fields) => Ok(eval_common_error(
                LogicalType::try_new_group(None, eval_fields(fields)?),
                &expr.1,
            )?
            .intern(db)),
            LogicalTypeDef::Union(fields) => Ok(eval_common_error(
                LogicalType::try_new_union(None, eval_fields(fields)?),
                &expr.1,
            )?
            .intern(db)),
            LogicalTypeDef::Stream(props) => {
                Ok(LogicalType::Stream(eval_stream(&props.1, &props.0)?).intern(db))
            }
        },
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use chumsky::{prelude::Simple, Parser, Stream};
    use til_query::ir::{db::Database, traits::GetSelf};
    use tydi_common::error::TryResult;

    use crate::{lex::lexer, report::report_errors, type_expr::type_expr};

    use super::*;

    pub(crate) fn test_expr_parse_type(
        src: impl Into<String>,
        name: impl TryResult<Name>,
        db: &dyn Ir,
        types: &mut HashMap<Name, Id<LogicalType>>,
    ) {
        let src = src.into();
        let (tokens, mut errs) = lexer().parse_recovery(src.as_str());

        // println!("{:#?}", tokens);

        let parse_errs = if let Some(tokens) = tokens {
            // println!("Tokens = {:?}", tokens);
            let len = src.chars().count();
            let (ast, parse_errs) =
                type_expr().parse_recovery(Stream::from_iter(len..len + 1, tokens.into_iter()));

            if let Some(expr) = ast {
                match eval_type_expr(db, (&expr.0, &expr.1), types, &HashMap::new()) {
                    Ok(def) => {
                        types.insert(name.try_result().unwrap(), def.clone());
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
    fn test_null_def() {
        let db = &Database::default();
        let mut types = HashMap::new();
        test_expr_parse_type("Null", "a", db, &mut types);
    }

    #[test]
    fn test_type_ref() {
        let db = &Database::default();
        let mut types = HashMap::new();
        test_expr_parse_type("Null", "a", db, &mut types);
        test_expr_parse_type("a", "b", db, &mut types);
    }

    #[test]
    fn test_bits_def() {
        let db = &Database::default();
        let mut types = HashMap::new();
        test_expr_parse_type("Bits(3)", "a", db, &mut types);
    }

    #[test]
    fn test_bits_invalid_def() {
        let db = &Database::default();
        let mut types = HashMap::new();
        test_expr_parse_type("Bits(0)", "a", db, &mut types);
    }

    #[test]
    fn test_group_def() {
        let db = &Database::default();
        let mut types = HashMap::new();
        test_expr_parse_type("Bits(3)", "a", db, &mut types);
        test_expr_parse_type("Group(a: Bits(1), b: a)", "b", db, &mut types);
    }

    #[test]
    fn test_union_def() {
        let db = &Database::default();
        let mut types = HashMap::new();
        test_expr_parse_type("Bits(3)", "a", db, &mut types);
        test_expr_parse_type("Union(a: Bits(1), b: a)", "b", db, &mut types);
    }

    #[test]
    fn test_invalid_union_def() {
        let db = &Database::default();
        let mut types = HashMap::new();
        test_expr_parse_type("Bits(3)", "a", db, &mut types);
        test_expr_parse_type("Union(a: Bits(1), a: a)", "b", db, &mut types);
    }

    #[test]
    fn test_invalid_union_def_names() {
        let db = &Database::default();
        let mut types = HashMap::new();
        test_expr_parse_type("Bits(3)", "a", db, &mut types);
        test_expr_parse_type("Union(a: Bits(1), b__b: a)", "b", db, &mut types);
    }

    #[test]
    fn test_stream_def() {
        let db = &Database::default();
        let mut types = HashMap::new();
        test_expr_parse_type("Bits(3)", "a", db, &mut types);
        test_expr_parse_type(
            "Stream (
        data: a,
        throughput: 2.0,
        dimensionality: 0,
        synchronicity: Sync,
        complexity: 4.3,
        direction: Forward,
        user: Null,
        keep: false,
    )",
            "b",
            db,
            &mut types,
        );
    }

    #[test]
    fn test_invalid_stream_def_duplicate() {
        let db = &Database::default();
        let mut types = HashMap::new();
        test_expr_parse_type("Bits(3)", "a", db, &mut types);
        test_expr_parse_type(
            "Stream (
        data: a,
        throughput: 2.0,
        dimensionality: 0,
        synchronicity: Sync,
        complexity: 4.3,
        direction: Forward,
        user: Null,
        keep: false,
        keep: true,
    )",
            "b",
            db,
            &mut types,
        );
    }

    #[test]
    fn test_invalid_stream_def_invalid_property() {
        let db = &Database::default();
        let mut types = HashMap::new();
        test_expr_parse_type("Bits(3)", "a", db, &mut types);
        test_expr_parse_type(
            "Stream (
        data: a,
        throughput: 2.0,
        dimensionality: 0,
        synchronicity: Sync,
        complexity: 4.3,
        direction: Forward,
        user: Null,
        keep: false,
        fake: false,
    )",
            "b",
            db,
            &mut types,
        );
    }

    #[test]
    fn test_stream_def_empty() {
        let db = &Database::default();
        let mut types = HashMap::new();
        test_expr_parse_type(
            "Stream (
    )",
            "b",
            db,
            &mut types,
        );
    }

    #[test]
    fn test_stream_def_order() {
        let db = &Database::default();
        let mut types = HashMap::new();
        test_expr_parse_type("Bits(3)", "a", db, &mut types);
        test_expr_parse_type(
            "Stream (
        synchronicity: Sync,
        complexity: 4.3,
        direction: Forward,
        data: a,
        throughput: 2.0,
        dimensionality: 0,
        user: Null,
        keep: false,
    )",
            "b",
            db,
            &mut types,
        );
    }
}
