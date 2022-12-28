use chumsky::prelude::*;
use til_query::ir::generics::{
    behavioral::integer::IntegerGeneric,
    interface::InterfaceGenericKind,
    param_value::{
        combination::{GenericParamValueOps, MathOperator},
        GenericParamValue,
    },
    GenericKind, GenericParameter,
};
use tydi_common::{error::Error, name::Name};

use crate::{
    lex::{Operator, Token},
    Span, Spanned,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum GenericParameterList {
    None,
    Error(Span),
    List(Vec<Spanned<Result<GenericParameter, Error>>>),
}

pub fn param_name() -> impl Parser<Token, Name, Error = Simple<Token>> + Clone {
    filter_map(|span, tok| match tok {
        Token::Identifier(ident) => Name::try_new(&ident).map_err(|e| {
            Simple::custom(span, format!("{} is not a valid name. Error: {}", ident, e))
        }),
        _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
    })
    .labelled("valid name")
}

pub fn param_kind() -> impl Parser<Token, GenericKind, Error = Simple<Token>> + Clone {
    filter_map(|span, tok| match tok {
        Token::Identifier(ident) => match ident.as_str() {
            "integer" => Ok(GenericKind::from(IntegerGeneric::integer())),
            "natural" => Ok(GenericKind::from(IntegerGeneric::natural())),
            "positive" => Ok(GenericKind::from(IntegerGeneric::positive())),
            "dimensionality" => Ok(GenericKind::from(InterfaceGenericKind::dimensionality())),
            _ => Err(Simple::custom(
                span,
                format!("{} is not a valid parameter type.", ident),
            )),
        },
        _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
    })
}

pub fn param_integer() -> impl Parser<Token, i32, Error = Simple<Token>> + Clone {
    let integer_labelled = filter_map(|span, tok| match tok {
        Token::Num(num) => match num.parse::<i32>() {
            Ok(i) => Ok(i),
            Err(e) => Err(Simple::custom(
                span,
                format!("{} is not a valid integer. Error: {}", num, e),
            )),
        },
        _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
    })
    .labelled("integer");

    let negative_integer = just(Token::Op(Operator::Sub))
        .ignore_then(integer_labelled.clone())
        .map(|x| -x)
        .labelled("negative integer");

    negative_integer.or(integer_labelled)
}

pub fn generic_param_expr(
) -> impl Parser<Token, Spanned<Result<GenericParameter, Error>>, Error = Simple<Token>> + Clone {
    param_name()
        .then_ignore(just(Token::Ctrl(':')))
        .then(param_kind())
        .then(
            just(Token::Op(Operator::Eq))
                .ignore_then(param_integer().map(|i| GenericParamValue::from(i))),
        )
        .map_with_span(|((name, kind), default_value), span| {
            (GenericParameter::try_new(name, kind, default_value), span)
        })
}

pub fn generic_parameters(
) -> impl Parser<Token, Vec<Spanned<Result<GenericParameter, Error>>>, Error = Simple<Token>> + Clone
{
    generic_param_expr()
        .separated_by(just(Token::Ctrl(',')))
        .allow_trailing()
        .labelled("generic parameters")
}

pub fn generic_parameter_assignment(
) -> impl Parser<Token, Spanned<Result<GenericParamValue, Error>>, Error = Simple<Token>> + Clone {
    recursive(|param_assignment| {
        let integer_value = param_integer().map(|x| Ok(GenericParamValue::from(x)));

        let negative = just(Token::Op(Operator::Sub))
            .ignore_then(param_assignment.clone())
            .map(
                |(x, _): Spanned<Result<GenericParamValue, Error>>| match x {
                    Ok(x) => x.g_negative().map(|x| GenericParamValue::from(x)),
                    Err(e) => Err(e),
                },
            );

        let single_value = integer_value.clone().or(negative.clone());

        // TODO: Ref parameter values

        let math_op = just(Token::Op(Operator::Add))
            .to(MathOperator::Add)
            .or(just(Token::Op(Operator::Sub)).to(MathOperator::Subtract))
            .or(just(Token::Op(Operator::Mul)).to(MathOperator::Multiply))
            .or(just(Token::Op(Operator::Div)).to(MathOperator::Divide))
            .or(just(Token::Op(Operator::Mod)).to(MathOperator::Modulo));

        let math_combination = single_value
            .clone()
            .then(math_op.clone())
            .then(single_value.clone())
            .map(|((l, o), r)| {
                match o {
                    MathOperator::Add => l?.g_add(r?),
                    MathOperator::Subtract => l?.g_sub(r?),
                    MathOperator::Multiply => l?.g_mul(r?),
                    MathOperator::Divide => l?.g_div(r?),
                    MathOperator::Modulo => l?.g_mod(r?),
                }
                .map(|x| GenericParamValue::from(x))
            });

        let parens = math_combination
            .clone()
            .delimited_by(just(Token::Ctrl('(')), just(Token::Ctrl(')')))
            .recover_with(nested_delimiters(
                Token::Ctrl('('),
                Token::Ctrl(')'),
                [],
                // TODO: Would be nice to include the span in the error somehow
                |_| {
                    Err(Error::ParsingError(
                        "Nesting delimiter fallback".to_string(),
                    ))
                },
            ));

        let single_value = parens.clone().or(single_value);

        // TODO: This isn't actually correct, it should be (left-associative) recursive
        // But doing it that way resulted in a stack overflow.
        // I suspect there's probably a way to tell chumsky how to do it correctly.
        // Specifically, I think the issue is that when you get something like "1 + 1 + 1", Chumsky doesn't know whether to interpret it as:
        // "(1 + 1) + 1" or "1 + (1 + 1)"
        // The correct usage may involve using foldl, like shown here: https://github.com/zesterer/chumsky/blob/6107b2f98a22e8d22a6ee64b0ab4f727166d6769/examples/nano_rust.rs
        let math_combination = single_value
            .clone()
            .then(math_op)
            .then(single_value.clone())
            .map(|((l, o), r)| {
                match o {
                    MathOperator::Add => l?.g_add(r?),
                    MathOperator::Subtract => l?.g_sub(r?),
                    MathOperator::Multiply => l?.g_mul(r?),
                    MathOperator::Divide => l?.g_div(r?),
                    MathOperator::Modulo => l?.g_mod(r?),
                }
                .map(|x| GenericParamValue::from(x))
            });

        parens
            .or(math_combination)
            .or(negative)
            .or(integer_value)
            .map_with_span(|x, span| (x, span))
    })
}

pub fn generic_parameter_assignments() -> impl Parser<
    Token,
    Vec<(Option<Name>, Spanned<Result<GenericParamValue, Error>>)>,
    Error = Simple<Token>,
> + Clone {
    let assignment = param_name()
        .then_ignore(just(Token::Op(Operator::Eq)))
        .or_not()
        .then(generic_parameter_assignment());

    assignment
        .separated_by(just(Token::Ctrl(',')))
        .allow_trailing()
        .labelled("generic parameter assignments")
}
