use std::convert::TryInto;

use tydi_common::error::{Error, Result};

use crate::declaration::ObjectDeclaration;

use super::{Assign, AssignDeclaration, Assignment};

impl Assign for ObjectDeclaration {
    fn assign(&self, assignment: &(impl Into<Assignment> + Clone)) -> Result<AssignDeclaration> {
        let true_assignment = assignment.clone().into();
        self.typ().can_assign(&true_assignment)?;
        Ok(AssignDeclaration::new(self.clone(), true_assignment))
    }
}

impl<T> Assign for T
where
    T: TryInto<ObjectDeclaration, Error = Error> + Clone,
{
    fn assign(&self, assignment: &(impl Into<Assignment> + Clone)) -> Result<AssignDeclaration> {
        let decl = self.clone().try_into()?;
        decl.assign(assignment)
    }
}
