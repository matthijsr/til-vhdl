use std::collections::{HashMap, HashSet};

use til_query::ir::physical_properties::InterfaceDirection;
use tydi_common::name::{Name, PathName};

use crate::{
    eval::{eval_ident, get_base_def},
    expr::Expr,
    ident_expr::IdentExpr,
    struct_parse::StructStat,
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
    connections: &mut HashMap<PortSelOk, PortSelOk>,
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
        StructStat::Documentation((doc, doc_span), sub_stat) => Ok(StructStatOk::Documentation(
            doc.clone(),
            Box::new(eval_struct_stat(
                sub_stat,
                interface,
                connections,
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
        StructStat::Connection((left_sel, left_span), (right_sel, right_span)) => todo!(),
    }
}
