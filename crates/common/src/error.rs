//! Error variants.
use std::{
    convert::{Infallible, TryFrom},
    error, fmt, result,
};

use log::SetLoggerError;

/// Result type with [`Error`] variants.
///
/// [`Error`]: ./enum.Error.html
pub type Result<T> = result::Result<T, Error>;

/// Error variants used in this crate.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Error {
    /// Unknown error.
    UnknownError,
    /// Generic CLI error.
    CLIError(String),
    /// Indicates an invalid argument is provided.
    InvalidArgument(String),
    /// Indicates an unexpected duplicate is provided.
    UnexpectedDuplicate,
    /// File I/O error.
    FileIOError(String),
    /// Parsing error.
    ParsingError(String),
    /// Parsing error.
    ImplParsingError(LineErr),
    /// Invalid target.
    InvalidTarget(String),
    /// Back-end error.
    BackEndError(String),
    /// Forbidden interface name.
    InterfaceError(String),
    /// Project error
    ProjectError(String),
    /// Composer error
    ComposerError(String),
    /// Library error
    LibraryError(String),
}

///Error variants for implementation parser
#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct LineErr {
    pub line: usize,
    pub err: String,
}

impl LineErr {
    #[allow(dead_code)]
    pub fn new(l: usize, s: String) -> Self {
        LineErr { line: l, err: s }
    }
}

impl LineErr {
    pub fn on_line(self, n: usize) -> LineErr {
        LineErr {
            line: n,
            err: self.err,
        }
    }
}

impl fmt::Display for Error {
    /// Display the error variants.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::CLIError(ref msg) => write!(f, "CLI Error: {}", msg),
            Error::InvalidArgument(ref msg) => write!(f, "Invalid argument: {}", msg),
            Error::UnexpectedDuplicate => write!(f, "Unexpected duplicate"),
            Error::UnknownError => write!(f, "Unknown error"),
            Error::FileIOError(ref msg) => write!(f, "File I/O error: {}", msg),
            Error::ParsingError(ref msg) => write!(f, "Parsing error: {}", msg),
            Error::ImplParsingError(ref err) => write!(
                f,
                "Implementation parsing error on line: {}:{}",
                err.line, err.err
            ),
            Error::InvalidTarget(ref msg) => write!(f, "Invalid target: {}", msg),
            Error::BackEndError(ref msg) => write!(f, "Back-end error: {}", msg),
            Error::InterfaceError(ref msg) => write!(f, "Interface error: {}", msg),
            Error::ProjectError(ref msg) => write!(f, "Project error: {}", msg),
            Error::ComposerError(ref msg) => write!(f, "Composer error: {}", msg),
            Error::LibraryError(ref msg) => write!(f, "Library error: {}", msg),
        }
    }
}

impl error::Error for Error {}

impl From<Box<dyn error::Error>> for Error {
    fn from(error: Box<dyn error::Error>) -> Self {
        if let Ok(error) = error.downcast::<Self>() {
            *error
        } else {
            Error::UnknownError
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::FileIOError(e.to_string())
    }
}

impl From<SetLoggerError> for Error {
    fn from(e: SetLoggerError) -> Self {
        Error::CLIError(e.to_string())
    }
}

impl From<Infallible> for Error {
    fn from(_: Infallible) -> Self {
        // Infallible should never actually exist as an "error"
        unreachable!()
    }
}

pub trait ResultFrom<T>: Sized {
    fn result_from(value: T) -> Result<Self>;
}

pub trait TryResult<T> {
    fn try_result(self) -> Result<T>;
}

pub trait TryOptionalFrom<T>: Sized {
    fn optional_result_from(value: T) -> Result<Option<Self>>;
}

pub trait TryOptional<T> {
    fn try_optional(self) -> Result<Option<T>>;
}

impl<T, U, E> ResultFrom<U> for T
where
    Error: From<E>,
    T: TryFrom<U, Error = E>,
{
    fn result_from(value: U) -> Result<Self> {
        T::try_from(value).map_err(From::from)
    }
}

impl<T, U> TryResult<T> for U
where
    T: ResultFrom<U>,
{
    fn try_result(self) -> Result<T> {
        T::result_from(self)
    }
}

impl<T, U> TryOptionalFrom<Option<T>> for U
where
    U: ResultFrom<T>,
{
    fn optional_result_from(value: Option<T>) -> Result<Option<Self>> {
        match value {
            Some(some) => Ok(Some(U::result_from(some)?)),
            None => Ok(None),
        }
    }
}

impl<T, U> TryOptional<T> for U
where
    T: TryOptionalFrom<U>,
{
    fn try_optional(self) -> Result<Option<T>> {
        T::optional_result_from(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error() {
        let a = Error::InvalidArgument("test".to_string());
        let b = Error::UnexpectedDuplicate;
        assert_eq!(a.to_string(), "Invalid argument: test");
        assert_eq!(b.to_string(), "Unexpected duplicate");
    }
}
