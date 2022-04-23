use tydi_common::{
    error::{Error, Result},
    traits::Identify,
};
use tydi_intern::Id;

use crate::{
    architecture::arch_storage::Arch,
    declaration::{ObjectDeclaration, ObjectKind},
};

use super::{Assign, AssignDeclaration, Assignment, ObjectSelection};

impl Assign for Id<ObjectDeclaration> {
    fn assign(
        &self,
        db: &dyn Arch,
        assignment: &(impl Into<Assignment> + Clone),
    ) -> Result<AssignDeclaration> {
        let true_assignment = assignment.clone().into();
        let self_obj = db.lookup_intern_object_declaration(*self);
        match self_obj.kind() {
            ObjectKind::ComponentPort(_) if !true_assignment.to_field().is_empty() => {
                // TODO: Validating whether all fields are assigned, and assigned in order,
                // is too complex to implement without a good use-case.
                Err(Error::BackEndError(format!(
                    "Back-end cannot assign a field of a component port ({}) directly, please use an indermediary signal.",
                    self_obj.identifier()
                )))
            }
            _ => match db.can_assign(self_obj.object_key().clone(), true_assignment.clone()) {
                Ok(_) => Ok(AssignDeclaration::new(*self, true_assignment)),
                Err(err) => Err(Error::InvalidArgument(format!(
                    "Cannot assign {}: {}",
                    self_obj.identifier(),
                    err
                ))),
            },
        }
    }
}

impl Assign for ObjectSelection {
    fn assign(
        &self,
        db: &dyn Arch,
        assignment: &(impl Into<Assignment> + Clone),
    ) -> Result<AssignDeclaration> {
        let true_assignment: Assignment = assignment.clone().into().to_nested(self.from_field());
        self.object().assign(db, &true_assignment)
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
