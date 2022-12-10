use tydi_common::{
    error::{Result, TryResult},
    name::{Name, NameSelf},
    traits::Identify,
};

use self::{
    behavioral::BehavioralGenericKind, condition::GenericCondition, interface::InterfaceGenericKind,
};

pub mod behavioral;
pub mod condition;
pub mod interface;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GenericKind {
    Behavioral(BehavioralGenericKind),
    Interface(InterfaceGenericKind),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GenericParameter {
    name: Name,
    kind: GenericKind,
    conditions: Vec<GenericCondition>,
}

impl GenericParameter {
    fn check_conditions(self) -> Result<Self> {
        self.kind().verify_conditions(self.conditions())?;
        Ok(self)
    }

    pub fn try_new(
        name: impl TryResult<Name>,
        kind: impl TryResult<GenericKind>,
        conditions: impl IntoIterator<Item = impl TryResult<GenericCondition>>,
    ) -> Result<Self> {
        Self {
            name: name.try_result()?,
            kind: kind.try_result()?,
            conditions: conditions
                .into_iter()
                .map(|x| x.try_result())
                .collect::<Result<Vec<_>>>()?,
        }
        .check_conditions()
    }

    pub fn kind(&self) -> &GenericKind {
        &self.kind
    }

    pub fn conditions(&self) -> &[GenericCondition] {
        self.conditions.as_ref()
    }
}

impl Identify for GenericParameter {
    fn identifier(&self) -> String {
        self.name().to_string()
    }
}

impl NameSelf for GenericParameter {
    fn name(&self) -> &Name {
        &self.name
    }
}

pub trait VerifyConditions {
    fn verify_conditions(&self, conditions: &[GenericCondition]) -> Result<()>;
}

impl VerifyConditions for GenericKind {
    fn verify_conditions(&self, conditions: &[GenericCondition]) -> Result<()> {
        match self {
            GenericKind::Behavioral(b) => b.verify_conditions(conditions),
            GenericKind::Interface(i) => i.verify_conditions(conditions),
        }
    }
}
