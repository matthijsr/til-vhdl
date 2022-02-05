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
    ident_expr::{ident_expr_parser, name_parser, IdentExpr},
    lex::{DeclKeyword, Operator, Token, TypeKeyword},
    struct_parse::{struct_parser, StructStat},
    Span, Spanned,
};

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
pub struct PortProps {
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
    PortDef(Spanned<String>, Spanned<PortProps>),
    InterfaceDef(Vec<Spanned<Self>>),
    RawImpl(RawImpl),
    ImplDef(Box<Spanned<Self>>, Box<Spanned<Self>>),
    StreamletProps(Vec<(Spanned<Token>, StreamletProperty)>),
    StreamletDef(Box<Spanned<Self>>, Box<Spanned<Self>>),
}

// Implementation definitions without ports. Used when defining an implementation on a streamlet directly.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum RawImpl {
    // Structural
    Struct(Vec<Spanned<StructStat>>),
    // Path
    Behavioural(String),
}

// Before eval
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum LogicalTypeExpr {
    Null,
    Bits(Spanned<String>),
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum StreamletProperty {
    Implementation(Box<Spanned<Expr>>),
}

pub fn expr_parser() -> impl Parser<Token, Spanned<Expr>, Error = Simple<Token>> + Clone {
    // ......
    // DOCUMENTATION
    // ......

    let doc_body = filter_map(|span, tok| match tok {
        Token::Documentation(docstr) => Ok(docstr.clone()),
        _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
    })
    .map_with_span(|body, span| (body, span))
    .labelled("documentation");

    // ......
    // VALUES
    // ......

    let val = filter_map(|span: Span, tok| match tok {
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
        Token::Synchronicity(synch) => Ok(Value::Synchronicity(synch)),
        Token::Direction(dir) => Ok(Value::Direction(dir)),
        Token::Version(ver) => Ok(Value::Version(ver)),
        Token::Boolean(boolean) => Ok(Value::Boolean(boolean)),
        _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
    })
    .map(Expr::Value)
    .map_with_span(|v, span| (v, span));

    // ......
    // IDENTITIES
    // ......

    let name = name_parser().labelled("name");

    let ident = ident_expr_parser().labelled("identifier");
    let ident_expr = ident
        .clone()
        .map(Expr::Ident)
        .map_with_span(|i, span| (i, span));

    let label = name.clone().then_ignore(just(Token::Ctrl(':')));

    // ......
    // TYPE DEFINITIONS
    // ......

    // Type definitions are recursive, as they can contain other type definitions
    let typ = recursive(|type_def| {
        let typ_el = label.clone().then(type_def.clone());

        // A group of types
        let group_def = typ_el
            .clone()
            .chain(
                just(Token::Ctrl(','))
                    .ignore_then(typ_el.clone())
                    .repeated(),
            )
            .then_ignore(just(Token::Ctrl(',')).or_not())
            .or_not()
            .map(|item| item.unwrap_or_else(Vec::new))
            .delimited_by(Token::Ctrl('('), Token::Ctrl(')'));

        let bits_def = filter_map(|span, tok| match tok {
            Token::Num(num) => Ok((num, span)),
            _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
        })
        .delimited_by(Token::Ctrl('('), Token::Ctrl(')'));

        // Stream properties are either values or types
        let stream_prop = typ_el.or(label.clone().then(val.clone()));

        let stream_def = stream_prop
            .clone()
            .chain(
                just(Token::Ctrl(','))
                    .ignore_then(stream_prop.clone())
                    .repeated(),
            )
            .then_ignore(just(Token::Ctrl(',')).or_not())
            .or_not()
            .map(|item| item.unwrap_or_else(Vec::new))
            .delimited_by(Token::Ctrl('('), Token::Ctrl(')'));

        just(Token::Type(TypeKeyword::Null))
            .to(LogicalTypeExpr::Null)
            .or(just(Token::Type(TypeKeyword::Bits))
                .ignore_then(bits_def)
                .map(|n| LogicalTypeExpr::Bits(n)))
            .or(just(Token::Type(TypeKeyword::Group))
                .ignore_then(group_def.clone())
                .map(|g| LogicalTypeExpr::Group(g)))
            .or(just(Token::Type(TypeKeyword::Union))
                .ignore_then(group_def.clone())
                .map(|g| LogicalTypeExpr::Union(g)))
            .or(just(Token::Type(TypeKeyword::Stream))
                .ignore_then(stream_def)
                .map(|g| LogicalTypeExpr::Stream(g)))
            .map(Expr::TypeDef)
            .map_with_span(|t, span| (t, span))
            // Type defs can be declared with identities
            .or(ident_expr.clone())
    });

    // ......
    // INTERFACE DEFINITIONS
    // ......

    let port_props = filter_map(|span, tok| match tok {
        Token::PortMode(mode) => Ok(mode),
        _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
    })
    .map_with_span(|mode, span| (mode, span))
    .then(typ.clone())
    .map(|(mode, typ)| PortProps {
        mode,
        typ: Box::new(typ),
    })
    .map_with_span(|p, span| (p, span))
    .labelled("port properties");

    let port_def = label
        .clone()
        .then(port_props.clone())
        .map(|(l, p)| Expr::PortDef(l, p))
        .map_with_span(|p, span| (p, span));

    // Individual ports can have documentation
    let doc_port_def = doc_body
        .clone()
        .then(port_def.clone())
        .map(|(body, subj)| Expr::Documentation(body, Box::new(subj)))
        .map_with_span(|d, span| (d, span));
    let port_def = doc_port_def.or(port_def);

    let interface_def = port_def
        .clone()
        .chain(
            just(Token::Ctrl(','))
                .ignore_then(port_def.clone())
                .repeated(),
        )
        .then_ignore(just(Token::Ctrl(',')).or_not())
        .or_not()
        .map(|item| item.unwrap_or_else(Vec::new))
        .delimited_by(Token::Ctrl('('), Token::Ctrl(')'))
        .map(Expr::InterfaceDef)
        .map_with_span(|i, span| (i, span));

    // As with types, interfaces can be declared with identities
    // Note: Interfaces can also be derived from streamlets and implementations
    let interface = interface_def.or(ident_expr.clone());

    // ......
    // IMPLEMENTATION DEFINITIONS
    // ......

    let behav = filter_map(|span, tok| match tok {
        Token::Path(pth) => Ok(RawImpl::Behavioural(pth)),
        _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
    })
    .labelled("behavioural impl path")
    .map(Expr::RawImpl)
    .map_with_span(|ri, span| (ri, span));

    let struct_bod = struct_parser()
        .repeated()
        .delimited_by(Token::Ctrl('{'), Token::Ctrl('}'))
        .map(|stats| RawImpl::Struct(stats))
        .map(Expr::RawImpl)
        .map_with_span(|ri, span| (ri, span))
        .recover_with(nested_delimiters(
            Token::Ctrl('{'),
            Token::Ctrl('}'),
            [],
            |span| (Expr::Error, span),
        ));

    let raw_impl = behav.or(struct_bod);

    // Implementations consist of an interface definition and a structural or behavioural implementation
    let impl_def = interface
        .clone()
        .then(raw_impl.clone())
        .map(|(e, ri)| Expr::ImplDef(Box::new(e), Box::new(ri)))
        .map_with_span(|i, span| (i, span));

    // Implementations can have (overall) documentation
    let doc_impl_def = doc_body
        .clone()
        .then(impl_def.clone())
        .map(|(body, subj)| Expr::Documentation(body, Box::new(subj)))
        .map_with_span(|d, span| (d, span));
    let impl_def = doc_impl_def.or(impl_def);

    // They can also be declared with identities
    let implementation = impl_def.or(ident_expr.clone());

    // ......
    // STREAMLET DEFINITIONS
    // ......

    let impl_prop = just(Token::Decl(DeclKeyword::Implementation))
        .map_with_span(|tok, span| (tok, span))
        .then_ignore(just(Token::Ctrl(':')))
        .then(ident_expr.clone().or(raw_impl.clone()))
        .map(|(lab, i)| (lab, StreamletProperty::Implementation(Box::new(i))));

    // In case more properties are added in the future, use a generic type
    let streamlet_prop = impl_prop;

    // Then require at least one property
    let streamlet_props = streamlet_prop
        .clone()
        .chain(
            just(Token::Ctrl(','))
                .ignore_then(streamlet_prop.clone())
                .repeated(),
        )
        .then_ignore(just(Token::Ctrl(',')).or_not())
        .delimited_by(Token::Ctrl('{'), Token::Ctrl('}'))
        .map_with_span(|p, span| (Expr::StreamletProps(p), span))
        .recover_with(nested_delimiters(
            Token::Ctrl('{'),
            Token::Ctrl('}'),
            [],
            |span| (Expr::Error, span),
        ));

    let streamlet_def = interface
        .clone()
        .then(streamlet_props)
        .map(|(i, p)| Expr::StreamletDef(Box::new(i), Box::new(p)))
        .map_with_span(|s, span| (s, span));

    // Note: Streamlet definitions can not have documentation, but streamlet declarations can.

    // A streamlet's definition can be either an interface definition, or a full streamlet definition with properties
    let streamlet = streamlet_def.or(interface.clone());

    // ......
    // RESULT
    // Valid expressions are:
    // * Type definitions
    // * Interface definitions
    // * Implementation definitions
    // * Streamlet definitions
    // All of which can be identities
    // ......

    streamlet
        .or(implementation)
        .or(interface)
        .or(typ)
        .recover_with(nested_delimiters(
            Token::Ctrl('{'),
            Token::Ctrl('}'),
            [(Token::Ctrl('('), Token::Ctrl(')'))],
            |span| (Expr::Error, span),
        ))
        // Attempt to recover anything that looks like a list but contains errors
        .recover_with(nested_delimiters(
            Token::Ctrl('('),
            Token::Ctrl(')'),
            [(Token::Ctrl('{'), Token::Ctrl('}'))],
            |span| (Expr::Error, span),
        ))
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
        test_expr_parse(
            "Stream (
        data: rgb,
        throughput: 2.0,
        dimensionality: 0,
        synchronicity: Sync,
        complexity: 4,
        direction: Forward,
        user: Null, // It's possible to use type definitions directly
        keep: false,
    )",
        );
    }

    #[test]
    fn test_invalid_struct() {
        test_expr_parse(
            "a {
            a = a::b;
            b = c
            a -- a.a;
            b -- b.a;
        }",
        );
    }

    #[test]
    fn test_impl_def() {
        test_expr_parse("(a: in stream) \"../path\"");
        test_expr_parse("(a: in stream) { a = a; a -- a.a; }");
    }

    #[test]
    fn playground() {
        test_expr_parse("a { impl: a }");
    }
}
