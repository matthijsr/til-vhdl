use chumsky::prelude::*;
use std::hash::Hash;

use crate::{
    lex::{Operator, Token},
    Spanned,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum IdentExpr {
    Name(Spanned<String>),
    PathName(Vec<Spanned<String>>),
}

pub fn name() -> impl Parser<Token, Spanned<String>, Error = Simple<Token>> + Clone {
    filter_map(|span, tok| match tok {
        Token::Identifier(ident) => Ok((ident, span)),
        _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
    })
    .labelled("name")
}

pub fn domain_name() -> impl Parser<Token, Spanned<String>, Error = Simple<Token>> + Clone {
    just(Token::Ctrl('\'')).ignore_then(name().labelled("domain name"))
}

pub fn label() -> impl Parser<Token, Spanned<String>, Error = Simple<Token>> + Clone {
    name().then_ignore(just(Token::Ctrl(':'))).labelled("label")
}

pub fn path_name() -> impl Parser<Token, Vec<Spanned<String>>, Error = Simple<Token>> + Clone {
    name()
        .chain(
            just(Token::Op(Operator::Path))
                .ignore_then(name())
                .repeated()
                .at_least(1),
        )
        .labelled("path name")
}

pub fn ident_expr() -> impl Parser<Token, IdentExpr, Error = Simple<Token>> + Clone {
    path_name()
        .map(IdentExpr::PathName)
        .or(name().map(IdentExpr::Name))
        .labelled("identifier")
}
