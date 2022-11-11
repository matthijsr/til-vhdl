use tydi_common::{
    name::{PathName, PathNameSelf},
    traits::Identify,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ImportStatement {
    Full(PathName),
}

impl PathNameSelf for ImportStatement {
    fn path_name(&self) -> &PathName {
        match &self {
            ImportStatement::Full(name) => name,
        }
    }
}

impl Identify for ImportStatement {
    fn identifier(&self) -> String {
        self.path_name().to_string()
    }
}
