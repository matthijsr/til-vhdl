use tydi_common::{
    error::{Error, Result},
    traits::Identify,
};
use tydi_intern::Id;

use crate::{
    architecture::arch_storage::{Arch, AssignmentState},
    declaration::{ObjectDeclaration, ObjectKind},
};

use super::{Assign, AssignDeclaration, Assignment, ObjectSelection};

impl Assign for Id<ObjectDeclaration> {
    fn assign(
        &self,
        db: &dyn Arch,
        assignment: impl Into<Assignment>,
    ) -> Result<AssignDeclaration> {
        let true_assignment = assignment.into();
        let self_obj = db.lookup_intern_object_declaration(*self);
        if let ObjectKind::ComponentPort(_) = self_obj.kind() {
            Err(Error::BackEndError(format!(
                "Component ports like \"{}\" should not be mapped, not assigned. This error should not occur.",
                self_obj.identifier()
            )))
        } else {
            match db.can_assign(
                self_obj.object_key().clone(),
                true_assignment.clone(),
                AssignmentState::Default,
            ) {
                Ok(_) => Ok(AssignDeclaration::new(*self, true_assignment)),
                Err(err) => Err(Error::InvalidArgument(format!(
                    "Cannot assign {}: {}",
                    self_obj.identifier(),
                    err,
                ))),
            }
        }
    }
}

impl Assign for ObjectSelection {
    fn assign(
        &self,
        db: &dyn Arch,
        assignment: impl Into<Assignment>,
    ) -> Result<AssignDeclaration> {
        let true_assignment: Assignment = assignment.into().to_nested(self.from_field());
        self.object().assign(db, true_assignment)
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
