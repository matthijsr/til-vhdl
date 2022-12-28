use chumsky::prelude::*;
use til_query::ir::physical_properties::InterfaceDirection;

use crate::{
    doc_expr::{doc_expr, DocExpr},
    ident_expr::{domain_name, ident_expr, label, IdentExpr},
    lex::Token,
    type_expr::{type_expr, TypeExpr},
    Spanned,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PortDef {
    pub doc: DocExpr,
    pub name: Spanned<String>,
    pub props: Spanned<PortProps>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PortProps {
    pub mode: Spanned<InterfaceDirection>,
    pub typ: Spanned<TypeExpr>,
    pub domain: Option<Spanned<String>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PortsDef {
    Error,
    Def(Vec<Spanned<PortDef>>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DomainList {
    Error,
    List(Vec<Spanned<String>>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum InterfaceDef {
    Error,
    Def(Option<Spanned<DomainList>>, Spanned<PortsDef>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum InterfaceExpr {
    Identifier(IdentExpr),
    Definition(Spanned<InterfaceDef>),
}

pub fn domain_list() -> impl Parser<Token, Spanned<DomainList>, Error = Simple<Token>> + Clone {
    domain_name()
        .separated_by(just(Token::Ctrl(',')))
        .at_least(1)
        .allow_trailing()
        .delimited_by(just(Token::Ctrl('<')), just(Token::Ctrl('>')))
        .map_with_span(|list, span| (DomainList::List(list), span))
        .recover_with(nested_delimiters(
            Token::Ctrl('<'),
            Token::Ctrl('>'),
            [],
            |span| (DomainList::Error, span),
        ))
}

pub fn ports_def() -> impl Parser<Token, Spanned<PortsDef>, Error = Simple<Token>> + Clone {
    let port_props = filter_map(|span, tok| match tok {
        Token::PortMode(mode) => Ok(mode),
        _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
    })
    .map_with_span(|mode, span| (mode, span))
    .then(type_expr())
    .then(domain_name().or_not())
    .map(|((mode, typ), dom)| PortProps {
        mode,
        typ,
        domain: dom,
    })
    .map_with_span(|p, span| (p, span))
    .labelled("port properties");

    let port_def = doc_expr()
        .then(label())
        .then(port_props)
        .map(|((doc, name), props)| PortDef { doc, name, props })
        .map_with_span(|p, span| (p, span));

    port_def
        .separated_by(just(Token::Ctrl(',')))
        .allow_trailing()
        .map_with_span(|ports, span| (PortsDef::Def(ports), span))
        .delimited_by(just(Token::Ctrl('(')), just(Token::Ctrl(')')))
        .recover_with(nested_delimiters(
            Token::Ctrl('('),
            Token::Ctrl(')'),
            [],
            |span| (PortsDef::Error, span),
        ))
}

pub fn interface_expr() -> impl Parser<Token, Spanned<InterfaceExpr>, Error = Simple<Token>> + Clone
{
    let interface_def = domain_list()
        .or_not()
        .then(ports_def())
        .map(|(domains, ports)| InterfaceDef::Def(domains, ports))
        .map_with_span(|x, span| (x, span))
        .recover_with(nested_delimiters(
            Token::Ctrl('<'),
            Token::Ctrl('>'),
            [(Token::Ctrl('('), Token::Ctrl(')'))],
            |span| (InterfaceDef::Error, span),
        ))
        .recover_with(nested_delimiters(
            Token::Ctrl('('),
            Token::Ctrl(')'),
            [(Token::Ctrl('<'), Token::Ctrl('>'))],
            |span| (InterfaceDef::Error, span),
        ));

    interface_def
        .map(InterfaceExpr::Definition)
        .or(ident_expr().map(InterfaceExpr::Identifier))
        .map_with_span(|x, span| (x, span))
}
