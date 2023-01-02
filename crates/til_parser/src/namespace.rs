use chumsky::prelude::*;
use std::hash::Hash;

use crate::{
    doc_expr::{doc_expr, DocExpr},
    expr::{doc_parser, expr_parser, Expr},
    generic_param::{generic_parameters, GenericParameterList},
    ident_expr::{ident_expr, name, path_name, IdentExpr},
    impl_expr::{impl_def_expr, ImplDefExpr},
    interface_expr::{interface_expr, InterfaceExpr},
    lex::{DeclKeyword, ImportKeyword, Operator, Token},
    type_expr::{type_expr, TypeExpr},
    Span, Spanned,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Decl {
    TypeDecl(
        Spanned<String>,
        Spanned<TypeExpr>,
        Spanned<GenericParameterList>,
    ),
    ImplDecl(DocExpr, Spanned<String>, Spanned<ImplDefExpr>),
    InterfaceDecl(Spanned<String>, Spanned<InterfaceExpr>),
    StreamletDecl(Option<String>, Spanned<String>, Box<Spanned<Expr>>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Import {
    /// Import an entire namespace
    FullImport(Spanned<Vec<Spanned<String>>>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Statement {
    Import(Import),
    Decl(Decl),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Namespace {
    name: Spanned<Vec<String>>,
    stats: Vec<Spanned<Statement>>,
}

impl Namespace {
    pub fn name(&self) -> &Vec<String> {
        &self.name.0
    }

    pub fn name_span(&self) -> &Span {
        &self.name.1
    }

    pub fn stats(&self) -> &Vec<Spanned<Statement>> {
        &self.stats
    }
}

pub fn namespaces_parser() -> impl Parser<Token, Vec<Namespace>, Error = Simple<Token>> + Clone {
    let namespace_name = ident_expr()
        .map(|i| match i {
            IdentExpr::Name((n, _)) => vec![n],
            IdentExpr::PathName(p) => p.into_iter().map(|(n, _)| n).collect(),
        })
        .map_with_span(|i, span| (i, span));

    let import_stat = just(Token::Import(ImportKeyword::Import))
        .ignore_then(path_name().map_with_span(|p, span| (p, span)))
        .then_ignore(just(Token::Ctrl(';')))
        .map_with_span(|n, span| (Statement::Import(Import::FullImport(n)), span));

    let type_decl = just(Token::Decl(DeclKeyword::LogicalType))
        .ignore_then(name())
        .then(
            generic_parameters()
                .delimited_by(just(Token::Ctrl('<')), just(Token::Ctrl('>')))
                .map(|x| GenericParameterList::List(x))
                .or_not()
                .map_with_span(|x, span| match x {
                    Some(x) => (x, span),
                    None => (GenericParameterList::None, span),
                })
                .recover_with(nested_delimiters(
                    Token::Ctrl('<'),
                    Token::Ctrl('>'),
                    [],
                    |span| (GenericParameterList::Error, span),
                )),
        )
        .then_ignore(just(Token::Op(Operator::Eq)))
        .then(type_expr())
        .map(|((n, g), e)| Decl::TypeDecl(n, e, g));

    let impl_decl = doc_expr()
        .then(just(Token::Decl(DeclKeyword::Implementation)).ignore_then(name()))
        .then_ignore(just(Token::Op(Operator::Eq)))
        .then(impl_def_expr())
        .map(|((doc, name), body)| Decl::ImplDecl(doc, name, body));

    let interface_decl = just(Token::Decl(DeclKeyword::Interface))
        .ignore_then(name())
        .then_ignore(just(Token::Op(Operator::Eq)))
        .then(interface_expr())
        .map(|(n, e)| Decl::InterfaceDecl(n, e));

    let streamlet_decl = just(Token::Decl(DeclKeyword::Streamlet))
        .ignore_then(name())
        .then_ignore(just(Token::Op(Operator::Eq)))
        .then(expr_parser());
    let doc_streamlet_decl = doc_parser()
        .then(streamlet_decl.clone())
        .map(|((doc, _), (n, e))| Decl::StreamletDecl(Some(doc), n, Box::new(e)));
    let streamlet_decl = doc_streamlet_decl
        .or(streamlet_decl.map(|(n, e)| Decl::StreamletDecl(None, n, Box::new(e))));

    let decl = type_decl
        .or(impl_decl)
        .or(interface_decl)
        .or(streamlet_decl)
        .then_ignore(just(Token::Ctrl(';')))
        .map_with_span(|d, span| (Statement::Decl(d), span));

    let stat = import_stat.or(decl);

    let namespace = just(Token::Decl(DeclKeyword::Namespace))
        .ignore_then(namespace_name)
        .then(
            stat.repeated()
                .delimited_by(just(Token::Ctrl('{')), just(Token::Ctrl('}'))),
        )
        .map(|(name, stats)| Namespace { name, stats });

    namespace.repeated().then_ignore(end())
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

    fn test_namespace_parse(src: impl Into<String>) {
        let src = src.into();
        let (tokens, errs) = lexer().parse_recovery(src.as_str());

        //println!("{:#?}", tokens);

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

        let errs_len = errs.len();
        let parse_err_len = parse_errs.len();
        report_errors(&src, errs, parse_errs);
        assert_eq!(errs_len, 0);
        assert_eq!(parse_err_len, 0);
    }

    #[test]
    fn test_test_nspace_til() {
        test_namespace_parse(source("test_nspace.til"))
    }

    #[test]
    fn test_generics_til() {
        test_namespace_parse(source("generics.til"))
    }

    #[test]
    fn test_simple_generics_til() {
        test_namespace_parse(source("simple_generics.til"))
    }
}
