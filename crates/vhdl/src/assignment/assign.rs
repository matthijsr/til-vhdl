use tydi_common::{
    error::{Error, Result},
    traits::Identify,
};
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
            ObjectKind::EntityPort(_) | ObjectKind::ComponentPort(_)
                if !true_assignment.to_field().is_empty() =>
            {
                Err(Error::BackEndError(format!(
                    "Back-end cannot assign a field of a port ({}) directly, please use an indermediary signal.",
                    self_obj.identifier()
                )))
            }
            _ => {
                db.can_assign(self_obj.object_key().clone(), true_assignment.clone())?;
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
