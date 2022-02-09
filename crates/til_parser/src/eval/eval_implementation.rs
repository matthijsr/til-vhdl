use std::collections::{HashMap, HashSet};

use til_query::{
    common::logical::logicaltype::LogicalType,
    ir::{
        connection::{Connection, InterfaceReference},
        implementation::{structure::Structure, Implementation},
        physical_properties::InterfaceDirection,
        project::interface_collection::InterfaceCollection,
        streamlet::Streamlet,
        traits::InternSelf,
        Ir,
    },
};
use tydi_common::name::{Name, PathName};
use tydi_intern::Id;

use crate::{
    eval::{eval_ident, eval_interface::eval_interface_expr},
    expr::{Expr, RawImpl},
    ident_expr::IdentExpr,
    struct_parse::{PortSel, StructStat},
    Spanned,
};

use super::{eval_common_error, eval_name, eval_type::eval_type_expr, Def, EvalError};

pub fn eval_struct_stat(
    db: &dyn Ir,
    stat: &Spanned<StructStat>,
    structure: &mut Structure,
    streamlets: &HashMap<Name, Id<Streamlet>>,
    streamlet_imports: &HashMap<PathName, Id<Streamlet>>,
    implementations: &HashMap<Name, Id<Implementation>>,
    implementation_imports: &HashMap<PathName, Id<Implementation>>,
    interfaces: &HashMap<Name, Id<InterfaceCollection>>,
    interface_imports: &HashMap<PathName, Id<InterfaceCollection>>,
    types: &HashMap<Name, Id<LogicalType>>,
    type_imports: &HashMap<PathName, Id<LogicalType>>,
) -> Result<(), EvalError> {
    match &stat.0 {
        StructStat::Error => Err(EvalError {
            span: stat.1.clone(),
            msg: "Invalid structural statement (ERROR)".to_string(),
        }),
        StructStat::Documentation(_, sub_stat) => {
            // NOTE: We're not actually doing anything with documentation yet.
            eval_struct_stat(
                db,
                sub_stat,
                structure,
                streamlets,
                streamlet_imports,
                implementations,
                implementation_imports,
                interfaces,
                interface_imports,
                types,
                type_imports,
            )?;
            Ok(())
        }
        StructStat::Instance((name_string, name_span), (ident_expr, ident_span)) => {
            let name = eval_name(name_string, name_span)?;
            let streamlet = eval_ident(ident_expr, ident_span, streamlets, streamlet_imports)?;
            eval_common_error(
                structure.try_add_streamlet_instance(name, streamlet),
                name_span,
            )?;
            Ok(())
        }
        StructStat::Connection(left_sel, right_sel) => {
            let parse_sel = |sel: &Spanned<PortSel>| -> Result<InterfaceReference, EvalError> {
                match &sel.0 {
                    PortSel::Own(own) => {
                        let own_name = eval_name(own, &sel.1)?;
                        Ok(InterfaceReference::new(None, own_name))
                    }
                    PortSel::Instance(
                        (instance_string, instance_span),
                        (port_string, port_span),
                    ) => {
                        let instance_name = eval_name(instance_string, instance_span)?;
                        let port_name = eval_name(port_string, port_span)?;
                        Ok(InterfaceReference::new(Some(instance_name), port_name))
                    }
                }
            };
            eval_common_error(
                structure.try_add_connection(db, parse_sel(left_sel)?, parse_sel(right_sel)?),
                &stat.1,
            )?;
            Ok(())
        }
    }
}

pub fn eval_implementation_expr(
    db: &dyn Ir,
    expr: &Spanned<Expr>,
    interface: Option<Id<InterfaceCollection>>,
    streamlets: &HashMap<Name, Id<Streamlet>>,
    streamlet_imports: &HashMap<PathName, Id<Streamlet>>,
    implementations: &HashMap<Name, Id<Implementation>>,
    implementation_imports: &HashMap<PathName, Id<Implementation>>,
    interfaces: &HashMap<Name, Id<InterfaceCollection>>,
    interface_imports: &HashMap<PathName, Id<InterfaceCollection>>,
    types: &HashMap<Name, Id<LogicalType>>,
    type_imports: &HashMap<PathName, Id<LogicalType>>,
) -> Result<Id<Implementation>, EvalError> {
    match &expr.0 {
        Expr::Ident(ident) => eval_ident(ident, &expr.1, implementations, implementation_imports),
        Expr::RawImpl(raw_impl) => {
            if let Some(interface) = interface {
                match raw_impl {
                    RawImpl::Struct(struct_stats) => {
                        let mut structure = Structure::new(interface);
                        for stat in struct_stats.iter() {
                            eval_struct_stat(
                                db,
                                stat,
                                &mut structure,
                                streamlets,
                                streamlet_imports,
                                implementations,
                                implementation_imports,
                                interfaces,
                                interface_imports,
                                types,
                                type_imports,
                            )?;
                        }
                        Ok(Implementation::from(structure).intern(db))
                    }
                    RawImpl::Behavioural(_) => todo!(),
                }
            } else {
                Err(EvalError {
                    span: expr.1.clone(),
                    msg: "An implementation definition requires an interface".to_string(),
                })
            }
        }
        Expr::ImplDef(interface, raw_impl) => {
            let interface = eval_interface_expr(
                db,
                interface,
                interfaces,
                interface_imports,
                types,
                type_imports,
            )?;
            eval_implementation_expr(
                db,
                raw_impl,
                Some(interface),
                streamlets,
                streamlet_imports,
                implementations,
                implementation_imports,
                interfaces,
                interface_imports,
                types,
                type_imports,
            )
        }
        _ => Err(EvalError {
            span: expr.1.clone(),
            msg: format!(
                "Invalid expression {:#?} for implementation definition",
                &expr.0
            ),
        }),
    }
}
