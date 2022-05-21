use chumsky::{prelude::Simple, Parser};

use crate::{
    ident_expr::{domain_name, ident_expr, name, IdentExpr},
    lex::{Operator, Token},
    Spanned,
};

use chumsky::prelude::*;
use std::hash::Hash;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DomainAssignments {
    Error,
    None,
    List(Vec<(Option<Spanned<String>>, Spanned<String>)>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum StructStat {
    Error,
    Documentation(Spanned<String>, Box<Spanned<Self>>),
    Instance(
        Spanned<String>,
        Spanned<IdentExpr>,
        Spanned<DomainAssignments>,
    ),
    Connection(Spanned<PortSel>, Spanned<PortSel>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PortSel {
    Own(String),
    Instance(Spanned<String>, Spanned<String>),
}

pub fn domain_assignments(
) -> impl Parser<Token, Spanned<DomainAssignments>, Error = Simple<Token>> + Clone {
    let domain_assignment = domain_name()
        .clone()
        .then(
            just(Token::Op(Operator::Declare))
                .ignore_then(domain_name())
                .or_not(),
        )
        .map(|(left, right)| match right {
            // <'instance_domain = 'parent_domain>
            Some(right) => (Some(left), right),
            // <'parent_domain>
            None => (None, left),
        });

    domain_assignment
        .clone()
        .chain(
            just(Token::Ctrl(','))
                .ignore_then(domain_assignment)
                .repeated(),
        )
        .then_ignore(just(Token::Ctrl(',')).or_not())
        .delimited_by(just(Token::Ctrl('<')), just(Token::Ctrl('>')))
        .or_not()
        .map_with_span(|x, span| {
            (
                match x {
                    Some(list) => DomainAssignments::List(list),
                    None => DomainAssignments::None,
                },
                span,
            )
        })
        .recover_with(nested_delimiters(
            Token::Ctrl('<'),
            Token::Ctrl('>'),
            [],
            |span| (DomainAssignments::Error, span),
        ))
}

pub fn struct_parser() -> impl Parser<Token, Spanned<StructStat>, Error = Simple<Token>> + Clone {
    let ident = ident_expr();

    let instance = name()
        .then_ignore(just(Token::Op(Operator::Declare)))
        .then(ident.clone().map_with_span(|i, span| (i, span)))
        //.then(domain_assignments())
        .map(|(i_name, streamlet_name)| {
            StructStat::Instance(i_name, streamlet_name, (DomainAssignments::None, 0..0))
        });

    let portsel = name()
        .then(
            just(Token::Op(Operator::Select))
                .ignore_then(name())
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
        .then(portsel)
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
        .then(stat.clone())
        .map(|(body, subj)| StructStat::Documentation(body, Box::new(subj)))
        .map_with_span(|expr, span| (expr, span));

    stat.or(doc)
}

// TODO: Also do an eval, to confirm the ports and streamlets actually exist

#[cfg(test)]
mod tests {
    use chumsky::Stream;

    use crate::lex::lexer;

    use super::*;

    type Assert = Result<(), String>;

    fn simple_parse(src: impl Into<String>) -> Result<Spanned<StructStat>, String> {
        let src = src.into();
        let tokens = lexer().parse(src.as_str());
        match tokens {
            Ok(tokens) => {
                let len = src.chars().count();
                match struct_parser().parse(Stream::from_iter(len..len + 1, tokens.into_iter())) {
                    Ok(stat) => Ok(stat),
                    Err(err) => Err(format!("{:#?}", err)),
                }
            }
            Err(err) => Err(format!("{:#?}", err)),
        }
    }

    fn assert_ast_eq(expected: StructStat, actual: Result<Spanned<StructStat>, String>) -> Assert {
        match actual {
            Ok((actual, _)) => {
                if actual == expected {
                    Ok(())
                } else {
                    Err(format!("Expected: {:#?}, Actual: {:#?}", expected, actual))
                }
            }
            Err(err) => Err(err),
        }
    }

    #[test]
    fn test_conn_parse() -> Assert {
        assert_ast_eq(
            StructStat::Connection(
                (
                    PortSel::Instance(("a".to_string(), 0..1), ("a".to_string(), 2..4)),
                    0..4,
                ),
                (PortSel::Own("b".to_string()), 7..8),
            ),
            simple_parse("a.a -- b;"),
        )
    }
}
