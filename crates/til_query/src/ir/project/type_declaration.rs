use tydi_common::{
    error::{Error, Result, TryOptional, TryResult},
    map::InsertionOrderedMap,
    name::{Name, NameSelf, PathName, PathNameSelf},
    traits::Identify,
};
use tydi_intern::Id;

use crate::{
    common::logical::logicaltype::LogicalType,
    ir::{
        generics::{param_value::GenericParamValue, GenericParameter},
        implementation::structure::streamlet_instance::GenericParameterAssignment,
        traits::MoveDb,
        Ir,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeDeclaration {
    name: PathName,
    typ: Id<LogicalType>,
    parameter_assignments: Option<InsertionOrderedMap<Name, GenericParameterAssignment>>,
}

impl TypeDeclaration {
    pub fn try_new_no_params(
        db: &dyn Ir,
        name: impl TryResult<PathName>,
        typ: Id<LogicalType>,
    ) -> Result<Self> {
        Self::try_new(db, name, typ, Vec::<GenericParameter>::new())
    }

    pub fn try_new(
        db: &dyn Ir,
        name: impl TryResult<PathName>,
        typ: Id<LogicalType>,
        parameters: impl IntoIterator<Item = impl TryResult<GenericParameter>>,
    ) -> Result<Self> {
        let mut parameter_assignments = InsertionOrderedMap::new();
        for param in parameters {
            let param = param.try_result()?;
            parameter_assignments.try_insert(
                param.name().clone(),
                GenericParameterAssignment::Default(param),
            )?;
        }
        let result = if parameter_assignments.len() > 0 {
            Self {
                name: name.try_result()?,
                typ,
                parameter_assignments: Some(parameter_assignments),
            }
        } else {
            Self {
                name: name.try_result()?,
                typ,
                parameter_assignments: None,
            }
        };
        result.verify_parameters(db)?;
        Ok(result)
    }

    pub fn parameter_assignments(
        &self,
    ) -> &Option<InsertionOrderedMap<Name, GenericParameterAssignment>> {
        &self.parameter_assignments
    }

    pub fn type_id(&self, db: &dyn Ir) -> Result<Id<LogicalType>> {
        if let Some(parameter_assignments) = self.parameter_assignments() {
            db.type_for_param_assignments(self.typ, parameter_assignments.clone())
        } else {
            Ok(self.typ)
        }
    }

    pub fn with_assignments(
        self,
        parameter_assignments: impl IntoIterator<
            Item = (impl TryOptional<Name>, impl TryResult<GenericParamValue>),
        >,
    ) -> Result<Self> {
        let attempted_parameter_assignments = parameter_assignments
            .into_iter()
            .map(|(x, a)| Ok((x.try_optional()?, a.try_result()?)))
            .collect::<Result<Vec<(Option<Name>, GenericParamValue)>>>()?;
        let typ_name = self.path_name().clone();
        let typ_id = self.typ.clone();
        if let Some(parameter_assignments) = self.parameter_assignments {
            let mut ordered_assignments = vec![];
            let mut named_assignments = vec![];
            let mut is_ordered: bool = true;
            for (opt_name, val) in attempted_parameter_assignments {
                if let Some(name) = opt_name {
                    is_ordered = false;
                    named_assignments.push((name, val));
                } else if is_ordered {
                    ordered_assignments.push(val);
                } else {
                    return Err(Error::InvalidArgument(
                        "Ordered parameter assignment must precede named assignments".to_string(),
                    ));
                }
            }
            if ordered_assignments.len() > parameter_assignments.len() {
                return Err(Error::InvalidArgument(
                    "More parameter assignments than there are parameters.".to_string(),
                ));
            }
            let mut new_parameter_assignments = InsertionOrderedMap::new();
            let mut ordered_assignments = ordered_assignments.into_iter();
            for (param_name, param_value) in parameter_assignments {
                if let Some(ordered_assignment) = ordered_assignments.next() {
                    let param_value = param_value.try_assign(ordered_assignment)?;
                    new_parameter_assignments.try_insert(param_name, param_value)?;
                } else {
                    new_parameter_assignments.try_insert(param_name, param_value)?;
                }
            }

            for (name, value) in named_assignments.into_iter() {
                if let Some(assignable) = new_parameter_assignments.get_mut(&name) {
                    *assignable = assignable.try_assign(value)?;
                } else {
                    return Err(Error::InvalidArgument(format!(
                        "No parameter with name {} on type {}",
                        name, typ_name,
                    )));
                }
            }

            Ok(Self {
                name: typ_name,
                typ: typ_id,
                parameter_assignments: Some(new_parameter_assignments),
            })
        } else {
            Err(Error::InvalidArgument(format!(
                "No parameters on type {}",
                self.path_name()
            )))
        }
    }

    pub fn with_name(self, name: impl TryResult<PathName>) -> Result<Self> {
        let name = name.try_result()?;
        Ok(Self {
            name,
            typ: self.typ,
            parameter_assignments: self.parameter_assignments,
        })
    }

    fn verify_parameters(&self, db: &dyn Ir) -> Result<()> {
        let expected_params = db.logical_type_parameter_kinds(self.typ)?;
        let expects_params = expected_params.len() > 0;
        if let Some(parameters) = self.parameter_assignments() {
            if !expects_params {
                return Err(Error::InvalidArgument(format!(
                    "Type {} declares parameters, but none are used.",
                    self.path_name()
                )));
            }
            for (expected_param_name, expected_param_kind) in expected_params.iter() {
                if let Some(param) = parameters.get(expected_param_name) {
                    if let Err(err) = param.kind().satisfies(expected_param_kind) {
                        return Err(Error::InvalidArgument(format!("Parameter {} does not satisfy the parameter expected the type defined by {}: {}", expected_param_name, self.path_name(), err)));
                    }
                } else {
                    return Err(Error::InterfaceError(format!(
                        "The type defined by {} expects a parameter with name {}, but none exists.",
                        self.path_name(),
                        expected_param_name
                    )));
                }
            }
            Ok(())
        } else if expects_params {
            Err(Error::InvalidArgument(format!(
                "Type defined by {} expects parameters, but none are declared.",
                self.path_name()
            )))
        } else {
            Ok(())
        }
    }

    pub fn parameters(&self) -> InsertionOrderedMap<Name, GenericParameter> {
        let mut result = InsertionOrderedMap::new();
        if let Some(assignments) = self.parameter_assignments().clone() {
            for (name, assignment) in assignments {
                match assignment {
                    GenericParameterAssignment::Default(p) => result.try_insert(name, p).unwrap(),
                    // TODO: May need to throw an error here, this shouldn't happen.
                    GenericParameterAssignment::Assigned(_, _) => (),
                }
            }
        }
        result
    }
}

impl Identify for TypeDeclaration {
    fn identifier(&self) -> String {
        self.path_name().to_string()
    }
}

impl PathNameSelf for TypeDeclaration {
    fn path_name(&self) -> &PathName {
        &self.name
    }
}

impl MoveDb<TypeDeclaration> for TypeDeclaration {
    fn move_db(
        &self,
        original_db: &dyn Ir,
        target_db: &dyn Ir,
        prefix: &Option<Name>,
    ) -> Result<TypeDeclaration> {
        Ok(Self {
            name: self.path_name().with_parents(prefix),
            typ: self.typ.move_db(original_db, target_db, prefix)?,
            parameter_assignments: self.parameter_assignments().clone(),
        })
    }
}
