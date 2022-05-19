use std::iter;

use til_query::ir::physical_properties::Domain;
use tydi_common::{
    map::{InsertionOrderedMap, InsertionOrderedSet},
    name::{Name, PathName},
};
use tydi_intern::Id;
use tydi_vhdl::{architecture::arch_storage::Arch, declaration::ObjectDeclaration, port::Port};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum VhdlDomainListOrDefault<T: Clone + PartialEq + Eq> {
    List(InsertionOrderedMap<Domain, VhdlDomain<T>>),
    Default(VhdlDomain<T>),
}

impl<T: Clone + PartialEq + Eq> VhdlDomainListOrDefault<T> {
    pub fn iterable(&self) -> impl IntoIterator<Item = &VhdlDomain<T>> {
        match self {
            VhdlDomainListOrDefault::List(list) => list.values().collect::<Vec<_>>(),
            VhdlDomainListOrDefault::Default(res) => iter::once(res).collect::<Vec<_>>(),
        }
    }
}

impl VhdlDomainListOrDefault<Port> {
    pub fn into_entity_objects(
        &self,
        arch_db: &dyn Arch,
    ) -> VhdlDomainListOrDefault<Id<ObjectDeclaration>> {
        match self {
            VhdlDomainListOrDefault::List(dmap) => VhdlDomainListOrDefault::List(
                dmap.clone().map_convert(|x| x.into_entity_objects(arch_db)),
            ),
            VhdlDomainListOrDefault::Default(res) => {
                VhdlDomainListOrDefault::Default(res.into_entity_objects(arch_db))
            }
        }
    }
}

impl From<Option<InsertionOrderedSet<Domain>>> for VhdlDomainListOrDefault<Port> {
    fn from(set: Option<InsertionOrderedSet<Domain>>) -> Self {
        match set {
            Some(set) => {
                let mut dmap = InsertionOrderedMap::new();
                for name in set.into_iter() {
                    let vdomain = VhdlDomain::from(&name);
                    dmap.try_insert(name, vdomain).unwrap();
                }
                Self::List(dmap)
            }
            None => Self::Default(VhdlDomain::<Port>::default()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VhdlDomain<T: Clone + PartialEq + Eq> {
    clock: T,
    reset: T,
}

impl<T: Clone + PartialEq + Eq> VhdlDomain<T> {
    pub fn new(clock: T, reset: T) -> Self {
        Self { clock, reset }
    }

    /// Get a reference to the vhdl domain's clock.
    #[must_use]
    pub fn clock(&self) -> &T {
        &self.clock
    }

    /// Get a reference to the vhdl domain's reset.
    #[must_use]
    pub fn reset(&self) -> &T {
        &self.reset
    }
}

impl VhdlDomain<Port> {
    pub fn default() -> Self {
        Self {
            clock: Port::clk(),
            reset: Port::rst(),
        }
    }

    pub fn into_entity_objects(&self, arch_db: &dyn Arch) -> VhdlDomain<Id<ObjectDeclaration>> {
        VhdlDomain::new(
            ObjectDeclaration::from_port(arch_db, self.clock(), true),
            ObjectDeclaration::from_port(arch_db, self.reset(), true),
        )
    }
}

impl VhdlDomain<Id<ObjectDeclaration>> {
    pub fn default(arch_db: &dyn Arch) -> Self {
        Self {
            clock: ObjectDeclaration::entity_clk(arch_db),
            reset: ObjectDeclaration::entity_rst(arch_db),
        }
    }
}

impl From<&Domain> for VhdlDomain<Port> {
    fn from(domain: &Domain) -> Self {
        let clk = PathName::from(domain.clone()).with_child(Name::try_new("clk").unwrap());
        let rst = PathName::from(domain.clone()).with_child(Name::try_new("rst").unwrap());
        Self::new(
            Port::try_bit_in(clk).unwrap(),
            Port::try_bit_in(rst).unwrap(),
        )
    }
}
