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
    lex::{DeclKeyword, Operator, Token, TypeKeyword},
    Span, Spanned,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum IdentExpr {
    Name(Spanned<String>),
    PathName(Vec<Spanned<String>>),
}

pub fn name_parser() -> impl Parser<Token, Spanned<String>, Error = Simple<Token>> + Clone {
    filter_map(|span, tok| match tok {
        Token::Identifier(ident) => Ok((ident, span)),
        _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
    })
}

pub fn path_name_parser() -> impl Parser<Token, Vec<Spanned<String>>, Error = Simple<Token>> + Clone
{
    let name = name_parser().labelled("name");

    name.clone().chain(
        just(Token::Op(Operator::Path))
            .ignore_then(name.clone())
            .repeated(),
    )
}

pub fn ident_expr_parser() -> impl Parser<Token, IdentExpr, Error = Simple<Token>> + Clone {
    let name = name_parser().labelled("name");

    let path_name = path_name_parser().map(|pth| IdentExpr::PathName(pth));

    path_name.or(name.map(IdentExpr::Name))
}
