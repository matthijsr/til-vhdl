use chumsky::prelude::*;
use til_query::ir::generics::param_value::GenericParamValue;
use tydi_common::{error::Error, name::Name};

use crate::{
    expr::{val, Value},
    generic_param::generic_parameter_assignments,
    ident_expr::{ident_expr, label, IdentExpr},
    lex::{Token, TypeKeyword},
    Span, Spanned,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TypeExpr {
    Error,
    Identifier(IdentExpr),
    Assigned(IdentExpr, GenericParameterAssignments),
    Definition(Box<Spanned<LogicalTypeDef>>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum GenericParameterAssignments {
    Error(Span),
    List(Vec<(Option<Name>, Spanned<Result<GenericParamValue, Error>>)>),
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
            .separated_by(just(Token::Ctrl(',')))
            .allow_trailing()
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
            .separated_by(just(Token::Ctrl(',')))
            .allow_trailing()
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

        let ident_typ = ident_expr()
            .then(
                generic_parameter_assignments()
                    .delimited_by(just(Token::Ctrl('<')), just(Token::Ctrl('>')))
                    .map(|x| GenericParameterAssignments::List(x))
                    .recover_with(nested_delimiters(
                        Token::Ctrl('<'),
                        Token::Ctrl('>'),
                        [],
                        |span| GenericParameterAssignments::Error(span),
                    ))
                    .or_not(),
            )
            .map(|(i, a)| match a {
                Some(a) => TypeExpr::Assigned(i, a),
                None => TypeExpr::Identifier(i),
            });

        logical_type_def
            .or(ident_typ)
            .map_with_span(|t, span| (t, span))
            .recover_with(nested_delimiters(
                Token::Ctrl('('),
                Token::Ctrl(')'),
                [],
                |span| (TypeExpr::Error, span),
            ))
    })
}
