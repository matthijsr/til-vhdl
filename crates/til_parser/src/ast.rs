use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};
use chumsky::{prelude::*, stream::Stream};
use std::{collections::HashMap, env, fmt, fs, path::PathBuf};
use til_query::common::{
    logical::logicaltype::stream::{Direction, Synchronicity, Throughput},
    physical::complexity::Complexity,
};
use tydi_common::{
    name::{Name, PathName},
    numbers::{NonNegative, Positive, PositiveReal},
};

use crate::{
    lex::{Operator, SynchronicityKeyword, Token, TypeKeyword},
    Span,
};

pub type Spanned<T> = (T, Span);

enum Value {
    Path(PathBuf),
    Synchronicity(Synchronicity),
    Direction(Direction),
    Int(NonNegative),
    Float(PositiveReal),
    Version(String),
    Boolean(bool),
}

pub enum Ident {
    Name(Name),
    PathName(PathName),
}

pub enum Expr {
    Error,
    Ident(Ident),
    TypeDef(LogicalType),
}

pub struct TypeDecl {
    name: Spanned<Name>,
    typ: Spanned<LogicalType>,
}

pub enum LogicalType {
    Null,
    Bits(Positive),
    Group(Vec<(Spanned<Name>, Spanned<LogicalType>)>),
    Union(Vec<(Spanned<Name>, Spanned<LogicalType>)>),
    Stream(StreamType),
}

pub struct StreamType {
    data: Box<Spanned<LogicalType>>,
    throughput: Spanned<Throughput>,
    dimensionality: Spanned<NonNegative>,
    synchronicity: Spanned<Synchronicity>,
    complexity: Spanned<Complexity>,
    direction: Spanned<Direction>,
    user: Box<Spanned<LogicalType>>,
    keep: Spanned<bool>,
}

fn expr_parser() -> impl Parser<Token, Spanned<Expr>, Error = Simple<Token>> + Clone {
    recursive(|expr| {
        let raw_expr = recursive(|raw_expr| {
            let val = filter_map(|span, tok| match tok {
                Token::Num(num) => {
                    if let Ok(i) = num.parse() {
                        Ok(Value::Int(i))
                    } else if let Ok(f) = num.parse() {
                        Ok(Value::Float(PositiveReal::new(f).unwrap()))
                    } else {
                        Err(Simple::custom(
                            span,
                            format!("Lexer error: {} is neither an integer nor a float.", num),
                        ))
                    }
                }
                Token::Path(path) => Ok(Value::Path(PathBuf::from(path))),
                Token::Synchronicity(synch) => Ok(Value::Synchronicity(synch)),
                Token::Direction(dir) => Ok(Value::Direction(dir)),
                Token::Version(ver) => Ok(Value::Version(ver)),
                Token::Boolean(boolean) => Ok(Value::Boolean(boolean)),
                _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
            })
            .labelled("value");

            let name = filter_map(|span, tok| match tok {
                Token::Identifier(ident) => match Name::try_new(ident) {
                    Ok(name) => Ok(name),
                    Err(err) => Err(Simple::custom(span, err)),
                },
                _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
            });

            let path_name = name
                .clone()
                .chain(
                    just(Token::Op(Operator::Path))
                        .ignore_then(name.clone())
                        .repeated()
                        .at_least(1),
                )
                .map(|pth| Ident::PathName(PathName::new(pth.into_iter())));

            let ident = path_name.or(name.map(Ident::Name)).labelled("identifier");

            todo!()
        });

        todo!()
    })
}
