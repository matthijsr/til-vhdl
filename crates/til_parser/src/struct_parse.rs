use chumsky::{prelude::Simple, Parser};

use crate::{
    ident_expr::{ident_expr_parser, name_parser, IdentExpr},
    lex::{Operator, Token},
    Spanned,
};
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum StructStat {
    Error,
    Documentation(Spanned<String>, Box<Spanned<Self>>),
    Instance(Spanned<String>, Spanned<IdentExpr>),
    Connection(Spanned<PortSel>, Spanned<PortSel>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PortSel {
    Own(String),
    Instance(Spanned<String>, Spanned<String>),
}

pub fn struct_parser() -> impl Parser<Token, Spanned<StructStat>, Error = Simple<Token>> + Clone {
    let name = name_parser().labelled("name");

    let ident = ident_expr_parser().labelled("identifier");

    let instance = name
        .clone()
        .then_ignore(just(Token::Op(Operator::Declare)))
        .then(ident.clone().map_with_span(|i, span| (i, span)))
        .map(|(i_name, streamlet_name)| StructStat::Instance(i_name, streamlet_name));

    let portsel = name
        .clone()
        .then(
            just(Token::Op(Operator::Select))
                .ignore_then(name.clone())
                .or_not(),
        )
        .map(|(subj, port)| {
            if let Some(port) = port {
                PortSel::Instance(subj, port)
            } else {
                PortSel::Own(subj.0)
            }
        })
        .map_with_span(|p, span| (p, span));

    let conn = portsel
        .clone()
        .then_ignore(just(Token::Op(Operator::Connect)))
        .then(portsel.clone())
        .map(|(left, right)| StructStat::Connection(left, right));

    let stat = instance
        .or(conn)
        .then_ignore(just(Token::Ctrl(';')))
        .map_with_span(|expr, span| (expr, span));

    let doc_body = filter_map(|span, tok| match tok {
        Token::Documentation(docstr) => Ok(docstr.clone()),
        _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
    })
    .map_with_span(|body, span| (body, span))
    .labelled("documentation");

    let doc = doc_body
        .clone()
        .then(stat.clone())
        .map(|(body, subj)| StructStat::Documentation(body, Box::new(subj)))
        .map_with_span(|expr, span| (expr, span));

    stat.or(doc)
}
