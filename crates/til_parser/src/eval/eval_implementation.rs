use std::{collections::HashMap, path::PathBuf, sync::Arc};

use til_query::ir::{
    connection::InterfaceReference,
    implementation::{link::Link, structure::Structure, Implementation},
    project::{interface::Interface, type_declaration::TypeDeclaration},
    streamlet::Streamlet,
    traits::InternSelf,
    Ir,
};
use tydi_common::{
    name::{Name, PathName},
    traits::Documents,
};
use tydi_intern::Id;

use crate::{
    doc_expr::DocExpr,
    eval::eval_ident,
    impl_expr::ImplBodyExpr,
    struct_parse::{InterfaceParamAssignments, PortSel, StructStat},
    Spanned,
};

use super::{
    eval_common_error, eval_name, eval_params::eval_generic_param_assignments_list, EvalError,
};

pub fn eval_struct_stat(
    db: &dyn Ir,
    stat: &Spanned<StructStat>,
    structure: &mut Structure,
    streamlets: &HashMap<Name, Id<Arc<Streamlet>>>,
    streamlet_imports: &HashMap<PathName, Id<Arc<Streamlet>>>,
    implementations: &HashMap<Name, Id<Implementation>>,
    implementation_imports: &HashMap<PathName, Id<Implementation>>,
    interfaces: &HashMap<Name, Id<Arc<Interface>>>,
    interface_imports: &HashMap<PathName, Id<Arc<Interface>>>,
    types: &HashMap<Name, TypeDeclaration>,
    type_imports: &HashMap<PathName, TypeDeclaration>,
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
        StructStat::Instance(
            (name_string, name_span),
            (ident_expr, ident_span),
            domain_assignments,
        ) => {
            let name = eval_name(name_string, name_span)?;
            let streamlet = eval_ident(
                ident_expr,
                ident_span,
                streamlets,
                streamlet_imports,
                "streamlet",
            )?;
            match &domain_assignments.0 {
                InterfaceParamAssignments::Error => Err(EvalError {
                    span: stat.1.clone(),
                    msg: "Invalid domain assignments (ERROR)".to_string(),
                }),
                InterfaceParamAssignments::None => eval_common_error(
                    structure
                        .try_add_streamlet_instance_default(db, name, streamlet)
                        .map(|_| ()),
                    name_span,
                ),
                InterfaceParamAssignments::JustDomains(domains) => {
                    let name_list = eval_domains(domains)?;
                    eval_common_error(
                        structure
                            .try_add_streamlet_instance_parameters_default(
                                db, name, streamlet, name_list,
                            )
                            .map(|_| ()),
                        name_span,
                    )
                }
                InterfaceParamAssignments::JustParams(param_assignments) => {
                    let assignments = eval_generic_param_assignments_list(
                        param_assignments,
                        structure.interface(db).parameters(),
                    )?;
                    eval_common_error(
                        structure
                            .try_add_streamlet_instance_domains_default(
                                db,
                                name,
                                streamlet,
                                assignments,
                            )
                            .map(|_| ()),
                        name_span,
                    )
                }
                InterfaceParamAssignments::Assignments(domains, param_assignments) => {
                    let name_list = eval_domains(domains)?;
                    let assignments = eval_generic_param_assignments_list(
                        param_assignments,
                        structure.interface(db).parameters(),
                    )?;
                    eval_common_error(
                        structure
                            .try_add_streamlet_instance(db, name, streamlet, name_list, assignments)
                            .map(|_| ()),
                        name_span,
                    )
                }
            }
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

fn eval_domains(
    list: &Vec<(
        Option<(String, std::ops::Range<usize>)>,
        (String, std::ops::Range<usize>),
    )>,
) -> Result<Vec<(Option<Name>, Name)>, EvalError> {
    let mut name_list = vec![];
    for (left, right) in list.iter() {
        let left = if let Some(left) = left {
            Some(eval_name(&left.0, &left.1)?)
        } else {
            None
        };
        let right = eval_name(&right.0, &right.1)?;
        name_list.push((left, right));
    }
    Ok(name_list)
}

pub fn eval_implementation_expr(
    db: &dyn Ir,
    link_root: &PathBuf,
    expr: &Spanned<ImplBodyExpr>,
    name: &PathName,
    doc: &DocExpr,
    interface: Option<Id<Arc<Interface>>>,
    streamlets: &HashMap<Name, Id<Arc<Streamlet>>>,
    streamlet_imports: &HashMap<PathName, Id<Arc<Streamlet>>>,
    implementations: &HashMap<Name, Id<Implementation>>,
    implementation_imports: &HashMap<PathName, Id<Implementation>>,
    interfaces: &HashMap<Name, Id<Arc<Interface>>>,
    interface_imports: &HashMap<PathName, Id<Arc<Interface>>>,
    types: &HashMap<Name, TypeDeclaration>,
    type_imports: &HashMap<PathName, TypeDeclaration>,
) -> Result<(Id<Implementation>, Id<Arc<Interface>>), EvalError> {
    match &expr.0 {
        ImplBodyExpr::Error => Err(EvalError {
            span: expr.1.clone(),
            msg: "Error parsing implementation body".to_string(),
        }),
        ImplBodyExpr::Struct(struct_doc, struct_stats) => {
            if let Some(interface) = interface {
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
                eval_common_error(structure.validate_connections(db), &expr.1)?;
                let mut implementation = Implementation::from(structure).with_name(name.clone());
                if let Some(struct_doc) = struct_doc {
                    if doc.is_some() {
                        return Err(EvalError {
                            span: struct_doc.1.clone(),
                            msg: "Two documentation instances".to_string(),
                        });
                    } else {
                        implementation.set_doc(&struct_doc.0);
                    }
                }
                if let Some(doc) = doc {
                    implementation.set_doc(&doc.0);
                }
                Ok((implementation.intern(db), interface))
            } else {
                Err(EvalError {
                    span: expr.1.clone(),
                    msg: "An implementation definition requires an interface".to_string(),
                })
            }
        }
        ImplBodyExpr::Link(pth) => {
            let mut link_pth = link_root.clone();
            link_pth.push(pth);
            let link = eval_common_error(Link::try_new(link_pth), &expr.1)?;
            let mut implementation = Implementation::from(link).with_name(name.clone());
            if let Some(doc) = doc {
                implementation.set_doc(&doc.0);
            }
            if let Some(interface) = interface {
                Ok((implementation.intern(db), interface))
            } else {
                Err(EvalError {
                    span: expr.1.clone(),
                    msg: "An implementation definition requires an interface".to_string(),
                })
            }
        }
    }
}
