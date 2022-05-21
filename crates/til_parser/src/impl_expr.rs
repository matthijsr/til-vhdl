use chumsky::prelude::*;

use crate::{
    doc_expr::{doc_expr, DocExpr},
    ident_expr::{ident_expr, IdentExpr},
    interface_expr::{interface_expr, InterfaceExpr},
    lex::Token,
    struct_parse::{struct_parser, StructStat},
    Spanned,
};

// Implementation definitions without ports. Used when defining an implementation on a streamlet directly.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ImplBodyExpr {
    Error,
    // Structural
    Struct(DocExpr, Vec<Spanned<StructStat>>),
    // Path
    Link(String),
}

pub fn impl_body_expr() -> impl Parser<Token, Spanned<ImplBodyExpr>, Error = Simple<Token>> + Clone
{
    let behav = filter_map(|span, tok| match tok {
        Token::Path(pth) => Ok(ImplBodyExpr::Link(pth)),
        _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
    })
    .labelled("behavioural impl path")
    .map_with_span(|ri, span| (ri, span));

    let struct_bod = doc_expr()
        .then(
            struct_parser()
                .repeated()
                .delimited_by(just(Token::Ctrl('{')), just(Token::Ctrl('}'))),
        )
        .map(|(doc, stats)| ImplBodyExpr::Struct(doc, stats))
        .map_with_span(|ri, span| (ri, span))
        .recover_with(nested_delimiters(
            Token::Ctrl('{'),
            Token::Ctrl('}'),
            [],
            |span| (ImplBodyExpr::Error, span),
        ));

    behav.or(struct_bod)
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ImplDefExpr {
    Identity(IdentExpr),
    Def(Spanned<InterfaceExpr>, Spanned<ImplBodyExpr>),
}

pub fn impl_def_expr() -> impl Parser<Token, Spanned<ImplDefExpr>, Error = Simple<Token>> + Clone {
    let impl_def = interface_expr()
        .then(impl_body_expr())
        .map(|(iface, bod)| ImplDefExpr::Def(iface, bod));

    impl_def
        .or(ident_expr().map(ImplDefExpr::Identity))
        .map_with_span(|x, span| (x, span))
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum StreamletImplExpr {
    Identity(IdentExpr),
    Def(Spanned<ImplBodyExpr>),
}

pub fn streamlet_impl_expr(
) -> impl Parser<Token, Spanned<StreamletImplExpr>, Error = Simple<Token>> + Clone {
    impl_body_expr()
        .map(StreamletImplExpr::Def)
        .or(ident_expr().map(StreamletImplExpr::Identity))
        .map_with_span(|x, span| (x, span))
}
