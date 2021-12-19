use std::convert::TryInto;

use tydi_common::error::{Error, Result};
use tydi_intern::Id;

use crate::{architecture::arch_storage::Arch, declaration::ObjectDeclaration};

use super::{Assign, AssignDeclaration, Assignment};

impl Assign for Id<ObjectDeclaration> {
    fn assign(
        &self,
        db: &impl Arch,
        assignment: &(impl Into<Assignment> + Clone),
    ) -> Result<AssignDeclaration> {
        let true_assignment = assignment.clone().into();
        let self_obj = db.get_object_declaration(*self);
        self_obj.typ().can_assign(db, &true_assignment)?;
        Ok(AssignDeclaration::new(*self, true_assignment))
    }
}

// impl<T> Assign for T
// where
//     T: TryInto<Id<ObjectDeclaration>, Error = Error> + Clone,
// {
//     fn assign(
//         &self,
//         db: &impl Arch,
//         assignment: &(impl Into<Assignment> + Clone),
//     ) -> Result<AssignDeclaration> {
//         let decl = self.clone().try_into()?;
//         decl.assign(db, assignment)
//     }
// }
