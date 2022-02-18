use std::{
    convert::TryFrom,
    fs::metadata,
    path::{Path, PathBuf},
    str::FromStr,
};

use tydi_common::error::{Error, Result};

/// This node represents a link to a behavioural `Implementation`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Link {
    /// The path to the implementation
    ///
    /// This refers to a directory, with the specific file being selected based on the streamlet
    /// as well as the back-end.
    path: PathBuf,
}

impl Link {
    pub fn try_new(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        match metadata(&path) {
            Ok(md) => {
                if md.is_dir() {
                    Ok(Link { path })
                } else {
                    Err(Error::FileIOError(format!(
                        "{} is not a directory",
                        path.display()
                    )))
                }
            }
            Err(err) => Err(Error::FileIOError(err.to_string())),
        }
    }

    pub fn path(&self) -> &Path {
        self.path.as_path()
    }
}

impl FromStr for Link {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Link::try_new(s)
    }
}

impl TryFrom<PathBuf> for Link {
    type Error = Error;

    fn try_from(value: PathBuf) -> Result<Self> {
        Link::try_new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_new() -> Result<()> {
        Link::try_new("../")?;
        Ok(())
    }

    #[test]
    fn try_new_fromstr() -> Result<()> {
        let expected = Link::try_new("../")?;
        let actual: Link = "../".parse()?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn try_invalid() -> Result<()> {
        match Link::try_new("gibberish") {
            Err(Error::FileIOError(_)) => Ok(()),
            Ok(_) => Err(Error::BackEndError("Expected FileIOError".to_string())),
            Err(err) => Err(err),
        }
    }
}
