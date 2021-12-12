use std::{cmp::Ordering, convert::TryFrom, fmt, str::FromStr};

use crate::common::{
    error::{Error, Result},
    integers::NonNegative,
};

/// Interface complexity level.
///
/// This logical stream parameter specifies the guarantees a source makes about
/// how elements are transferred. Equivalently, it specifies the assumptions a
/// sink can safely make.
///
/// # Examples
///
/// ```rust
/// use tydi::physical::Complexity;
///
/// let c3 = Complexity::new_major(3);
/// let c30 = Complexity::new(vec![3, 0])?;
/// let c31 = Complexity::new(vec![3, 1])?;
/// let c4 = Complexity::new_major(4);
///
/// assert_eq!(c3, c30);
/// assert!(c3 < c31);
/// assert!(c31 < c4);
///
/// assert_eq!(c31.to_string(), "3.1");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// [Reference](https://abs-tudelft.github.io/tydi/specification/physical.html#complexity-c)
#[derive(Debug, Clone, Hash)]
pub struct Complexity {
    /// The complexity level.
    level: Vec<NonNegative>,
}

impl Default for Complexity {
    fn default() -> Self {
        Complexity { level: vec![4] }
    }
}

impl PartialEq for Complexity {
    /// A complexity number is higher than another when the leftmost integer is
    /// greater, and lower when the leftmost integer is lower. If the leftmost
    /// integer is equal, the next integer is checked recursively. If one
    /// complexity number has more entries than another, the shorter number is
    /// padded with zeros on the right.
    fn eq(&self, other: &Self) -> bool {
        (0..self.level.len().max(other.level.len()))
            .all(|idx| self.level.get(idx).unwrap_or(&0) == other.level.get(idx).unwrap_or(&0))
    }
}

impl Eq for Complexity {}

impl PartialOrd for Complexity {
    fn partial_cmp(&self, other: &Complexity) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Complexity {
    /// A complexity number is higher than another when the leftmost integer is
    /// greater, and lower when the leftmost integer is lower. If the leftmost
    /// integer is equal, the next integer is checked recursively. If one
    /// complexity number has more entries than another, the shorter number is
    /// padded with zeros on the right.
    fn cmp(&self, other: &Complexity) -> Ordering {
        (0..self.level.len().max(other.level.len()))
            .map(|idx| {
                (
                    self.level.get(idx).unwrap_or(&0),
                    other.level.get(idx).unwrap_or(&0),
                )
            })
            .fold(None, |ord, (i, j)| match ord {
                Some(ord) => Some(ord),
                None => {
                    if i == j {
                        None
                    } else {
                        Some(i.cmp(j))
                    }
                }
            })
            .unwrap_or(Ordering::Equal)
    }
}

impl From<NonNegative> for Complexity {
    /// Convert a NonNegative into complexity with the NonNegative as major version.
    fn from(major: NonNegative) -> Self {
        Complexity::new_major(major)
    }
}

impl TryFrom<Vec<NonNegative>> for Complexity {
    type Error = Error;
    /// Try to convert a vector of NonNegative into a complexity. Returns an
    /// error when the provided vector is empty.
    fn try_from(level: Vec<NonNegative>) -> Result<Self> {
        Complexity::new(level)
    }
}

impl FromStr for Complexity {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Complexity::new(
            // split string into string slices
            s.split('.')
                // convert slices to nonnegatives after trimming whitespace
                .map(|d| d.trim().parse::<NonNegative>())
                // convert to result with vector of nonnegatives
                .collect::<std::result::Result<Vec<_>, std::num::ParseIntError>>()
                // convert potential error to tydi error
                .map_err(|e| Error::InvalidArgument(e.to_string()))?,
        )
    }
}

impl Complexity {
    /// Constructs a new Complexity with provided level. Returns an error when
    /// the provided level iterator is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tydi::physical::Complexity;
    ///
    /// let c = Complexity::new(vec![1, 2, 3, 4])?;
    /// assert!(Complexity::new(vec![]).is_err());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub(crate) fn new(level: impl IntoIterator<Item = NonNegative>) -> Result<Self> {
        let level = level.into_iter().collect::<Vec<NonNegative>>();
        if level.is_empty() {
            Err(Error::InvalidArgument(
                "complexity level cannot be empty".to_string(),
            ))
        } else {
            Ok(Complexity { level })
        }
    }

    /// Constructs a new Complexity with provided level as major version.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tydi::physical::Complexity;
    ///
    /// let c = Complexity::new_major(4);
    ///
    /// assert_eq!(c, Complexity::new(vec![4])?);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub(crate) fn new_major(level: NonNegative) -> Self {
        Complexity { level: vec![level] }
    }

    /// Returns the level of this Complexity.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tydi::physical::Complexity;
    ///
    /// let c = Complexity::new(vec![3, 14])?;
    /// assert_eq!(c.level(), &[3, 14]);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub(crate) fn level(&self) -> &[NonNegative] {
        self.level.as_ref()
    }

    /// Returns the major version of this Complexity level.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tydi::physical::Complexity;
    ///
    /// let c = Complexity::new(vec![3, 14])?;
    /// assert_eq!(c.major(), 3);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    pub(crate) fn major(&self) -> NonNegative {
        self.level[0]
    }
}

impl fmt::Display for Complexity {
    /// Display a complexity level as a version number. The levels are
    /// separated by periods.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tydi::physical::Complexity;
    ///
    /// let c = Complexity::new(vec![3, 14])?;
    /// assert_eq!(c.to_string(), "3.14");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = String::new();
        let mut level = self.level.iter().map(|x| x.to_string());
        if let Some(x) = level.next() {
            result.push_str(&x);
            level.for_each(|x| {
                result.push('.');
                result.push_str(&x);
            });
        }
        write!(f, "{}", result)
    }
}
