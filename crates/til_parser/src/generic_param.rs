use core::fmt;

use chumsky::prelude::*;
use til_query::ir::generics::{
    behavioral::{integer::IntegerGeneric, BehavioralGenericKind},
    condition::{
        integer_condition::IntegerCondition, AppliesCondition, GenericCondition, TestValue,
    },
    interface::InterfaceGenericKind,
    param_value::{combination::MathOperator, GenericParamValue},
    GenericKind, GenericParameter,
};
use tydi_common::{error::Error, name::Name};

use crate::{
    lex::{ConditionKeyword, Operator, StreamPropertyKeyword, Token},
    Span, Spanned,
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

impl fmt::Display for GenericParameterValueExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GenericParameterValueExpr::Error => write!(f, "GenericParameterValueExpr::Error"),
            GenericParameterValueExpr::Integer(i) => write!(f, "{}", i),
            GenericParameterValueExpr::Ref(r) => write!(f, "{}", r),
            GenericParameterValueExpr::Combination(l_box, op, r_box) => {
                write!(f, "{} {} {}", &l_box.0, op, &r_box.0)
            }
            GenericParameterValueExpr::Parentheses(p) => write!(f, "({})", &p.0),
            GenericParameterValueExpr::Negative(n) => write!(f, "-{}", &n.0),
        }
    }
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum GenericConditionExpr<T: TestValue> {
    Error(Span),
    Condition(GenericCondition<T>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum GenericConditionCombiningKeyword {
    And,
    Or,
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
            _ => Err(Simple::custom(
                span,
                format!("{} is not a valid parameter type.", ident),
            )),
        },
        Token::StreamProperty(StreamPropertyKeyword::Dimensionality) => {
            Ok(GenericKind::from(InterfaceGenericKind::dimensionality()))
        }
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
    // NOTE: It would make more sense to combine the default assignment and condition into the "kind" expression
    // As this would let us choose the right kind of (expected) value and condition.
    // However, at this time we only support integers, so there's not need to get fancy.
    param_name()
        .then_ignore(just(Token::Ctrl(':')))
        .then(param_kind())
        .then(
            just(Token::Op(Operator::Eq))
                .ignore_then(param_integer().map(|i| GenericParamValue::from(i))),
        )
        .then(
            just(Token::Ctrl(';'))
                .ignore_then(generic_param_integer_condition())
                .or_not(),
        )
        .map_with_span(
            |(((name, kind), default_value), opt_condition), span| match opt_condition {
                Some(cond_expr) => match cond_expr {
                    GenericConditionExpr::Error(s) => (
                        Err(Error::ParsingError(
                            "Something went wrong parsing the condition".to_string(),
                        )),
                        s,
                    ),
                    GenericConditionExpr::Condition(c) => {
                        let kind_res = match kind {
                            GenericKind::Behavioral(b) => match b {
                                BehavioralGenericKind::Integer(i) => {
                                    i.with_condition(c).map(|x| GenericKind::from(x))
                                }
                            },
                            GenericKind::Interface(i) => match i {
                                InterfaceGenericKind::Dimensionality(d) => {
                                    d.with_condition(c).map(|x| GenericKind::from(x))
                                }
                            },
                        };
                        match kind_res {
                            Ok(kind) => {
                                (GenericParameter::try_new(name, kind, default_value), span)
                            }
                            Err(e) => (Err(e), span),
                        }
                    }
                },
                None => (GenericParameter::try_new(name, kind, default_value), span),
            },
        )
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
            .then(op.then(atom).repeated())
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
        .at_least(1)
        .labelled("generic parameter assignments")
}

pub fn generic_param_integer_condition(
) -> impl Parser<Token, GenericConditionExpr<IntegerCondition>, Error = Simple<Token>> + Clone {
    recursive(|condition| {
        let gt = just(Token::Ctrl('>'))
            .ignore_then(param_integer())
            .map(|x| GenericConditionExpr::Condition(IntegerCondition::Gt(x).into()));
        let lt = just(Token::Ctrl('<'))
            .ignore_then(param_integer())
            .map(|x| GenericConditionExpr::Condition(IntegerCondition::Lt(x).into()));
        let gteq = just(Token::Op(Operator::GtEq))
            .ignore_then(param_integer())
            .map(|x| GenericConditionExpr::Condition(IntegerCondition::GtEq(x).into()));
        let lteq = just(Token::Op(Operator::LtEq))
            .ignore_then(param_integer())
            .map(|x| GenericConditionExpr::Condition(IntegerCondition::LtEq(x).into()));
        let eq = just(Token::Op(Operator::Eq))
            .ignore_then(param_integer())
            .map(|x| GenericConditionExpr::Condition(IntegerCondition::Eq(x).into()));
        let one_of = just(Token::Condition(ConditionKeyword::OneOf)).ignore_then(
            param_integer()
                .separated_by(just(Token::Ctrl(',')))
                .allow_trailing()
                .at_least(1)
                .delimited_by(just(Token::Ctrl('(')), just(Token::Ctrl(')')))
                .map(|x| GenericConditionExpr::Condition(IntegerCondition::IsIn(x).into()))
                .recover_with(nested_delimiters(
                    Token::Ctrl('('),
                    Token::Ctrl(')'),
                    [(Token::Ctrl('<'), Token::Ctrl('>'))],
                    |span: Span| GenericConditionExpr::Error(span),
                )),
        );

        let atom = gteq
            .or(lteq)
            .or(gt)
            .or(lt)
            .or(eq)
            .or(one_of)
            .or(condition
                .clone()
                .delimited_by(just(Token::Ctrl('(')), just(Token::Ctrl(')')))
                .map(|x| match x {
                    GenericConditionExpr::Error(e) => GenericConditionExpr::Error(e),
                    GenericConditionExpr::Condition(c) => {
                        GenericConditionExpr::Condition(GenericCondition::Parentheses(Box::new(c)))
                    }
                }))
            .recover_with(nested_delimiters(
                Token::Ctrl('('),
                Token::Ctrl(')'),
                [(Token::Ctrl('<'), Token::Ctrl('>'))],
                |span: Span| GenericConditionExpr::Error(span),
            ));

        // If people want to do multiple nots for whatever reason, at least parenthesize them...
        let not = just(Token::Condition(ConditionKeyword::Not)).ignore_then(atom.clone());
        let atom = atom.or(not);

        // And and Or have the same precedence
        let op = just(Token::Condition(ConditionKeyword::And))
            .to(GenericConditionCombiningKeyword::And)
            .or(just(Token::Condition(ConditionKeyword::Or))
                .to(GenericConditionCombiningKeyword::Or));

        let combination = atom
            .clone()
            .then(op.then(atom).repeated())
            .foldl(|l, (op, r)| match (l, r) {
                (GenericConditionExpr::Error(ls), GenericConditionExpr::Error(rs)) => {
                    GenericConditionExpr::Error(ls.start..rs.end)
                }
                (GenericConditionExpr::Error(s), GenericConditionExpr::Condition(_)) => {
                    GenericConditionExpr::Error(s)
                }
                (GenericConditionExpr::Condition(_), GenericConditionExpr::Error(s)) => {
                    GenericConditionExpr::Error(s)
                }
                (GenericConditionExpr::Condition(l), GenericConditionExpr::Condition(r)) => {
                    match op {
                        GenericConditionCombiningKeyword::And => GenericConditionExpr::Condition(
                            GenericCondition::And(Box::new(l), Box::new(r)),
                        ),
                        GenericConditionCombiningKeyword::Or => GenericConditionExpr::Condition(
                            GenericCondition::Or(Box::new(l), Box::new(r)),
                        ),
                    }
                }
            });

        combination
    })
}
