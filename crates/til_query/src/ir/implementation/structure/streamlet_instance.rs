use std::sync::Arc;

use tydi_common::{
    error::{Error, Result, TryOptional, TryResult, WrapError},
    map::{InsertionOrderedMap, InsertionOrderedSet},
    name::{Name, NameSelf},
    traits::{Document, Documents, Identify},
};
use tydi_intern::Id;

use crate::ir::{
    interface_port::InterfacePort,
    physical_properties::{Domain, InterfaceDirection},
    streamlet::Streamlet,
    traits::GetSelf,
    Ir,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DomainAssignments {
    /// Definition has a list of Domains, and is assigned either a Default
    /// Domain (None), or a named Domain (Some).
    List(InsertionOrderedMap<Domain, Option<Domain>>),
    /// Definition has a Default Domain, and is assigned either a Default
    /// Domain (None), or a named Domain (Some).
    Default(Option<Domain>),
}

impl DomainAssignments {
    fn get_assignment_default(&self) -> Result<Option<&Domain>> {
        match self {
            DomainAssignments::List(_) => Err(Error::ProjectError(
                "Attempting to determine domain for Default, but Streamlet has named Domains."
                    .to_string(),
            )),
            DomainAssignments::Default(res) => Ok(res.as_ref()),
        }
    }

    pub fn get_assignment(&self, base_domain: Option<&Domain>) -> Result<Option<&Domain>> {
        match base_domain {
            Some(base_domain_name) => match self {
                DomainAssignments::List(domain_map) => Ok(domain_map
                    .try_get(base_domain_name)
                    .wrap_err(Error::ProjectError(format!(
                        "Cannot find assigned domain for {}, no such domain on Interface",
                        base_domain_name
                    )))?
                    .as_ref()),
                DomainAssignments::Default(res) => Ok(res.as_ref()),
            },
            None => self.get_assignment_default(),
        }
    }

    pub fn assigned_domains(&self) -> Result<Option<InsertionOrderedSet<Domain>>> {
        match self {
            DomainAssignments::List(list) => {
                let mut set = InsertionOrderedSet::new();
                let mut has_default = false;
                for assigned in list.values() {
                    match assigned {
                        Some(name) => {
                            set.insert(name.clone());
                        }
                        None => {
                            has_default = true;
                        }
                    }
                }
                if has_default {
                    if set.len() > 0 {
                        Err(Error::ProjectError(
                            "Streamlet has both Default and named domains assigned.".to_string(),
                        ))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(Some(set))
                }
            }
            DomainAssignments::Default(res) => match res {
                Some(assigned) => {
                    let mut set = InsertionOrderedSet::new();
                    set.try_insert(assigned.clone())?;
                    Ok(Some(set))
                }
                None => Ok(None),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StreamletInstance {
    name: Name,
    definition: Arc<Streamlet>,
    domain_assignments: DomainAssignments,
    ports: InsertionOrderedMap<Name, InterfacePort>,
    doc: Option<String>,
}

impl StreamletInstance {
    pub fn new_assign_default(
        db: &dyn Ir,
        name: impl TryResult<Name>,
        definition: Id<Arc<Streamlet>>,
    ) -> Result<Self> {
        let name = name.try_result()?;
        let definition = definition.get(db);
        let base_domains = definition.domains(db);
        let domain_assignments = match base_domains {
            Some(named_domains) => {
                if named_domains.len() > 0 {
                    let mut result_assignments = InsertionOrderedMap::new();
                    for name in named_domains.iter() {
                        result_assignments.try_insert(name.clone(), None)?;
                    }
                    DomainAssignments::List(result_assignments)
                } else {
                    return Err(Error::ProjectError(format!("Streamlet {} has an empty named domain list. Should be None (= Default Domain).", definition.identifier())));
                }
            }
            None => DomainAssignments::Default(None),
        };
        let mut ports = definition.ports(db);
        for port in ports.values_mut() {
            port.set_domain(domain_assignments.get_assignment(port.domain())?.cloned());
        }

        Ok(Self {
            name,
            definition,
            domain_assignments,
            ports,
            doc: None,
        })
    }

    pub fn new(
        db: &dyn Ir,
        name: impl TryResult<Name>,
        definition: Id<Arc<Streamlet>>,
        assignments: impl IntoIterator<Item = (impl TryOptional<Domain>, impl TryResult<Domain>)>,
    ) -> Result<Self> {
        let name = name.try_result()?;
        let mut assignments = assignments
            .into_iter()
            .map(|(x, a)| Ok((x.try_optional()?, a.try_result()?)))
            .collect::<Result<Vec<(Option<Domain>, Domain)>>>()?;
        if assignments.len() > 0 {
            let definition = definition.get(db);
            let base_domains = definition.domains(db);
            let domain_assignments = match base_domains {
                Some(named_domains) => {
                    if named_domains.len() > 0 {
                        if named_domains.len() != assignments.len() {
                            return Err(Error::InvalidArgument(format!("Domain assignment list does not match base domain list length. Base: {}, Assignments: {}", named_domains.len(), assignments.len())));
                        }
                        let mut result_assignments = InsertionOrderedMap::new();
                        let mut ordered_assignments = vec![];
                        let mut named_assignments = InsertionOrderedMap::new();
                        let mut used_name = false;
                        for (a_name, a_val) in assignments {
                            match a_name {
                                Some(a_name) => {
                                    used_name = true;
                                    if named_domains.contains(&a_name) {
                                        named_assignments.try_insert(a_name, a_val).wrap_err(
                                            Error::InvalidArgument(
                                                "Duplicate domain assignment".to_string(),
                                            ),
                                        )?;
                                    } else {
                                        return Err(Error::InvalidArgument(format!(
                                            "No Domain named {} on Streamlet {}",
                                            &a_name,
                                            definition.identifier()
                                        )));
                                    }
                                }
                                None => {
                                    if used_name {
                                        return Err(Error::InvalidArgument("Cannot use nameless Domain assignment after using a name.".to_string()));
                                    } else {
                                        ordered_assignments.push(a_val);
                                    }
                                }
                            }
                        }
                        let mut ordered_assignments = ordered_assignments.into_iter();
                        for base_name in named_domains.into_iter() {
                            if let Some(a_val) = ordered_assignments.next() {
                                result_assignments.try_insert(base_name, Some(a_val))?;
                            } else {
                                let n_val = named_assignments.try_get(&base_name).wrap_err(
                                    Error::InvalidArgument("Missing domain assignment".to_string()),
                                )?;
                                result_assignments.try_insert(base_name, Some(n_val.clone()))?;
                            }
                        }

                        DomainAssignments::List(result_assignments)
                    } else {
                        return Err(Error::ProjectError(format!("Streamlet {} has an empty named domain list. Should be None (= Default Domain).", definition.identifier())));
                    }
                }
                None => {
                    if assignments.len() > 1 {
                        return Err(Error::ProjectError("Attempting to assign multiple Domains, but Streamlet should only have one (Default) domain.".to_string()));
                    } else if assignments.len() == 1 {
                        DomainAssignments::Default(Some(assignments.pop().unwrap().1))
                    } else {
                        unreachable!()
                    }
                }
            };
            let mut ports = definition.ports(db);
            for port in ports.values_mut() {
                port.set_domain(domain_assignments.get_assignment(port.domain())?.cloned());
            }

            Ok(Self {
                name,
                definition,
                domain_assignments,
                ports,
                doc: None,
            })
        } else {
            Self::new_assign_default(db, name, definition)
        }
    }

    pub fn definition(&self) -> Arc<Streamlet> {
        self.definition.clone()
    }

    pub fn assigned_domains(&self) -> Result<Option<InsertionOrderedSet<Domain>>> {
        self.domain_assignments.assigned_domains()
    }

    pub fn ports(&self) -> &InsertionOrderedMap<Name, InterfacePort> {
        &self.ports
    }

    pub fn inputs(&self) -> impl Iterator<Item = &InterfacePort> {
        self.ports()
            .into_iter()
            .map(|(_, x)| x)
            .filter(|x| x.physical_properties().direction() == InterfaceDirection::In)
    }

    pub fn outputs(&self) -> impl Iterator<Item = &InterfacePort> {
        self.ports()
            .into_iter()
            .map(|(_, x)| x)
            .filter(|x| x.physical_properties().direction() == InterfaceDirection::Out)
    }

    pub fn try_get_port(&self, name: &Name) -> Result<InterfacePort> {
        match self.ports().get(name) {
            Some(port) => Ok(port.clone()),
            None => Err(Error::InvalidArgument(format!(
                "No port with name {} exists on this Streamlet instance",
                name
            ))),
        }
    }

    pub fn domain_assignments(&self) -> &DomainAssignments {
        &self.domain_assignments
    }
}

impl Document for StreamletInstance {
    fn doc(&self) -> Option<&String> {
        self.doc.as_ref()
    }
}

impl Documents for StreamletInstance {
    fn set_doc(&mut self, doc: impl Into<String>) {
        self.doc = Some(doc.into())
    }
}

impl Identify for StreamletInstance {
    fn identifier(&self) -> String {
        self.name.to_string()
    }
}

impl NameSelf for StreamletInstance {
    fn name(&self) -> &Name {
        &self.name
    }
}