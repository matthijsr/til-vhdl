use std::collections::HashMap;

use til_query::{
    common::logical::logicaltype::LogicalType,
    ir::{
        implementation::Implementation,
        project::interface::Interface,
        streamlet::Streamlet,
        traits::{GetSelf, InternSelf},
        Ir,
    },
};
use tydi_common::{name::{Name, PathName}, traits::Documents};
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
    name: &PathName,
    doc: &Option<String>,
    streamlets: &HashMap<Name, Id<Streamlet>>,
    streamlet_imports: &HashMap<PathName, Id<Streamlet>>,
    implementations: &HashMap<Name, Id<Implementation>>,
    implementation_imports: &HashMap<PathName, Id<Implementation>>,
    interfaces: &HashMap<Name, Id<Interface>>,
    interface_imports: &HashMap<PathName, Id<Interface>>,
    types: &HashMap<Name, Id<LogicalType>>,
    type_imports: &HashMap<PathName, Id<LogicalType>>,
) -> Result<(Id<Streamlet>, Id<Interface>), EvalError> {
    match &expr.0 {
        Expr::Ident(ident) => {
            if let Ok(val) = eval_ident(ident, &expr.1, streamlets, streamlet_imports, "streamlet")
            {
                let interface =
                    eval_ident(ident, &expr.1, interfaces, interface_imports, "interface")?;
                let mut streamlet = val.get(db).with_name(name.clone());
                if let Some(doc) = doc {
                    streamlet.set_doc(doc);
                }
                Ok((streamlet.intern(db), interface))
            } else {
                match eval_ident(ident, &expr.1, interfaces, interface_imports, "streamlet") {
                    Ok(interface) => {
                        let mut streamlet: Streamlet = interface.into();
                        if let Some(doc) = doc {
                            streamlet.set_doc(doc);
                        }
                        Ok((streamlet.with_name(name.clone()).intern(db), interface))
                    }
                    Err(err) => Err(EvalError {
                        span: err.span,
                        msg: "No interface or streamlet with this identity".to_string(),
                    }),
                }
            }
        }
        Expr::StreamletDef(interface, properties) => {
            let interface = eval_interface_expr(
                db,
                interface,
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
                                        name,
                                        None,
                                        Some(interface),
                                        streamlets,
                                        streamlet_imports,
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
                    if let Some((implementation, _)) = implementation {
                        streamlet = streamlet.with_implementation(Some(implementation));
                    }
                    if let Some(doc) = doc {
                        streamlet.set_doc(doc);
                    }
                    Ok((streamlet.with_name(name.clone()).intern(db), interface))
                }
                _ => Err(EvalError {
                    span: properties.1.clone(),
                    msg: "Invalid expression, expected streamlet properties".to_string(),
                }),
            }
        }
        Expr::InterfaceDef(_) => {
            let interface =
                eval_interface_expr(db, expr, interfaces, interface_imports, types, type_imports)?;
            let mut streamlet = Streamlet::from(interface);
            if let Some(doc) = doc {
                streamlet.set_doc(doc);
            }
            Ok((streamlet.with_name(name.clone()).intern(db), interface))
        }
        _ => Err(EvalError {
            span: expr.1.clone(),
            msg: format!("Invalid expression {:#?} for streamlet definition", &expr.0),
        }),
    }
}
