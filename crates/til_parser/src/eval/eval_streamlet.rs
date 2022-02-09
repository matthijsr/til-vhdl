use std::collections::HashMap;

use til_query::{
    common::logical::logicaltype::LogicalType,
    ir::{
        implementation::Implementation, project::interface_collection::InterfaceCollection,
        streamlet::Streamlet, traits::InternSelf, Ir,
    },
};
use tydi_common::name::{Name, PathName};
use tydi_intern::Id;

use crate::{
    eval::eval_implementation::eval_implementation_expr,
    expr::{Expr, StreamletProperty},
    Spanned,
};

use super::{eval_ident, eval_interface::eval_interface_expr, EvalError};

pub fn eval_streamlet_expr(
    db: &dyn Ir,
    expr: &Spanned<Expr>,
    streamlets: &HashMap<Name, Id<Streamlet>>,
    streamlet_imports: &HashMap<PathName, Id<Streamlet>>,
    implementations: &HashMap<Name, Id<Implementation>>,
    implementation_imports: &HashMap<PathName, Id<Implementation>>,
    interfaces: &HashMap<Name, Id<InterfaceCollection>>,
    interface_imports: &HashMap<PathName, Id<InterfaceCollection>>,
    types: &HashMap<Name, Id<LogicalType>>,
    type_imports: &HashMap<PathName, Id<LogicalType>>,
) -> Result<Id<Streamlet>, EvalError> {
    match &expr.0 {
        Expr::Ident(ident) => match eval_ident(ident, &expr.1, streamlets, streamlet_imports) {
            Ok(val) => Ok(val),
            Err(_) => match eval_ident(ident, &expr.1, interfaces, interface_imports) {
                Ok(interface) => {
                    let streamlet: Streamlet = interface.into();
                    Ok(streamlet.intern(db))
                }
                Err(err) => Err(EvalError {
                    span: err.span,
                    msg: "No interface or streamlet with this identity".to_string(),
                }),
            },
        },
        Expr::StreamletDef(interface, properties) => {
            let interface = eval_interface_expr(
                db,
                interface.as_ref(),
                interfaces,
                interface_imports,
                types,
                type_imports,
            )?;
            match &properties.0 {
                Expr::StreamletProps(props) => {
                    let mut implementation = None;
                    for ((_, prop_span), prop) in props.iter() {
                        match prop {
                            StreamletProperty::Implementation(impl_expr) => {
                                if implementation == None {
                                    implementation = Some(eval_implementation_expr(
                                        db,
                                        impl_expr,
                                        Some(interface),
                                        implementations,
                                        implementation_imports,
                                        interfaces,
                                        interface_imports,
                                        types,
                                        type_imports,
                                    )?)
                                } else {
                                    return Err(EvalError {
                                        span: prop_span.clone(),
                                        msg: format!("Duplicate property implementation property"),
                                    });
                                }
                            }
                        }
                    }
                    let mut streamlet = Streamlet::from(interface);
                    if let Some(implementation) = implementation {
                        streamlet = streamlet.with_implementation(Some(implementation));
                    }
                    Ok(streamlet.intern(db))
                }
                _ => Err(EvalError {
                    span: properties.1.clone(),
                    msg: "Invalid expression, expected streamlet properties".to_string(),
                }),
            }
        }
        Expr::InterfaceDef(_) => Ok(Streamlet::from(eval_interface_expr(
            db,
            expr,
            interfaces,
            interface_imports,
            types,
            type_imports,
        )?)
        .intern(db)),
        _ => Err(EvalError {
            span: expr.1.clone(),
            msg: format!("Invalid expression {:#?} for streamlet definition", &expr.0),
        }),
    }
}
