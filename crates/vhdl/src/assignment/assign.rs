use std::convert::TryInto;

use tydi_common::error::{Error, Result};
use tydi_intern::Id;

use crate::{
    architecture::arch_storage::Arch,
    declaration::{ObjectDeclaration, ObjectKind},
};

use super::{Assign, AssignDeclaration, Assignment};

impl Assign for Id<ObjectDeclaration> {
    fn assign(
        &self,
        db: &dyn Arch,
        assignment: &(impl Into<Assignment> + Clone),
    ) -> Result<AssignDeclaration> {
        let true_assignment = assignment.clone().into();
        let self_obj = db.lookup_intern_object_declaration(*self);
        // TODO: Fix this nonsense
        match self_obj.kind() {
            ObjectKind::EntityPort | ObjectKind::ComponentPort
                if !true_assignment.to_field().is_empty() =>
            {
                Err(Error::BackEndError(format!(
                    "Back-end cannot assign a field of a port ({}) directly, please use an indermediary signal.",
                    self_obj.identifier()
                )))
            }
            _ => {
                self_obj.typ().can_assign(db, &true_assignment)?;
                Ok(AssignDeclaration::new(*self, true_assignment))
            }
        }
    }
}

// impl<T> Assign for T
// where
//     T: TryInto<Id<ObjectDeclaration>, Error = Error> + Clone,
// {
//     fn assign(
//         &self,
//         db: &dyn Arch,
//         assignment: &(impl Into<Assignment> + Clone),
//     ) -> Result<AssignDeclaration> {
//         let decl = self.clone().try_into()?;
//         decl.assign(db, assignment)
//     }
// }
