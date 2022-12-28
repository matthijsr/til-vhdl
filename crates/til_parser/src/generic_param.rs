use chumsky::prelude::*;
use til_query::ir::generics::{
    behavioral::integer::IntegerGeneric,
    interface::InterfaceGenericKind,
    param_value::{combination::MathOperator, GenericParamValue},
    GenericKind, GenericParameter,
};
use tydi_common::{error::Error, name::Name};

use crate::{
    lex::{Operator, Token},
    Spanned,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum GenericParameterValueExpr {
    Error,
    Integer(i32),
    Ref(Name),
    Combination(
        Box<Spanned<GenericParameterValueExpr>>,
        MathOperator,
        Box<Spanned<GenericParameterValueExpr>>,
    ),
    Parentheses(Box<Spanned<GenericParameterValueExpr>>),
    Negative(Box<Spanned<GenericParameterValueExpr>>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum GenericParameterList {
    None,
    Error,
    List(Vec<Spanned<Result<GenericParameter, Error>>>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum GenericParameterAssignments {
    Error,
    List(Vec<(Option<Name>, Spanned<GenericParameterValueExpr>)>),
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
        .at_least(1)
        .labelled("generic parameters")
}

pub fn generic_parameter_assignment(
) -> impl Parser<Token, Spanned<GenericParameterValueExpr>, Error = Simple<Token>> + Clone {
    recursive(|param_assignment| {
        let integer_value =
            param_integer().map_with_span(|x, span| (GenericParameterValueExpr::Integer(x), span));

        let negative = just(Token::Op(Operator::Sub))
            .ignore_then(param_assignment.clone())
            .map_with_span(|x, span| (GenericParameterValueExpr::Negative(Box::new(x)), span));

        let ref_n = param_name().map_with_span(|n, span| (GenericParameterValueExpr::Ref(n), span));

        let atom = integer_value
            .or(negative)
            .or(ref_n)
            .or(param_assignment
                .clone()
                .delimited_by(just(Token::Ctrl('(')), just(Token::Ctrl(')')))
                .map_with_span(|x, span| {
                    (GenericParameterValueExpr::Parentheses(Box::new(x)), span)
                }))
            .recover_with(nested_delimiters(
                Token::Ctrl('('),
                Token::Ctrl(')'),
                [],
                |span| (GenericParameterValueExpr::Error, span),
            ));

        // Multiplication, division and modulo (remainder) have the same precedence
        let op = just(Token::Op(Operator::Mul))
            .to(MathOperator::Multiply)
            .or(just(Token::Op(Operator::Div)).to(MathOperator::Divide))
            .or(just(Token::Op(Operator::Mod)).to(MathOperator::Modulo));

        let product = atom
            .clone()
            .then(op.then(param_assignment).repeated())
            .foldl(parse_math_combination);

        // Sum and subtraction have the same precedence (but a lower precedence than products)
        let op = just(Token::Op(Operator::Add))
            .to(MathOperator::Add)
            .or(just(Token::Op(Operator::Sub)).to(MathOperator::Subtract));

        let sum = product
            .clone()
            .then(op.then(product).repeated())
            .foldl(parse_math_combination);

        sum
    })
}

fn parse_math_combination(
    l: Spanned<GenericParameterValueExpr>,
    (op, r): (MathOperator, Spanned<GenericParameterValueExpr>),
) -> Spanned<GenericParameterValueExpr> {
    let span = l.1.start..r.1.end;
    (
        GenericParameterValueExpr::Combination(Box::new(l), op, Box::new(r)),
        span,
    )
}

pub fn generic_parameter_assignments(
) -> impl Parser<Token, Vec<(Option<Name>, Spanned<GenericParameterValueExpr>)>, Error = Simple<Token>>
       + Clone {
    let assignment = param_name()
        .then_ignore(just(Token::Op(Operator::Eq)))
        .or_not()
        .then(generic_parameter_assignment());

    assignment
        .separated_by(just(Token::Ctrl(',')))
        .allow_trailing()
        .labelled("generic parameter assignments")
}
