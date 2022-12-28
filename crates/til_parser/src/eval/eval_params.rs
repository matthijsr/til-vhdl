use til_query::ir::generics::{
    param_value::{
        combination::{GenericParamValueOps, MathCombination},
        GenericParamValue,
    },
    GenericParameter,
};
use tydi_common::{map::InsertionOrderedMap, name::Name};

use crate::{
    generic_param::{GenericParameterAssignments, GenericParameterList, GenericParameterValueExpr},
    Spanned,
};

use super::EvalError;

pub fn eval_generic_params(
    expr: &Spanned<GenericParameterList>,
) -> Result<Vec<GenericParameter>, EvalError> {
    match &expr.0 {
        GenericParameterList::None => Ok(vec![]),
        GenericParameterList::Error => Err(EvalError {
            span: expr.1.clone(),
            msg: "There was an issue with the parameter list".to_string(),
        }),
        GenericParameterList::List(params) => params
            .iter()
            .map(|(param, span)| match param {
                Ok(param) => Ok(param.clone()),
                Err(err) => Err(EvalError {
                    span: span.clone(),
                    msg: format!("There was an issue with a parameter: {}", err),
                }),
            })
            .collect::<Result<Vec<_>, EvalError>>(),
    }
}

pub fn eval_generic_param_assignment(
    expr: &Spanned<GenericParameterValueExpr>,
    parent_params: &InsertionOrderedMap<Name, GenericParameter>,
) -> Result<GenericParamValue, EvalError> {
    let err_map = |e| EvalError {
        span: expr.1.clone(),
        msg: format!("Cannot perform this operation: {}", e),
    };
    match &expr.0 {
        GenericParameterValueExpr::Error => Err(EvalError {
            span: expr.1.clone(),
            msg: "There was an issue parsing a generic parameter value".to_string(),
        }),
        GenericParameterValueExpr::Integer(i) => Ok(GenericParamValue::Integer(*i)),
        GenericParameterValueExpr::Ref(r) => {
            if let Some(p) = parent_params.get(r) {
                Ok(GenericParamValue::from(p))
            } else {
                Err(EvalError {
                    span: expr.1.clone(),
                    msg: format!("No parameter {} exists on the parent", r),
                })
            }
        }
        GenericParameterValueExpr::Combination(l, op, r) => MathCombination::Combination(
            Box::new(eval_generic_param_assignment(l, parent_params)?),
            *op,
            Box::new(eval_generic_param_assignment(r, parent_params)?),
        )
        .verify_integer()
        .map(|x| GenericParamValue::from(x))
        .map_err(err_map),
        GenericParameterValueExpr::Parentheses(p) => {
            eval_generic_param_assignment(p, parent_params)?
                .try_add_parens()
                .map_err(err_map)
        }
        GenericParameterValueExpr::Negative(n) => eval_generic_param_assignment(n, parent_params)?
            .g_negative()
            .map(|x| GenericParamValue::from(x))
            .map_err(err_map),
    }
}

pub fn eval_generic_param_assignments_list(
    list: &Vec<(Option<Name>, Spanned<GenericParameterValueExpr>)>,
    parent_params: &InsertionOrderedMap<Name, GenericParameter>,
) -> Result<Vec<(Option<Name>, GenericParamValue)>, EvalError> {
    list.iter()
        .map(|(opt_name, res_val)| {
            Ok((
                opt_name.clone(),
                eval_generic_param_assignment(res_val, parent_params)?,
            ))
        })
        .collect::<Result<Vec<_>, EvalError>>()
}

pub fn eval_generic_param_assignments(
    expr: &Spanned<GenericParameterAssignments>,
    parent_params: &InsertionOrderedMap<Name, GenericParameter>,
) -> Result<Vec<(Option<Name>, GenericParamValue)>, EvalError> {
    match &expr.0 {
        GenericParameterAssignments::Error => Err(EvalError {
            span: expr.1.clone(),
            msg: "There's an issue with the parameter assignments".to_string(),
        }),
        GenericParameterAssignments::List(assignments) => {
            eval_generic_param_assignments_list(assignments, parent_params)
        }
    }
}
