use chumsky::prelude::*;
use std::{fmt, hash::Hash};
use til_query::common::{
    logical::logicaltype::stream::{Synchronicity, Throughput},
    stream_direction::StreamDirection,
};
use tydi_common::numbers::{NonNegative, Positive, PositiveReal};

use crate::{
    ident_expr::{ident_expr, IdentExpr},
    impl_expr::{streamlet_impl_expr, StreamletImplExpr},
    interface_expr::{interface_expr, InterfaceExpr},
    lex::{DeclKeyword, Token},
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
    Direction(StreamDirection),
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
pub enum Expr {
    Error,
    Ident(IdentExpr),
    Documentation(Spanned<String>, Box<Spanned<Self>>),
    StreamletProps(Vec<(Spanned<Token>, StreamletProperty)>),
    StreamletDef(Spanned<InterfaceExpr>, Option<Box<Spanned<Self>>>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum StreamletProperty {
    Implementation(Spanned<StreamletImplExpr>),
}

pub fn doc_parser() -> impl Parser<Token, Spanned<String>, Error = Simple<Token>> + Clone {
    filter_map(|span, tok| match tok {
        Token::Documentation(docstr) => Ok(docstr.clone()),
        _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
    })
    .map_with_span(|body, span| (body, span))
    .labelled("documentation")
}

pub fn val() -> impl Parser<Token, Spanned<Value>, Error = Simple<Token>> + Clone {
    filter_map(|span: Span, tok| match tok {
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
                    format!(
                        "Lexer error: {} is neither an integer nor a positive float.",
                        num
                    ),
                ))
            }
        }
        Token::Synchronicity(synch) => Ok(Value::Synchronicity(synch)),
        Token::Direction(dir) => Ok(Value::Direction(dir)),
        Token::Version(ver) => Ok(Value::Version(ver)),
        Token::Boolean(boolean) => Ok(Value::Boolean(boolean)),
        _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
    })
    .map_with_span(|v, span| (v, span))
}

pub fn expr_parser() -> impl Parser<Token, Spanned<Expr>, Error = Simple<Token>> + Clone {
    // ......
    // IDENTITIES
    // ......

    let ident_expr = ident_expr()
        .map(Expr::Ident)
        .map_with_span(|i, span| (i, span));

    // ......
    // STREAMLET DEFINITIONS
    // ......

    let impl_prop = just(Token::Decl(DeclKeyword::Implementation))
        .map_with_span(|tok, span| (tok, span))
        .then_ignore(just(Token::Ctrl(':')))
        .then(streamlet_impl_expr())
        .map(|(lab, i)| (lab, StreamletProperty::Implementation(i)));

    // In case more properties are added in the future, use a generic type
    let streamlet_prop = impl_prop;

    // Then require at least one property
    let streamlet_props = streamlet_prop
        .clone()
        .chain(
            just(Token::Ctrl(','))
                .ignore_then(streamlet_prop)
                .repeated(),
        )
        .then_ignore(just(Token::Ctrl(',')).or_not())
        .delimited_by(just(Token::Ctrl('{')), just(Token::Ctrl('}')))
        .map_with_span(|p, span| (Expr::StreamletProps(p), span))
        .recover_with(nested_delimiters(
            Token::Ctrl('{'),
            Token::Ctrl('}'),
            [],
            |span| (Expr::Error, span),
        ));

    let streamlet_def = interface_expr()
        .then(streamlet_props.map(|x| Box::new(x)).or_not())
        .map(|(i, p)| Expr::StreamletDef(i, p))
        .map_with_span(|s, span| (s, span));

    // Note: Streamlet definitions can not have documentation, but streamlet declarations can.

    // ......
    // RESULT
    // Valid expressions are:
    // * Type definitions
    // * Interface definitions
    // * Implementation definitions
    // * Streamlet definitions
    // All of which can be identities
    // ......

    streamlet_def
        .or(ident_expr)
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

    use chumsky::Stream;

    use crate::{lex::lexer, report::report_errors};

    use super::*;

    fn source(path: impl AsRef<Path>) -> String {
        std::fs::read_to_string(path).unwrap()
    }

    fn test_expr_parse(src: impl Into<String>) {
        let src = src.into();
        let (tokens, errs) = lexer().parse_recovery(src.as_str());

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

        report_errors(&src, errs, parse_errs);
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
            r#"(#doc
doc doc# some_port: in a)"#,
        )
    }

    #[test]
    fn test_port_list() {
        test_expr_parse("(port: in a, port: out b)")
    }

    #[test]
    fn test_port_list_empty_dom() {
        test_expr_parse("<>(port: in a, port: out b)")
    }

    #[test]
    fn test_port_list_with_dom() {
        test_expr_parse("<'a, 'b>(port: in a 'b, port: out b 'a)")
    }

    #[test]
    fn test_invalid_port_list() {
        test_expr_parse("(port: a, port: out b)")
    }

    #[test]
    fn test_invalid_dom_list() {
        test_expr_parse("<'a, b>(port: in a, port: out b)")
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
