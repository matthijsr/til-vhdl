use textwrap::indent;
use tydi_common::error::{Error, Result};

use crate::{architecture::arch_storage::Arch, declaration::DeclareWithIndent};

use super::{label::Label, mapping::{Mapping, MapAssignment}, Statement};

impl DeclareWithIndent for Mapping {
    fn declare_with_indent(&self, db: &dyn Arch, indent_style: &str) -> Result<String> {
        let mut result = String::new();
        result.push_str(&format!("{} port map(\n", self.component_name()));
        let mut port_maps = vec![];
        for (port, assignment) in self.port_mappings() {
            if let MapAssignment::Assigned(expr) = assignment {
                port_maps.push(expr.declare_with_indent(db, indent_style)?);
            } else {
                return Err(Error::BackEndError(format!(
                    "Error while declaring port mapping, port {} is not assigned",
                    port
                )));
            }
        }
        result.push_str(&indent(&port_maps.join(",\n"), indent_style));
        result.push_str("\n)");
        Ok(result)
    }
}

impl DeclareWithIndent for Statement {
    fn declare_with_indent(&self, db: &dyn Arch, indent_style: &str) -> Result<String> {
        let result = match self {
            Statement::Assignment(assignment) => assignment.declare_with_indent(db, indent_style),
            Statement::Mapping(portmapping) => portmapping.declare_with_indent(db, indent_style),
            Statement::Process(process) => process.declare_with_indent(db, indent_style),
        };
        if let Some(label) = self.label() {
            Ok(format!("{}: {}", label, result?))
        } else {
            result
        }
    }
}

#[cfg(test)]
mod tests {
    // TODO

    //     use super::*;
    //     use crate::{
    //         generator::common::test::{
    //             records::{rec_rev, rec_rev_nested},
    //             test_comp,
    //         },
    //         stdlib::common::architecture::{
    //             assignment::AssignmentKind, declaration::ObjectDeclaration, object::ObjectType,
    //         },
    //     };

    //     #[test]
    //     fn test_simple_portmapping_declare() -> Result<()> {
    //         let (a_dn, a_up) = ObjectType::try_from_splittable(rec_rev("a"))?;
    //         let a_dn_rec = ObjectDeclaration::signal("a_dn_rec", a_dn.unwrap(), None);
    //         let a_up_rec = ObjectDeclaration::signal("a_up_rec", a_up.unwrap(), None);
    //         let (b_dn, b_up) = ObjectType::try_from_splittable(rec_rev_nested("b"))?;
    //         let b_dn_rec = ObjectDeclaration::signal("b_dn_rec", b_dn.unwrap(), None);
    //         let b_up_rec = ObjectDeclaration::signal("b_up_rec", b_up.unwrap(), None);
    //         let mut pm = PortMapping::from_component(&test_comp(), "some_label")?;
    //         let mapped = pm
    //             .map_port("a_dn", &a_dn_rec)?
    //             .map_port("a_up", &a_up_rec)?
    //             .map_port("b_dn", &b_dn_rec)?
    //             .map_port("b_up", &b_up_rec)?;
    //         assert_eq!(
    //             r#"  some_label: test_comp port map(
    //     a_dn => a_dn_rec,
    //     a_up => a_up_rec,
    //     b_dn => b_dn_rec,
    //     b_up => b_up_rec
    //   );
    // "#,
    //             mapped.declare("  ", ";\n")?
    //         );
    //         Ok(())
    //     }

    //     #[test]
    //     fn test_complex_portmapping_declare() -> Result<()> {
    //         let (a_dn, a_up) = ObjectType::try_from_splittable(rec_rev("a_other"))?;
    //         let a_dn_rec = ObjectDeclaration::signal("a_other_dn_rec", a_dn.unwrap(), None);
    //         let a_up_rec = ObjectDeclaration::signal("a_other_up_rec", a_up.unwrap(), None);
    //         let (b_dn, b_up) = ObjectType::try_from_splittable(rec_rev_nested("b_other"))?;
    //         let b_dn_rec = ObjectDeclaration::signal("b_other_dn_rec", b_dn.unwrap(), None);
    //         let b_up_rec = ObjectDeclaration::signal("b_other_up_rec", b_up.unwrap(), None);
    //         let mut pm = PortMapping::from_component(&test_comp(), "some_label")?;
    //         let mapped = pm
    //             .map_port("a_dn", &AssignmentKind::to_direct(&a_dn_rec, true)?)?
    //             .map_port("a_up", &AssignmentKind::to_direct(&a_up_rec, true)?)?
    //             .map_port("b_dn", &AssignmentKind::to_direct(&b_dn_rec, true)?)?
    //             .map_port("b_up", &AssignmentKind::to_direct(&b_up_rec, true)?)?;
    //         assert_eq!(
    //             r#"some_label: test_comp port map(
    //   a_dn => (
    //     c => a_other_dn_rec.c
    //   ),
    //   a_up => (
    //     d => a_other_up_rec.d
    //   ),
    //   b_dn => (
    //     a => (
    //       c => b_other_dn_rec.a.c,
    //       d => b_other_dn_rec.a.d
    //     ),
    //     b => (
    //       c => b_other_dn_rec.b.c
    //     )
    //   ),
    //   b_up => (
    //     b => (
    //       d => b_other_up_rec.b.d
    //     )
    //   )
    // );
    // "#,
    //             mapped.declare("", ";\n")?
    //         );
    //         Ok(())
    //     }
}
