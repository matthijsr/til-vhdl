use std::collections::{HashMap, HashSet};

use til_query::ir::physical_properties::InterfaceDirection;
use tydi_common::name::{Name, PathName};

use crate::{
    eval::{eval_ident, get_base_def},
    expr::Expr,
    ident_expr::IdentExpr,
    struct_parse::{PortSel, StructStat},
    Spanned,
};

use super::{
    eval_interface::InterfaceDef,
    eval_name,
    eval_streamlet::StreamletDef,
    eval_type::{eval_type_expr, LogicalTypeDef},
    Def, EvalError,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ImplementationDef {
    interface: Def<InterfaceDef>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ImplementationKindDef {
    Structural(Vec<StructStat>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum StructStatOk {
    Documentation(String, Box<Self>),
    Instance(Name, Def<InterfaceDef>),
    Connection(PortSelOk, PortSelOk),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PortSelOk {
    Own(Name),
    Instance(Name, Name),
}

pub fn eval_struct_stat(
    stat: &Spanned<StructStat>,
    interface: InterfaceDef,
    connected_ports: &mut HashSet<PortSelOk>,
    instances: &mut HashMap<Name, InterfaceDef>,
    streamlets: &HashMap<Name, Def<StreamletDef>>,
    streamlet_imports: &HashMap<PathName, Def<StreamletDef>>,
    interfaces: &HashMap<Name, Def<InterfaceDef>>,
    interface_imports: &HashMap<PathName, Def<InterfaceDef>>,
) -> Result<StructStatOk, EvalError> {
    match &stat.0 {
        StructStat::Error => Err(EvalError {
            span: stat.1.clone(),
            msg: "Invalid structural statement (ERROR)".to_string(),
        }),
        StructStat::Documentation((doc, _doc_span), sub_stat) => Ok(StructStatOk::Documentation(
            doc.clone(),
            Box::new(eval_struct_stat(
                sub_stat,
                interface,
                connected_ports,
                instances,
                streamlets,
                streamlet_imports,
                interfaces,
                interface_imports,
            )?),
        )),
        StructStat::Instance((name_string, name_span), (interface_expr, interface_span)) => {
            let instance_name = eval_name(name_string, name_span)?;

            let interface = match interface_expr {
                IdentExpr::Name((n, s)) => {
                    let streamlet_name = eval_name(n, s)?;
                    match streamlets.get(&streamlet_name) {
                        Some(streamlet_def) => {
                            let iface =
                                get_base_def(streamlet_def, s, streamlets, &mut HashSet::new())?
                                    .interface()
                                    .clone();
                            if instances.insert(
                                instance_name.clone(),
                                get_base_def(&iface, s, interfaces, &mut HashSet::new())?,
                            ) == None
                            {
                                Ok(iface)
                            } else {
                                Err(EvalError {
                    span: name_span.clone(),
                    msg: format!("A Streamlet instance with name {} already exists within this implementation", instance_name),
                })
                            }
                        }
                        None => {
                            return Err(EvalError {
                                span: s.clone(),
                                msg: format!("No such streamlet"),
                            })
                        }
                    }
                }
                IdentExpr::PathName(_) => todo!(),
            }?;

            Ok(StructStatOk::Instance(instance_name, interface))
        }
        StructStat::Connection(left_sel, right_sel) => {
            let parse_sel = |sel: &Spanned<PortSel>| -> Result<PortSelOk, EvalError> {
                match &sel.0 {
                    PortSel::Own(own) => {
                        let own_name = eval_name(own, &sel.1)?;
                        if interface.iter().any(|(name, _, _)| name == &own_name) {
                            Ok(PortSelOk::Own(own_name))
                        } else {
                            Err(EvalError {
                                span: sel.1.clone(),
                                msg: format!("No port {} on own interface", own_name),
                            })
                        }
                    }
                    PortSel::Instance(
                        (instance_string, instance_span),
                        (port_string, port_span),
                    ) => {
                        let instance_name = eval_name(instance_string, instance_span)?;
                        let port_name = eval_name(port_string, port_span)?;
                        if let Some(iface) = instances.get(&instance_name) {
                            if iface.iter().any(|(name, _, _)| name == &port_name) {
                                Ok(PortSelOk::Instance(instance_name, port_name))
                            } else {
                                Err(EvalError {
                                    span: sel.1.clone(),
                                    msg: format!(
                                        "No port {} on instance {}",
                                        port_name, instance_name
                                    ),
                                })
                            }
                        } else {
                            Err(EvalError {
                                span: instance_span.clone(),
                                msg: format!("No such instance, {}", instance_name),
                            })
                        }
                    }
                }
            };
            let left = parse_sel(left_sel)?;
            let right = parse_sel(right_sel)?;
            if left == right {
                return Err(EvalError {
                    span: right_sel.1.clone(),
                    msg: format!("Cannot connect a port to itself"),
                });
            }
            if connected_ports.contains(&left) {
                Err(EvalError {
                    span: left_sel.1.clone(),
                    msg: "This port was already connected".to_string(),
                })
            } else if connected_ports.contains(&right) {
                Err(EvalError {
                    span: right_sel.1.clone(),
                    msg: "This port was already connected".to_string(),
                })
            } else {
                connected_ports.insert(left.clone());
                connected_ports.insert(right.clone());
                Ok(StructStatOk::Connection(left, right))
            }
        }
    }
}
