use chumsky::prelude::*;

use crate::{
    expr::{val, Value},
    ident_expr::{ident_expr, label, IdentExpr},
    lex::{Token, TypeKeyword},
    Span, Spanned,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TypeExpr {
    Error,
    Identifier(IdentExpr),
    Definition(Box<Spanned<LogicalTypeDef>>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum LogicalTypeDef {
    Null,
    Bits(Spanned<String>),
    Group(Spanned<FieldsDef>),
    Union(Spanned<FieldsDef>),
    Stream(Spanned<StreamProps>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum FieldsDef {
    Error,
    Fields(Vec<(Spanned<String>, Spanned<TypeExpr>)>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum StreamProp {
    Value(Value),
    Type(TypeExpr),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum StreamProps {
    Error,
    Props(Vec<(Spanned<String>, Spanned<StreamProp>)>),
}

pub fn type_expr() -> impl Parser<Token, Spanned<TypeExpr>, Error = Simple<Token>> + Clone {
    recursive(|type_def| {
        let typ_el = label().then(type_def.clone());

        // A group of types
        let fields_def = typ_el
            .clone()
            .chain(just(Token::Ctrl(',')).ignore_then(typ_el).repeated())
            .then_ignore(just(Token::Ctrl(',')).or_not())
            .or_not()
            .map(|item| item.unwrap_or_else(Vec::new))
            .map_with_span(|fields, span| (FieldsDef::Fields(fields), span))
            .delimited_by(just(Token::Ctrl('(')), just(Token::Ctrl(')')))
            .recover_with(nested_delimiters(
                Token::Ctrl('('),
                Token::Ctrl(')'),
                [],
                |span| (FieldsDef::Error, span),
            ));

        let bits_def = filter_map(|span, tok| match tok {
            Token::Num(num) => Ok((num, span)),
            _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
        })
        .delimited_by(just(Token::Ctrl('(')), just(Token::Ctrl(')')));

        // Stream properties are either values or types
        let stream_prop = label()
            .then(
                type_def
                    .clone()
                    .map(|(t, span)| (StreamProp::Type(t), span))
                    .or(val().map(|(v, span)| (StreamProp::Value(v), span))),
            )
            .map(|(lab, prop)| (lab, prop));

        let stream_props = stream_prop
            .clone()
            .chain(just(Token::Ctrl(',')).ignore_then(stream_prop).repeated())
            .then_ignore(just(Token::Ctrl(',')).or_not())
            .or_not()
            .map(|item| item.unwrap_or_else(Vec::new))
            .delimited_by(just(Token::Ctrl('(')), just(Token::Ctrl(')')))
            .map_with_span(|props, span| (StreamProps::Props(props), span))
            .recover_with(nested_delimiters(
                Token::Ctrl('('),
                Token::Ctrl(')'),
                [],
                |span| (StreamProps::Error, span),
            ));

        let null_def = just(Token::Type(TypeKeyword::Null)).to(LogicalTypeDef::Null);

        let bits_def = just(Token::Type(TypeKeyword::Bits))
            .ignore_then(bits_def)
            .map(|n| LogicalTypeDef::Bits(n));

        let group_def = just(Token::Type(TypeKeyword::Group))
            .ignore_then(fields_def.clone())
            .map(|g| LogicalTypeDef::Group(g));

        let union_def = just(Token::Type(TypeKeyword::Union))
            .ignore_then(fields_def)
            .map(|g| LogicalTypeDef::Union(g));

        let stream_def = just(Token::Type(TypeKeyword::Stream))
            .ignore_then(stream_props)
            .map(|g| LogicalTypeDef::Stream(g));

        let logical_type_def = null_def
            .or(bits_def)
            .or(group_def)
            .or(union_def)
            .or(stream_def)
            .map_with_span(|x, span| (x, span))
            .map(|x| TypeExpr::Definition(Box::new(x)));

        logical_type_def
            .or(ident_expr().map(TypeExpr::Identifier))
            .map_with_span(|t, span| (t, span))
            .recover_with(nested_delimiters(
                Token::Ctrl('('),
                Token::Ctrl(')'),
                [],
                |span| (TypeExpr::Error, span),
            ))
    })
}
