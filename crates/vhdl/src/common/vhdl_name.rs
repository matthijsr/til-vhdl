use std::{convert::TryFrom, fmt, ops::Deref, str::FromStr};

use tydi_common::{
    error::{Error, Result},
    name::{Name, PathName},
};
/// Type-safe wrapper for valid names.
///
/// The following rules apply for valid names
/// - The name is non-empty
/// - The name consists of letter, number and underscores
/// - The name does not start or end with an underscore
/// - The name does not start with a digit
/// - The name does not contain double underscores
///
/// # Examples
///
/// ```rust
/// ```
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VhdlName(String);

impl VhdlName {
    /// Constructs a new name wrapper. Returns an error when the provided name
    /// is invalid.
    pub fn try_new(name: impl Into<String>) -> Result<Self> {
        let name = name.into();
        if name.is_empty() {
            Err(Error::InvalidArgument("name cannot be empty".to_string()))
        } else if name.chars().next().unwrap().is_ascii_digit() {
            Err(Error::InvalidArgument(
                "name cannot start with a digit".to_string(),
            ))
        } else if name.starts_with('_') || name.ends_with('_') {
            Err(Error::InvalidArgument(
                "name cannot start or end with an underscore".to_string(),
            ))
        } else if !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c.eq(&'_'))
        {
            Err(Error::InvalidArgument(
                format!(
                    "name must consist of letters, numbers, and/or underscores {}",
                    name
                )
                .to_string(),
            ))
        } else {
            Ok(VhdlName(name))
        }
    }
}

impl From<VhdlName> for String {
    fn from(name: VhdlName) -> Self {
        name.0
    }
}

impl From<&VhdlName> for String {
    fn from(name: &VhdlName) -> Self {
        name.0.clone()
    }
}

impl From<Name> for VhdlName {
    fn from(name: Name) -> Self {
        VhdlName::try_new(name).unwrap()
    }
}

impl From<PathName> for VhdlName {
    fn from(path: PathName) -> Self {
        VhdlName::try_new(path.to_string()).unwrap()
    }
}

impl Deref for VhdlName {
    type Target = str;
    fn deref(&self) -> &str {
        self.0.as_ref()
    }
}

impl TryFrom<&str> for VhdlName {
    type Error = Error;
    fn try_from(str: &str) -> Result<Self> {
        VhdlName::try_new(str)
    }
}

impl TryFrom<String> for VhdlName {
    type Error = Error;
    fn try_from(string: String) -> Result<Self> {
        VhdlName::try_new(string)
    }
}

impl FromStr for VhdlName {
    type Err = Error;
    fn from_str(str: &str) -> Result<Self> {
        VhdlName::try_new(str)
    }
}

impl PartialEq<String> for VhdlName {
    fn eq(&self, other: &String) -> bool {
        &self.0 == other
    }
}

impl PartialEq<str> for VhdlName {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl fmt::Display for VhdlName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub trait VhdlNameSelf {
    fn vhdl_name(&self) -> &VhdlName;
}
