use super::Span;
use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};
use chumsky::{prelude::*, stream::Stream};
use std::{collections::HashMap, env, fmt, fs, path::PathBuf};
use til_query::common::logical::logicaltype::stream::{Direction, Synchronicity};
use tydi_common::name::{Name, PathName};

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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TypeKeyword {
    Bits,
    Group,
    Union,
    Stream,
    Null,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Operator {
    /// `=`
    Declare,
    /// `.`
    Select,
    /// `--`
    Connect,
    /// `::`
    Path,
    /// `*`
    All,
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
    Direction(Direction),
    /// Words that precede declarations (e.g., `namespace`, `impl`)
    Decl(DeclKeyword),
    /// Operators `=` `.` `--` `::` `*`
    Op(Operator),
    /// Control characters: `(` `)` `{` `}` `:` `,` `;`
    Ctrl(char),
    /// Documentation delineated by /* */
    Documentation(String),
    /// Integer or floating point number
    Num(String),
    /// Version number, e.g. 7.2.1
    Version(String),
    /// `true` and `false`, for the `keep` of Streams
    Boolean(bool),
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
        .to(Operator::Declare)
        .or(just(".").to(Operator::Select))
        .or(just("--").to(Operator::Connect))
        .or(just("::").to(Operator::Path))
        .or(just("*").to(Operator::All))
        .map(Token::Op);

    let ctrl = one_of("(){}:,;").map(|c| Token::Ctrl(c));

    let doc = just('#')
        .ignore_then(filter(|c| *c != '#').repeated())
        .then_ignore(just('#'))
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
        "Forward" => Token::Direction(Direction::Forward),
        "Reverse" => Token::Direction(Direction::Reverse),
        "streamlet" => Token::Decl(DeclKeyword::Streamlet),
        "impl" => Token::Decl(DeclKeyword::Implementation),
        "type" => Token::Decl(DeclKeyword::LogicalType),
        "namespace" => Token::Decl(DeclKeyword::Namespace),
        "true" => Token::Boolean(true),
        "false" => Token::Boolean(false),
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
