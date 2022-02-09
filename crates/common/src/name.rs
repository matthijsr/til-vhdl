use std::{
    convert::{TryFrom, TryInto},
    fmt,
    iter::FromIterator,
    ops::Deref,
    str::FromStr,
};

use crate::{
    error::{Error, Result, TryOptionalFrom, TryResult},
    traits::Identify,
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
pub struct Name(String);

impl Name {
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
        } else if name.contains("__") {
            Err(Error::InvalidArgument(
                "name cannot contain two or more consecutive underscores".to_string(),
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
            Ok(Name(name))
        }
    }
}

impl From<Name> for String {
    fn from(name: Name) -> Self {
        name.0
    }
}

impl From<&Name> for String {
    fn from(name: &Name) -> Self {
        name.0.clone()
    }
}

impl From<&Name> for Name {
    fn from(value: &Name) -> Self {
        value.clone()
    }
}

impl Deref for Name {
    type Target = str;
    fn deref(&self) -> &str {
        self.0.as_ref()
    }
}

impl TryFrom<&str> for Name {
    type Error = Error;
    fn try_from(str: &str) -> Result<Self> {
        Name::try_new(str)
    }
}

impl TryFrom<&String> for Name {
    type Error = Error;
    fn try_from(str: &String) -> Result<Self> {
        Name::try_new(str)
    }
}

impl TryFrom<String> for Name {
    type Error = Error;
    fn try_from(string: String) -> Result<Self> {
        Name::try_new(string)
    }
}

impl TryOptionalFrom<&str> for Name {
    fn optional_result_from(str: &str) -> Option<Result<Self>> {
        if str.trim() == "" {
            None
        } else {
            Some(Name::try_new(str))
        }
    }
}

impl FromStr for Name {
    type Err = Error;
    fn from_str(str: &str) -> Result<Self> {
        Name::try_new(str)
    }
}

impl PartialEq<String> for Name {
    fn eq(&self, other: &String) -> bool {
        &self.0 == other
    }
}

impl PartialEq<str> for Name {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Type-safe path for names.
///
/// Allows wrapping a set of valid names in a hierarchy.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PathName(Vec<Name>);

impl PathName {
    pub fn new_empty() -> Self {
        PathName(Vec::new())
    }

    pub fn new(names: impl Iterator<Item = Name>) -> Self {
        PathName(names.collect())
    }

    pub fn try_new(names: impl IntoIterator<Item = impl TryResult<Name>>) -> Result<Self> {
        Ok(PathName(
            names
                .into_iter()
                .map(|name| name.try_result())
                .collect::<Result<_>>()?,
        ))
    }

    /// Returns true if this PathName is empty (∅).
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn push(&mut self, name: impl Into<Name>) {
        self.0.push(name.into())
    }

    pub fn with_parents(&self, path: impl Into<PathName>) -> PathName {
        let parent = path.into();
        let mut result: Vec<Name> = Vec::with_capacity(self.len() + parent.len());
        result.extend(parent.0.into_iter());
        result.extend(self.0.clone().into_iter());
        PathName::new(result.into_iter())
    }

    pub fn with_parent(&self, name: impl Into<Name>) -> PathName {
        let mut result: Vec<Name> = Vec::with_capacity(self.len() + 1);
        result.push(name.into());
        result.extend(self.0.clone().into_iter());
        PathName::new(result.into_iter())
    }

    pub fn with_child(&self, name: impl Into<Name>) -> PathName {
        let mut result: Vec<Name> = Vec::with_capacity(self.len() + 1);
        result.extend(self.0.clone().into_iter());
        result.push(name.into());
        PathName::new(result.into_iter())
    }

    pub fn with_children(&self, path: impl Into<PathName>) -> PathName {
        let parent = path.into();
        let mut result: Vec<Name> = Vec::with_capacity(self.len() + parent.len());
        result.extend(self.0.clone().into_iter());
        result.extend(parent.0.into_iter());
        PathName::new(result.into_iter())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn first(&self) -> Option<&Name> {
        self.0.first()
    }

    pub fn last(&self) -> Option<&Name> {
        self.0.last()
    }

    pub fn parent(&self) -> Option<PathName> {
        if self.is_empty() {
            None
        } else {
            Some(PathName(self.0[..self.len() - 1].to_vec()))
        }
    }

    /// Returns all but the last part of the PathName
    pub fn root(&self) -> PathName {
        if self.is_empty() {
            self.clone()
        } else {
            let mut names = self.0.clone();
            names.pop();
            PathName(names)
        }
    }
}

impl fmt::Display for PathName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut result = String::new();
        let mut names = self.0.iter().map(|x| x.as_ref());
        if let Some(x) = names.next() {
            result.push_str(x);
            names.for_each(|name| {
                result.push_str("__");
                result.push_str(name);
            });
        } else {
            result.push_str("");
        }
        write!(f, "{}", result)
    }
}

impl AsRef<[Name]> for PathName {
    fn as_ref(&self) -> &[Name] {
        self.0.as_slice()
    }
}

impl<'a> IntoIterator for &'a PathName {
    type Item = &'a Name;
    type IntoIter = std::slice::Iter<'a, Name>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl FromIterator<Name> for PathName {
    fn from_iter<I: IntoIterator<Item = Name>>(iter: I) -> Self {
        PathName(iter.into_iter().collect())
    }
}

impl From<Name> for PathName {
    fn from(name: Name) -> Self {
        PathName(vec![name])
    }
}

impl From<&Name> for PathName {
    fn from(value: &Name) -> Self {
        PathName::from(value.clone())
    }
}

impl From<&Option<Name>> for PathName {
    fn from(value: &Option<Name>) -> Self {
        match value {
            Some(name) => PathName::from(name),
            None => PathName::new_empty(),
        }
    }
}

impl TryFrom<String> for PathName {
    type Error = Error;
    fn try_from(string: String) -> Result<Self> {
        if string.trim() == "" {
            Ok(PathName::new_empty())
        } else if string.contains(".") {
            PathName::try_new(string.split("."))
        } else if string.contains("__") {
            PathName::try_new(string.split("__"))
        } else {
            let name: Name = string.try_into()?;
            Ok(PathName::from(name))
        }
    }
}

impl TryFrom<&str> for PathName {
    type Error = Error;
    fn try_from(str: &str) -> Result<Self> {
        if str.trim() == "" {
            Ok(PathName::new_empty())
        } else if str.contains(".") {
            PathName::try_new(str.split("."))
        } else if str.contains("__") {
            PathName::try_new(str.split("__"))
        } else {
            let name: Name = str.try_into()?;
            Ok(PathName::from(name))
        }
    }
}

impl TryFrom<Vec<String>> for PathName {
    type Error = Error;

    fn try_from(value: Vec<String>) -> Result<Self> {
        PathName::try_new(value)
    }
}

impl From<PathName> for String {
    fn from(name: PathName) -> Self {
        name.to_string()
    }
}

impl From<&PathName> for String {
    fn from(name: &PathName) -> Self {
        name.to_string()
    }
}

pub trait NameSelf: Identify {
    fn name(&self) -> &Name;
}

pub trait PathNameSelf: Identify {
    fn path_name(&self) -> &PathName;
}
