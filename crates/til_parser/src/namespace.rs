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
    expr::{expr_parser, Expr},
    ident_expr::{ident_expr_parser, name_parser, path_name_parser},
    lex::{DeclKeyword, Operator, Token, TypeKeyword},
    Span, Spanned,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Decl {
    TypeDecl(Spanned<String>, Box<Spanned<Expr>>),
    ImplDecl(Spanned<String>, Box<Spanned<Expr>>),
    InterfaceDecl(Spanned<String>, Box<Spanned<Expr>>),
    StreamletDecl(Spanned<String>, Box<Spanned<Expr>>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Statement {
    Import,
    Decl(Decl),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Namespace {
    name: Spanned<Vec<Spanned<String>>>,
    stats: Vec<Spanned<Statement>>,
}

impl Namespace {
    pub fn name_span(&self) -> &Span {
        &self.name.1
    }

    pub fn stats(&self) -> &Vec<Spanned<Statement>> {
        &self.stats
    }
}

pub fn namespaces_parser(
) -> impl Parser<Token, HashMap<Vec<String>, Namespace>, Error = Simple<Token>> + Clone {
    let namespace_name = path_name_parser().map_with_span(|p, span| (p, span));
    let name = name_parser();

    let type_decl = just(Token::Decl(DeclKeyword::LogicalType))
        .ignore_then(name.clone())
        .then_ignore(just(Token::Op(Operator::Declare)))
        .then(expr_parser())
        .then_ignore(just(Token::Ctrl(';')))
        .map(|(n, e)| Decl::TypeDecl(n, Box::new(e)));

    let impl_decl = just(Token::Decl(DeclKeyword::Implementation))
        .ignore_then(name.clone())
        .then_ignore(just(Token::Op(Operator::Declare)))
        .then(expr_parser())
        .then_ignore(just(Token::Ctrl(';')))
        .map(|(n, e)| Decl::ImplDecl(n, Box::new(e)));

    let interface_decl = just(Token::Decl(DeclKeyword::Interface))
        .ignore_then(name.clone())
        .then_ignore(just(Token::Op(Operator::Declare)))
        .then(expr_parser())
        .then_ignore(just(Token::Ctrl(';')))
        .map(|(n, e)| Decl::InterfaceDecl(n, Box::new(e)));

    let streamlet_decl = just(Token::Decl(DeclKeyword::Streamlet))
        .ignore_then(name.clone())
        .then_ignore(just(Token::Op(Operator::Declare)))
        .then(expr_parser())
        .then_ignore(just(Token::Ctrl(';')))
        .map(|(n, e)| Decl::StreamletDecl(n, Box::new(e)));

    let decl = type_decl
        .or(impl_decl)
        .or(interface_decl)
        .or(streamlet_decl)
        .map_with_span(|d, span| (Statement::Decl(d), span));

    let stat = decl; // TODO: Or import

    let namespace = just(Token::Decl(DeclKeyword::Namespace))
        .ignore_then(namespace_name)
        .then(
            stat.clone()
                .repeated()
                .delimited_by(Token::Ctrl('{'), Token::Ctrl('}')),
        )
        .map(|(name, stats)| {
            let (n, span) = name.clone();
            (
                (
                    n.into_iter().map(|(part, _)| part).collect::<Vec<String>>(),
                    span,
                ),
                Namespace { name, stats },
            )
        });

    namespace
        .repeated()
        .try_map(|ns, _| {
            let mut namespaces = HashMap::new();
            for ((name, name_span), n) in ns {
                if namespaces.insert(name.clone(), n).is_some() {
                    return Err(Simple::custom(
                        name_span.clone(),
                        format!("Namespace '{}' already exists", name.join("::")),
                    ));
                }
            }
            Ok(namespaces)
        })
        .then_ignore(end())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::{lex::lexer, report::report_errors};

    use super::*;

    fn source(path: impl AsRef<Path>) -> String {
        std::fs::read_to_string(path).unwrap()
    }

    fn test_namespace_parse(src: impl Into<String>) {
        let src = src.into();
        let (tokens, mut errs) = lexer().parse_recovery(src.as_str());

        // println!("{:#?}", tokens);

        let parse_errs = if let Some(tokens) = tokens {
            // println!("Tokens = {:?}", tokens);
            let len = src.chars().count();
            let (ast, parse_errs) = namespaces_parser()
                .parse_recovery(Stream::from_iter(len..len + 1, tokens.into_iter()));

            println!("{:#?}", ast);

            parse_errs
        } else {
            Vec::new()
        };

        report_errors(&src, errs, parse_errs);
    }

    #[test]
    fn test_test_nspace_til() {
        test_namespace_parse(source("test_nspace.til"))
    }
}