use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};
use chumsky::{prelude::*, primitive::Just, stream::Stream};
use std::{collections::HashMap, env, fmt, fs, hash::Hash, path::PathBuf};
use til_query::{
    common::{
        logical::logicaltype::stream::{Direction, Synchronicity, Throughput},
        physical::complexity::Complexity,
    },
    ir::physical_properties::InterfaceDirection,
};
use tydi_common::{
    name::{Name, PathName},
    numbers::{NonNegative, Positive, PositiveReal},
};

use crate::{
    lex::{Operator, Token, TypeKeyword},
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

impl fmt::Display for HashablePositiveReal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get())
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

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Path(p) => write!(f, "{}", p.to_string_lossy()),
            Value::Synchronicity(s) => write!(f, "{}", s),
            Value::Direction(d) => write!(f, "{}", d),
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(ff) => write!(f, "{}", ff),
            Value::Version(v) => write!(f, "{}", v),
            Value::Boolean(b) => write!(f, "{}", b),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum IdentExpr {
    Name(Spanned<String>),
    PathName(Vec<Spanned<String>>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PortDef {
    mode: Spanned<InterfaceDirection>,
    typ: Box<Spanned<Expr>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Expr {
    Error,
    Value(Value),
    Ident(IdentExpr),
    TypeDef(LogicalTypeExpr),
    Documentation(Spanned<String>, Box<Spanned<Self>>),
    PortList(Vec<(Spanned<String>, PortDef)>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TypeDecl {
    name: Spanned<Name>,
    typ: Spanned<LogicalTypeExpr>,
}

// Before eval
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum LogicalTypeExpr {
    Null,
    Bits(Box<Spanned<Expr>>),
    Group(Vec<(Spanned<String>, Spanned<Expr>)>),
    Union(Vec<(Spanned<String>, Spanned<Expr>)>),
    Stream(Vec<(Spanned<String>, Spanned<Expr>)>),
}

// After eval
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum LogicalType {
    Null,
    Bits(Positive),
    Group(Vec<(Spanned<Name>, Spanned<LogicalTypeExpr>)>),
    Union(Vec<(Spanned<Name>, Spanned<LogicalTypeExpr>)>),
    Stream(StreamType),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StreamType {
    data: Box<Spanned<Expr>>,
    throughput: Spanned<Throughput>,
    dimensionality: Spanned<NonNegative>,
    synchronicity: Spanned<Synchronicity>,
    complexity: Spanned<Complexity>,
    direction: Spanned<Direction>,
    user: Box<Spanned<Expr>>,
    keep: Spanned<bool>,
}

fn expr_parser() -> impl Parser<Token, Spanned<Expr>, Error = Simple<Token>> + Clone {
    recursive(|expr| {
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

            let raw_val = filter_map(|span, tok| match tok {
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
            .labelled("value");

            let val = raw_val.map(Expr::Value);

            let name = filter_map(|span, tok| match tok {
                Token::Identifier(ident) => Ok((ident, span)),
                _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
            })
            .labelled("name");

            let path_name = name
                .clone()
                .chain(
                    just(Token::Op(Operator::Path))
                        .ignore_then(name.clone())
                        .repeated()
                        .at_least(1),
                )
                .map(|pth| IdentExpr::PathName(pth));

            let ident = path_name
                .or(name.map(IdentExpr::Name))
                .labelled("identifier");

            let port_def = filter_map(|span, tok| match tok {
                Token::PortMode(mode) => Ok(mode),
                _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
            })
            .map_with_span(|mode, span| (mode, span))
            .then(expr.clone())
            .map(|(mode, typ)| PortDef {
                mode,
                typ: Box::new(typ),
            });

            let named_item = name
                .clone()
                .then_ignore(just(Token::Ctrl(':')))
                .then(expr.clone());

            // A list of named expressions
            let named_items = named_item
                .clone()
                .chain(
                    just(Token::Ctrl(','))
                        .ignore_then(named_item.clone())
                        .repeated(),
                )
                .then_ignore(just(Token::Ctrl(',')).or_not())
                .or_not()
                .map(|item| item.unwrap_or_else(Vec::new));

            let group_def = named_items
                .clone()
                .delimited_by(Token::Ctrl('('), Token::Ctrl(')'));

            let type_def = 
            // Null
            just(Token::Type(TypeKeyword::Null))
                .to(LogicalTypeExpr::Null)
                // Bits
                .or(just(Token::Type(TypeKeyword::Bits))
                    .ignore_then(
                        expr.clone()
                            .delimited_by(Token::Ctrl('('), Token::Ctrl(')')),
                    )
                    .map(|e| LogicalTypeExpr::Bits(Box::new(e))))
                // Group
                .or(just(Token::Type(TypeKeyword::Group))
                    .ignore_then(group_def.clone())
                    .map(|g| LogicalTypeExpr::Group(g)))
                // Union
                .or(just(Token::Type(TypeKeyword::Union))
                    .ignore_then(group_def.clone())
                    .map(|g| LogicalTypeExpr::Union(g)))
                // Stream
                .or(just(Token::Type(TypeKeyword::Stream))
                    .ignore_then(group_def.clone())
                    .map(|g| LogicalTypeExpr::Stream(g)))
                .map(Expr::TypeDef);

            let port = name
                .clone()
                .then_ignore(just(Token::Ctrl(':')))
                .then(port_def.clone())
                .labelled("port definition");

            let port_list = port
                .clone()
                .chain(just(Token::Ctrl(',')).ignore_then(port.clone()).repeated())
                .then_ignore(just(Token::Ctrl(',')).or_not())
                .or_not()
                .map(|item| item.unwrap_or_else(Vec::new))
                .delimited_by(Token::Ctrl('('), Token::Ctrl(')'))
                .map(Expr::PortList);

            ident
                .map(Expr::Ident)
                .or(doc)
                .or(val)
                .or(type_def)
                .or(port_list)
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
                ))
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

        // println!("{:#?}", tokens);

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
    fn test_doc_expr() {
        test_expr_parse(
            r#"#doc
doc doc# 1.3"#,
        )
    }

    #[test]
    fn test_port_list() {
        test_expr_parse("(port: in a, port: out b)")
    }

    #[test]
    fn test_invalid_port_list() {
        test_expr_parse("(port: a, port: out b)")
    }

    #[test]
    fn test_typedefs() {
        test_expr_parse("Null");
        test_expr_parse("Bits(23)");
        test_expr_parse("Group(a: Bits(32), b: path::name)");
        test_expr_parse("Union()");
        test_expr_parse("Stream (
        data: rgb,
        throughput: 2.0,
        dimensionality: 0,
        synchronicity: Sync,
        complexity: 4,
        direction: Forward,
        user: Null, // It's possible to use type definitions directly
        keep: false,
    )");
    }

    #[test]
    fn playground() {
        test_expr_parse("Bits(0)");
    }
}
