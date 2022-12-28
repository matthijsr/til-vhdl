use chumsky::prelude::*;
use til_query::ir::generics::{
    behavioral::integer::IntegerGeneric,
    interface::InterfaceGenericKind,
    param_value::{
        combination::{GenericParamValueOps, MathCombination, MathOperator},
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
        let integer_value =
            param_integer().map_with_span(|x, span| (Ok(GenericParamValue::from(x)), span));

        let negative = just(Token::Op(Operator::Sub))
            .ignore_then(param_assignment.clone())
            .map_with_span(
                |(x, inner_span): Spanned<Result<GenericParamValue, Error>>, span| match x {
                    Ok(x) => (x.g_negative().map(|x| GenericParamValue::from(x)), span),
                    Err(e) => (Err(e), inner_span),
                },
            );

        // TODO: Ref parameter values

        let atom = integer_value
            .or(negative)
            .or(param_assignment
                .clone()
                .delimited_by(just(Token::Ctrl('(')), just(Token::Ctrl(')')))
                .map_with_span(
                    |(x, inner_span): Spanned<Result<GenericParamValue, Error>>, span| match x {
                        Ok(x) => (x.try_add_parens(), span),
                        Err(e) => (Err(e), inner_span),
                    },
                ))
            .recover_with(nested_delimiters(
                Token::Ctrl('('),
                Token::Ctrl(')'),
                [],
                |span| {
                    (
                        Err(Error::ParsingError(
                            "Nesting delimiter fallback".to_string(),
                        )),
                        span,
                    )
                },
            ));

        // Multiplication, division and modulo (remainder) have the same precedence
        let op = just(Token::Op(Operator::Mul))
            .to(MathOperator::Multiply)
            .or(just(Token::Op(Operator::Div)).to(MathOperator::Divide))
            .or(just(Token::Op(Operator::Mod)).to(MathOperator::Modulo));

        let product = atom
            .clone()
            .then(op.then(param_assignment).repeated())
            .foldl(|l, (op, r)| parse_math_combination(l, op, r));

        // Sum and subtraction have the same precedence (but a lower precedence than products)
        let op = just(Token::Op(Operator::Add))
            .to(MathOperator::Add)
            .or(just(Token::Op(Operator::Sub)).to(MathOperator::Subtract));

        let sum = product
            .clone()
            .then(op.then(product).repeated())
            .foldl(|l, (op, r)| parse_math_combination(l, op, r));

        sum
    })
}

fn parse_math_combination(
    l: (Result<GenericParamValue, Error>, std::ops::Range<usize>),
    op: MathOperator,
    r: (Result<GenericParamValue, Error>, std::ops::Range<usize>),
) -> (Result<GenericParamValue, Error>, std::ops::Range<usize>) {
    let span = l.1.start..r.1.end;
    match (l.0, r.0) {
        (Ok(l), Ok(r)) => (
            MathCombination::Combination(Box::new(l), op, Box::new(r))
                .verify_integer()
                .map(|x| GenericParamValue::from(x)),
            span,
        ),
        (Ok(_), Err(e)) => (Err(e), r.1),
        (Err(e), Ok(_)) => (Err(e), l.1),
        (Err(le), Err(re)) => (
            Err(Error::ParsingError(format!(
                "Both left and right values are invalid. Left: {} ; Right: {}",
                le, re
            ))),
            span,
        ),
    }
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
