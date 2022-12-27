use super::Span;

use chumsky::prelude::*;
use core::fmt;
use til_query::{
    common::{logical::logicaltype::stream::Synchronicity, stream_direction::StreamDirection},
    ir::physical_properties::InterfaceDirection,
};

pub struct LexerError {
    pub span: Span,
    pub msg: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DeclKeyword {
    Streamlet,
    Implementation,
    LogicalType,
    Namespace,
    Interface,
}

impl fmt::Display for DeclKeyword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeclKeyword::Streamlet => write!(f, "streamlet"),
            DeclKeyword::Implementation => write!(f, "impl"),
            DeclKeyword::LogicalType => write!(f, "type"),
            DeclKeyword::Namespace => write!(f, "namespace"),
            DeclKeyword::Interface => write!(f, "interface"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ImportKeyword {
    /// `import`
    Import,
    /// `as`
    As,
    /// `prefixed`
    Prefixed,
}

impl fmt::Display for ImportKeyword {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ImportKeyword::Import => write!(f, "import"),
            ImportKeyword::As => write!(f, "as"),
            ImportKeyword::Prefixed => write!(f, "prefixed"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TypeKeyword {
    Bits,
    Group,
    Union,
    Stream,
    Null,
}

impl fmt::Display for TypeKeyword {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TypeKeyword::Bits => write!(f, "Bits"),
            TypeKeyword::Group => write!(f, "Group"),
            TypeKeyword::Union => write!(f, "Union"),
            TypeKeyword::Stream => write!(f, "Stream"),
            TypeKeyword::Null => write!(f, "Null"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ConditionKeyword {
    /// and
    And,
    /// or
    Or,
    /// not
    Not,
    /// one_of
    OneOf,
}

impl fmt::Display for ConditionKeyword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConditionKeyword::And => write!(f, "and"),
            ConditionKeyword::Or => write!(f, "or"),
            ConditionKeyword::Not => write!(f, "not"),
            ConditionKeyword::OneOf => write!(f, "one_of"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Operator {
    /// `=`
    Eq,
    /// `.`
    Select,
    /// `--`
    Connect,
    /// `::`
    Path,
    /// `>=`
    GtEq,
    /// `<=`
    LtEq,
    /// `+`
    Add,
    /// `-`
    Sub,
    /// `*`
    Mul,
    /// `/`
    Div,
    /// `%`
    Mod,
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operator::Eq => write!(f, "="),
            Operator::Select => write!(f, "."),
            Operator::Connect => write!(f, "--"),
            Operator::Path => write!(f, "::"),
            Operator::GtEq => write!(f, ">="),
            Operator::LtEq => write!(f, "<="),
            Operator::Add => write!(f, "+"),
            Operator::Sub => write!(f, "-"),
            Operator::Mul => write!(f, "*"),
            Operator::Div => write!(f, "/"),
            Operator::Mod => write!(f, "%"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Token {
    /// Identifiers: Names and parts of PathNames
    Identifier(String),
    /// `"../path"`, TIL does not use strings for any other purpose.
    Path(String),
    /// Import keywords: `import`, `as`, `prefixed`
    Import(ImportKeyword),
    /// Type keywords: `Bits`, `Group`, `Union`, `Stream`, `Null`
    Type(TypeKeyword),
    /// Synchronicity keywords: `Sync`, `Flatten`, `Desync`, `FlatDesync`
    Synchronicity(Synchronicity),
    /// Direction keywords: `Forward`, `Reverse`
    Direction(StreamDirection),
    /// Words that precede declarations (e.g., `namespace`, `impl`)
    Decl(DeclKeyword),
    /// Operators `=` `.` `--` `::` `*`
    Op(Operator),
    /// Control characters: `(` `)` `{` `}` `:` `,` `;`
    Ctrl(char),
    /// Documentation delineated by /* */
    Documentation(String),
    /// Natural or non-negative floating point number
    Num(String),
    /// Version number, e.g. 7.2.1
    Version(String),
    /// `true` and `false`, for the `keep` of Streams
    Boolean(bool),
    /// `in` and `out` for ports
    PortMode(InterfaceDirection),
    /// Words pertaining to conditions
    Condition(ConditionKeyword),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Identifier(s) => write!(f, "{}", s),
            Token::Path(s) => write!(f, "{}", s),
            Token::Import(i) => write!(f, "{}", i),
            Token::Type(t) => write!(f, "{}", t),
            Token::Synchronicity(s) => write!(f, "{}", s),
            Token::Direction(d) => write!(f, "{}", d),
            Token::Decl(d) => write!(f, "{}", d),
            Token::Op(o) => write!(f, "{}", o),
            Token::Ctrl(c) => write!(f, "{}", c),
            Token::Documentation(s) => write!(f, "{}", s),
            Token::Num(s) => write!(f, "{}", s),
            Token::Version(s) => write!(f, "{}", s),
            Token::Boolean(b) => write!(f, "{}", b),
            Token::PortMode(p) => write!(f, "{}", p),
            Token::Condition(c) => write!(f, "{}", c),
        }
    }
}

pub fn lexer() -> impl Parser<char, Vec<(Token, Span)>, Error = Simple<char>> {
    let num = text::int(10)
        .chain::<char, _, _>(just('.').chain(text::digits(10)).or_not().flatten())
        .collect::<String>()
        .map(Token::Num);

    // As versions and numbers overlap, a version must have at least 2 subversion levels (i.e., 4.3.2 is a version, but 4.3 is not)
    let ver = text::int(10)
        .chain::<char, _, _>(
            just('.')
                .chain(text::digits(10))
                .repeated()
                .at_least(2)
                .flatten(),
        )
        .collect::<String>()
        .map(Token::Version);

    let path_ = just('"')
        .ignore_then(filter(|c| *c != '"').repeated())
        .then_ignore(just('"'))
        .collect::<String>()
        .map(Token::Path);

    let op = just("=")
        .to(Operator::Eq)
        .or(just(".").to(Operator::Select))
        .or(just("--").to(Operator::Connect))
        .or(just("::").to(Operator::Path))
        .or(just(">=").to(Operator::GtEq))
        .or(just("<=").to(Operator::LtEq))
        .or(just("+").to(Operator::Add))
        .or(just("-").to(Operator::Sub))
        .or(just("*").to(Operator::Mul))
        .or(just("/").to(Operator::Div))
        .or(just("%").to(Operator::Mod))
        .map(Token::Op);

    let ctrl = one_of("(){}:,;'<>").map(|c| Token::Ctrl(c));

    let doc = filter(|c| *c != '#')
        .repeated()
        .delimited_by(just('#'), just('#'))
        .collect::<String>()
        .map(Token::Documentation);

    let ident = text::ident().map(|ident: String| match ident.as_str() {
        "import" => Token::Import(ImportKeyword::Import),
        "as" => Token::Import(ImportKeyword::As),
        "prefixed" => Token::Import(ImportKeyword::Prefixed),
        "Bits" => Token::Type(TypeKeyword::Bits),
        "Group" => Token::Type(TypeKeyword::Group),
        "Union" => Token::Type(TypeKeyword::Union),
        "Stream" => Token::Type(TypeKeyword::Stream),
        "Null" => Token::Type(TypeKeyword::Null),
        "Sync" => Token::Synchronicity(Synchronicity::Sync),
        "Flatten" => Token::Synchronicity(Synchronicity::Flatten),
        "Desync" => Token::Synchronicity(Synchronicity::Desync),
        "FlatDesync" => Token::Synchronicity(Synchronicity::FlatDesync),
        "Forward" => Token::Direction(StreamDirection::Forward),
        "Reverse" => Token::Direction(StreamDirection::Reverse),
        "streamlet" => Token::Decl(DeclKeyword::Streamlet),
        "impl" => Token::Decl(DeclKeyword::Implementation),
        "type" => Token::Decl(DeclKeyword::LogicalType),
        "namespace" => Token::Decl(DeclKeyword::Namespace),
        "interface" => Token::Decl(DeclKeyword::Interface),
        "true" => Token::Boolean(true),
        "false" => Token::Boolean(false),
        "in" => Token::PortMode(InterfaceDirection::In),
        "out" => Token::PortMode(InterfaceDirection::Out),
        "and" => Token::Condition(ConditionKeyword::And),
        "or" => Token::Condition(ConditionKeyword::Or),
        "not" => Token::Condition(ConditionKeyword::Not),
        "one_of" => Token::Condition(ConditionKeyword::OneOf),
        _ => Token::Identifier(ident),
    });

    let token = doc
        .or(ver)
        .or(num)
        .or(path_)
        .or(op)
        .or(ctrl)
        .or(ident)
        .recover_with(skip_then_retry_until([]));

    let single_line = just("//").then(take_until(text::newline())).ignored();
    let multi_line = just("///").then(take_until(just("///"))).ignored();
    let comment = multi_line.or(single_line);

    token
        .padded()
        .padded_by(comment.padded().repeated())
        .map_with_span(|tok, span| (tok, span))
        .repeated()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chumsky::Parser;
    use std::path::Path;

    fn test_lex(path: impl AsRef<Path>) {
        let src = std::fs::read_to_string(path).unwrap();

        let (tokens, errs) = lexer().parse_recovery(src.as_str());

        if let Some(tokens) = tokens {
            println!("Tokens:");
            for token in tokens {
                println!("{:?}", token);
            }
        }

        let err_len = errs.len();
        println!("Errors:");
        for err in errs {
            println!("{}", err);
        }
        assert_eq!(0, err_len);
    }

    #[test]
    fn test_test_til() {
        test_lex("test.til")
    }

    #[test]
    fn test_sample_til() {
        test_lex("sample.til")
    }

    #[test]
    fn test_generics_til() {
        test_lex("generics.til")
    }
}
