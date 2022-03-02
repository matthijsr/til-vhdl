use tydi_common::{
    error::{Error, Result, TryResult},
    traits::Identify,
};
use tydi_intern::Id;

use crate::{
    architecture::arch_storage::Arch,
    declaration::{AliasDeclaration, ObjectDeclaration, ObjectKind},
};

use super::{Assign, AssignDeclaration, Assignment};

impl Assign for Id<ObjectDeclaration> {
    fn assign(
        &self,
        db: &dyn Arch,
        assignment: impl TryResult<Assignment>,
    ) -> Result<AssignDeclaration> {
        let true_assignment = assignment.try_result()?;
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

impl Assign for AliasDeclaration {
    fn assign(
        &self,
        db: &dyn Arch,
        assignment: impl TryResult<Assignment>,
    ) -> Result<AssignDeclaration> {
        let assignment = assignment.try_result()?.to_nested(self.field_selection());
        self.object().assign(db, assignment)
    }
}
