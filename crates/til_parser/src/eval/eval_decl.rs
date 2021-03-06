use std::{collections::HashMap, sync::Arc};

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
use tydi_common::{
    name::{Name, PathName},
    traits::Documents,
};
use tydi_intern::Id;

use crate::{
    eval::{
        eval_implementation::eval_implementation_expr, eval_interface::eval_interface_expr,
        eval_streamlet::eval_streamlet_expr,
    },
    impl_expr::ImplDefExpr,
    namespace::Decl,
    Span,
};

use super::{eval_ident, eval_name, eval_type::eval_type_expr, EvalError};

pub fn eval_declaration(
    db: &dyn Ir,
    decl: &Decl,
    namespace: &PathName,
    streamlets: &mut HashMap<Name, Id<Arc<Streamlet>>>,
    streamlet_imports: &HashMap<PathName, Id<Arc<Streamlet>>>,
    implementations: &mut HashMap<Name, Id<Implementation>>,
    implementation_imports: &HashMap<PathName, Id<Implementation>>,
    interfaces: &mut HashMap<Name, Id<Arc<Interface>>>,
    interface_imports: &HashMap<PathName, Id<Arc<Interface>>>,
    types: &mut HashMap<Name, Id<LogicalType>>,
    type_imports: &HashMap<PathName, Id<LogicalType>>,
) -> Result<(), EvalError> {
    // As everything is exported (public) by default, shadowing declarations would be confusing
    let dup_id = |n: &String, s: &Span, kind: &str| -> EvalError {
        EvalError {
            span: s.clone(),
            msg: format!("Duplicate declaration for {} identity {}", kind, n),
        }
    };

    match decl {
        Decl::TypeDecl((n, s), expr) => {
            let name = eval_name(n, s)?;
            let type_id = eval_type_expr(db, (&expr.0, &expr.1), types, type_imports)?;
            if let Some(_) = types.insert(name, type_id) {
                Err(dup_id(n, s, "type"))
            } else {
                Ok(())
            }
        }
        Decl::ImplDecl(doc, (n, s), expr) => {
            let name = eval_name(n, s)?;
            let (impl_id, interface_id) = match &expr.0 {
                ImplDefExpr::Identity(ident) => {
                    let mut implementation = eval_ident(
                        ident,
                        &expr.1,
                        implementations,
                        implementation_imports,
                        "implementation",
                    )?;
                    if let Some(doc) = doc {
                        implementation = implementation.get(db).with_doc(&doc.0).intern(db);
                    }
                    let interface =
                        eval_ident(ident, &expr.1, interfaces, interface_imports, "interface")?;
                    (implementation, interface)
                }
                ImplDefExpr::Def(iface, body) => {
                    let interface = eval_interface_expr(
                        db,
                        iface,
                        interfaces,
                        interface_imports,
                        types,
                        type_imports,
                    )?;
                    eval_implementation_expr(
                        db,
                        body,
                        &namespace.with_child(name.clone()),
                        doc,
                        Some(interface),
                        streamlets,
                        streamlet_imports,
                        implementations,
                        implementation_imports,
                        interfaces,
                        interface_imports,
                        types,
                        type_imports,
                    )?
                }
            };

            if let Some(_) = interfaces.insert(name.clone(), interface_id) {
                Err(dup_id(n, s, "interface"))
            } else if let Some(_) = implementations.insert(name, impl_id) {
                Err(dup_id(n, s, "implementation"))
            } else {
                Ok(())
            }
        }
        Decl::InterfaceDecl((n, s), expr) => {
            let name = eval_name(n, s)?;
            let interface_id =
                eval_interface_expr(db, expr, interfaces, interface_imports, types, type_imports)?;
            if let Some(_) = interfaces.insert(name, interface_id) {
                Err(dup_id(n, s, "interface"))
            } else {
                Ok(())
            }
        }
        Decl::StreamletDecl(doc, (n, s), expr) => {
            let name = eval_name(n, s)?;
            let (streamlet_id, interface_id) = eval_streamlet_expr(
                db,
                expr,
                &namespace.with_child(name.clone()),
                doc,
                streamlets,
                streamlet_imports,
                implementations,
                implementation_imports,
                interfaces,
                interface_imports,
                types,
                type_imports,
            )?;

            if let Some(_) = interfaces.insert(name.clone(), interface_id) {
                Err(dup_id(n, s, "interface"))
            } else if let Some(_) = streamlets.insert(name, streamlet_id) {
                Err(dup_id(n, s, "streamlet"))
            } else {
                Ok(())
            }
        }
    }
}
