use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};
use chumsky::{prelude::*, stream::Stream};
use std::{collections::HashMap, env, fmt, fs, hash::Hash, path::PathBuf};
use til_query::common::{
    logical::logicaltype::stream::{Direction, Synchronicity, Throughput},
    physical::complexity::Complexity,
};
use tydi_common::{
    name::{Name, PathName},
    numbers::{NonNegative, Positive, PositiveReal},
};

use crate::{
    lex::{Operator, PortMode, Token, TypeKeyword},
    Span,
};

pub type Spanned<T> = (T, Span);

#[derive(Clone, Debug, PartialEq)]
pub struct HashablePositiveReal(PositiveReal);

impl Eq for HashablePositiveReal {}

impl Hash for HashablePositiveReal {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.get().to_ne_bytes().hash(state);
    }
}

impl HashablePositiveReal {
    pub fn get(&self) -> f64 {
        self.0.get()
    }

    pub fn positive_real(&self) -> PositiveReal {
        self.0
    }

    pub fn non_negative(&self) -> NonNegative {
        self.0.get().ceil() as NonNegative
    }

    pub fn positive(&self) -> Positive {
        Positive::new(self.non_negative()).unwrap()
    }
}

impl Into<Throughput> for HashablePositiveReal {
    fn into(self) -> Throughput {
        Throughput::new(self.positive_real())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Value {
    Path(PathBuf),
    Synchronicity(Synchronicity),
    Direction(Direction),
    Int(NonNegative),
    Float(HashablePositiveReal),
    Version(String),
    Boolean(bool),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Ident {
    Name(Name),
    PathName(PathName),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Expr {
    Error,
    Value(Value),
    Ident(Ident),
    TypeDef(LogicalType),
    PortDef(Spanned<PortMode>, Box<Spanned<Self>>),
    Documentation(Spanned<String>, Box<Spanned<Self>>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TypeDecl {
    name: Spanned<Name>,
    typ: Spanned<LogicalType>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum LogicalType {
    Null,
    Bits(Positive),
    Group(Vec<(Spanned<Name>, Spanned<LogicalType>)>),
    Union(Vec<(Spanned<Name>, Spanned<LogicalType>)>),
    Stream(StreamType),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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
            let doc_body = filter_map(|span, tok| match tok {
                Token::Documentation(docstr) => Ok(docstr.clone()),
                _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
            })
            .map_with_span(|body, span| (body, span))
            .labelled("documentation");

            let doc = doc_body
                .clone()
                .then(expr.clone())
                .map(|(body, subj)| Expr::Documentation(body, Box::new(subj)));

            let val = filter_map(|span, tok| match tok {
                Token::Num(num) => {
                    if let Ok(i) = num.parse() {
                        Ok(Value::Int(i))
                    } else if let Ok(f) = num.parse() {
                        Ok(Value::Float(HashablePositiveReal(
                            PositiveReal::new(f).unwrap(),
                        )))
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
            .map(Expr::Value)
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

            let atom = doc
                .or(val)
                .or(ident.map(Expr::Ident))
                .map_with_span(|expr, span| (expr, span))
                // Attempt to recover anything that looks like a parenthesised expression but contains errors
                .recover_with(nested_delimiters(
                    Token::Ctrl('('),
                    Token::Ctrl(')'),
                    [
                        (Token::Ctrl('['), Token::Ctrl(']')),
                        (Token::Ctrl('{'), Token::Ctrl('}')),
                    ],
                    |span| (Expr::Error, span),
                ))
                // Attempt to recover anything that looks like a list but contains errors
                .recover_with(nested_delimiters(
                    Token::Ctrl('['),
                    Token::Ctrl(']'),
                    [
                        (Token::Ctrl('('), Token::Ctrl(')')),
                        (Token::Ctrl('{'), Token::Ctrl('}')),
                    ],
                    |span| (Expr::Error, span),
                ));

            atom
        });

        raw_expr
    })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::lex::lexer;

    use super::*;

    fn source(path: impl AsRef<Path>) -> String {
        std::fs::read_to_string(path).unwrap()
    }

    fn test_expr_parse(src: impl Into<String>) {
        let src = src.into();
        let (tokens, mut errs) = lexer().parse_recovery(src.as_str());

        let parse_errs = if let Some(tokens) = tokens {
            // println!("Tokens = {:?}", tokens);
            let len = src.chars().count();
            let (ast, parse_errs) =
                expr_parser().parse_recovery(Stream::from_iter(len..len + 1, tokens.into_iter()));

            println!("{:#?}", ast);

            parse_errs
        } else {
            Vec::new()
        };

        errs.into_iter()
            .map(|e| e.map(|c| c.to_string()))
            .chain(parse_errs.into_iter().map(|e| e.map(|tok| tok.to_string())))
            .for_each(|e| {
                let report = Report::build(ReportKind::Error, (), e.span().start);

                let report = match e.reason() {
                    chumsky::error::SimpleReason::Unclosed { span, delimiter } => report
                        .with_message(format!(
                            "Unclosed delimiter {}",
                            delimiter.fg(Color::Yellow)
                        ))
                        .with_label(
                            Label::new(span.clone())
                                .with_message(format!(
                                    "Unclosed delimiter {}",
                                    delimiter.fg(Color::Yellow)
                                ))
                                .with_color(Color::Yellow),
                        )
                        .with_label(
                            Label::new(e.span())
                                .with_message(format!(
                                    "Must be closed before this {}",
                                    e.found()
                                        .unwrap_or(&"end of file".to_string())
                                        .fg(Color::Red)
                                ))
                                .with_color(Color::Red),
                        ),
                    chumsky::error::SimpleReason::Unexpected => report
                        .with_message(format!(
                            "{}, expected {}",
                            if e.found().is_some() {
                                "Unexpected token in input"
                            } else {
                                "Unexpected end of input"
                            },
                            if e.expected().len() == 0 {
                                "something else".to_string()
                            } else {
                                e.expected()
                                    .map(|expected| match expected {
                                        Some(expected) => expected.to_string(),
                                        None => "end of input".to_string(),
                                    })
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            }
                        ))
                        .with_label(
                            Label::new(e.span())
                                .with_message(format!(
                                    "Unexpected token {}",
                                    e.found()
                                        .unwrap_or(&"end of file".to_string())
                                        .fg(Color::Red)
                                ))
                                .with_color(Color::Red),
                        ),
                    chumsky::error::SimpleReason::Custom(msg) => {
                        report.with_message(msg).with_label(
                            Label::new(e.span())
                                .with_message(format!("{}", msg.fg(Color::Red)))
                                .with_color(Color::Red),
                        )
                    }
                };

                report.finish().print(Source::from(&src)).unwrap();
            });
    }

    #[test]
    fn test_test_til() {
        test_expr_parse(source("test.til"))
    }

    #[test]
    fn test_sample_til() {
        test_expr_parse(source("sample.til"))
    }

    #[test]
    fn test_name_expr() {
        test_expr_parse("name");
    }

    #[test]
    fn test_invalid_name() {
        test_expr_parse("_name");
    }

    #[test]
    fn test_path_name_expr() {
        test_expr_parse("path::name::thing");
    }

    #[test]
    fn test_invalid_path_name() {
        test_expr_parse("path::_name::thing");
    }

    #[test]
    fn playground() {
        test_expr_parse("-1.2");
    }
}
