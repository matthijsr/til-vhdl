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

    name.map(IdentExpr::Name).or(path_name)
}
