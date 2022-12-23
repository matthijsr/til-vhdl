use std::{
    convert::{TryFrom, TryInto},
    fmt,
    iter::FromIterator,
    ops::Deref,
    str::FromStr,
};

use tydi_common::{
    error::{Error, Result, TryOptionalFrom, TryResult},
    name::{Name, PathName},
};
use uncased::Uncased;
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
pub struct VhdlName(Uncased<'static>);

impl VhdlName {
    /// Constructs a new name wrapper. Returns an error when the provided name
    /// is invalid.
    pub fn try_new(name: impl Into<String>) -> Result<Self> {
        let name: String = name.into();

        if name.is_empty() {
            Err(Error::InvalidArgument("name cannot be empty".to_string()))
        } else if name.chars().next().unwrap().is_ascii_digit() {
            Err(Error::InvalidArgument(format!(
                "{}: name cannot start with a digit",
                name
            )))
        } else if name.starts_with('_') || name.ends_with('_') {
            Err(Error::InvalidArgument(format!(
                "{}: name cannot start or end with an underscore",
                name
            )))
        } else if name.contains("__") {
            Err(Error::InvalidArgument(format!(
                "{}: name cannot contain two or more consecutive underscores",
                name
            )))
        } else if !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c.eq(&'_'))
        {
            Err(Error::InvalidArgument(
                format!(
                    "{}: name must consist of letters, numbers, and/or underscores",
                    name
                )
                .to_string(),
            ))
        } else {
            Ok(VhdlName(name.into()))
        }
    }
}

impl From<VhdlName> for String {
    fn from(name: VhdlName) -> Self {
        name.0.into_string()
    }
}

impl From<&VhdlName> for String {
    fn from(name: &VhdlName) -> Self {
        name.0.clone().into_string()
    }
}

impl From<Name> for VhdlName {
    fn from(name: Name) -> Self {
        VhdlName::try_new(name).unwrap()
    }
}

impl From<PathName> for VhdlName {
    fn from(path: PathName) -> Self {
        VhdlName::try_new(path.join("_0_")).unwrap()
    }
}

impl From<&PathName> for VhdlName {
    fn from(path: &PathName) -> Self {
        VhdlName::try_new(path.join("_0_")).unwrap()
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

impl TryOptionalFrom<&str> for VhdlName {
    fn optional_result_from(str: &str) -> Option<Result<Self>> {
        if str.trim() == "" {
            None
        } else {
            Some(VhdlName::try_new(str))
        }
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

// TODO: VhdlPathName is kind of useless, since libraries/packages can't be nested.

/// Type-safe path for names.
///
/// Allows wrapping a set of valid names in a hierarchy.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VhdlPathName(Vec<VhdlName>);

impl VhdlPathName {
    pub fn new_empty() -> Self {
        VhdlPathName(Vec::new())
    }

    pub fn new(names: impl Iterator<Item = VhdlName>) -> Self {
        VhdlPathName(names.collect())
    }

    pub fn try_new(names: impl IntoIterator<Item = impl TryResult<VhdlName>>) -> Result<Self> {
        Ok(VhdlPathName(
            names
                .into_iter()
                .map(|name| name.try_result())
                .collect::<Result<_>>()?,
        ))
    }

    /// Returns true if this VhdlPathName is empty (∅).
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn push(&mut self, name: impl Into<VhdlName>) {
        self.0.push(name.into())
    }

    pub fn with_parents(&self, path: impl Into<VhdlPathName>) -> VhdlPathName {
        let parent = path.into();
        let mut result: Vec<VhdlName> = Vec::with_capacity(self.len() + parent.len());
        result.extend(parent.0.into_iter());
        result.extend(self.0.clone().into_iter());
        VhdlPathName::new(result.into_iter())
    }

    pub fn with_parent(&self, name: impl Into<VhdlName>) -> VhdlPathName {
        let mut result: Vec<VhdlName> = Vec::with_capacity(self.len() + 1);
        result.push(name.into());
        result.extend(self.0.clone().into_iter());
        VhdlPathName::new(result.into_iter())
    }

    pub fn with_child(&self, name: impl Into<VhdlName>) -> VhdlPathName {
        let mut result: Vec<VhdlName> = Vec::with_capacity(self.len() + 1);
        result.extend(self.0.clone().into_iter());
        result.push(name.into());
        VhdlPathName::new(result.into_iter())
    }

    pub fn with_children(&self, path: impl Into<VhdlPathName>) -> VhdlPathName {
        let parent = path.into();
        let mut result: Vec<VhdlName> = Vec::with_capacity(self.len() + parent.len());
        result.extend(self.0.clone().into_iter());
        result.extend(parent.0.into_iter());
        VhdlPathName::new(result.into_iter())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn first(&self) -> Option<&VhdlName> {
        self.0.first()
    }

    pub fn last(&self) -> Option<&VhdlName> {
        self.0.last()
    }

    pub fn parent(&self) -> Option<VhdlPathName> {
        if self.is_empty() {
            None
        } else {
            Some(VhdlPathName(self.0[..self.len() - 1].to_vec()))
        }
    }

    /// Returns all but the last part of the VhdlPathName
    pub fn root(&self) -> VhdlPathName {
        if self.is_empty() {
            self.clone()
        } else {
            let mut names = self.0.clone();
            names.pop();
            VhdlPathName(names)
        }
    }
}

impl fmt::Display for VhdlPathName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut result = String::new();
        let mut names = self.0.iter().map(|x| x.as_ref());
        if let Some(x) = names.next() {
            result.push_str(x);
            names.for_each(|name| {
                result.push_str(".");
                result.push_str(name);
            });
        } else {
            result.push_str("");
        }
        write!(f, "{}", result)
    }
}

impl AsRef<[VhdlName]> for VhdlPathName {
    fn as_ref(&self) -> &[VhdlName] {
        self.0.as_slice()
    }
}

impl<'a> IntoIterator for &'a VhdlPathName {
    type Item = &'a VhdlName;
    type IntoIter = std::slice::Iter<'a, VhdlName>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl FromIterator<Name> for VhdlPathName {
    fn from_iter<I: IntoIterator<Item = Name>>(iter: I) -> Self {
        VhdlPathName(iter.into_iter().map(|name| name.into()).collect())
    }
}

impl FromIterator<VhdlName> for VhdlPathName {
    fn from_iter<I: IntoIterator<Item = VhdlName>>(iter: I) -> Self {
        VhdlPathName(iter.into_iter().collect())
    }
}

impl From<VhdlName> for VhdlPathName {
    fn from(name: VhdlName) -> Self {
        VhdlPathName(vec![name])
    }
}

impl From<&VhdlName> for VhdlPathName {
    fn from(value: &VhdlName) -> Self {
        VhdlPathName::from(value.clone())
    }
}

impl From<&Option<VhdlName>> for VhdlPathName {
    fn from(value: &Option<VhdlName>) -> Self {
        match value {
            Some(name) => VhdlPathName::from(name),
            None => VhdlPathName::new_empty(),
        }
    }
}

impl TryFrom<String> for VhdlPathName {
    type Error = Error;
    fn try_from(string: String) -> Result<Self> {
        if string.trim() == "" {
            Ok(VhdlPathName::new_empty())
        } else if string.contains(".") {
            VhdlPathName::try_new(string.split("."))
        } else if string.contains("__") {
            VhdlPathName::try_new(string.split("__"))
        } else {
            let name: VhdlName = string.try_into()?;
            Ok(VhdlPathName::from(name))
        }
    }
}

impl TryFrom<&str> for VhdlPathName {
    type Error = Error;
    fn try_from(str: &str) -> Result<Self> {
        if str.trim() == "" {
            Ok(VhdlPathName::new_empty())
        } else if str.contains(".") {
            VhdlPathName::try_new(str.split("."))
        } else if str.contains("__") {
            VhdlPathName::try_new(str.split("__"))
        } else {
            let name: VhdlName = str.try_into()?;
            Ok(VhdlPathName::from(name))
        }
    }
}

impl From<&PathName> for VhdlPathName {
    fn from(path: &PathName) -> Self {
        VhdlPathName::new(path.into_iter().map(|n| VhdlName::from(n.clone())))
    }
}

impl From<VhdlPathName> for String {
    fn from(name: VhdlPathName) -> Self {
        name.to_string()
    }
}

impl From<&VhdlPathName> for String {
    fn from(name: &VhdlPathName) -> Self {
        name.to_string()
    }
}

pub trait VhdlNameSelf {
    fn vhdl_name(&self) -> &VhdlName;
}

pub trait VhdlPathNameSelf {
    fn vhdl_path_name(&self) -> &VhdlPathName;
}
