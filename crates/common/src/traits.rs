//! Tydi common traits.

/// In-place reverse.
///
/// # Implementing `Reverse`
///
/// ```rust
/// use tydi_common::traits::{Reverse, Reversed};
///
/// #[derive(Clone, Copy, Debug, PartialEq)]
/// struct A {
///     in_port: bool,
///     size: u32,
/// }
///
/// impl Reverse for A {
///     fn reverse(&mut self) {
///         self.in_port = !self.in_port;
///     }
/// }
///
/// let mut a = A { in_port: false, size: 42 };
/// a.reverse();
/// assert!(a.in_port);
/// a.reverse();
/// assert!(!a.in_port);
/// let b = A { in_port: true, size: 42 };
/// assert_eq!(a.reversed(), b);
/// ```
pub trait Reverse {
    /// Reverse in-place.
    fn reverse(&mut self);
}

/// Construct reversed items.
pub trait Reversed {
    /// Returns a new reversed instance.
    fn reversed(&self) -> Self;
}

impl<T> Reversed for T
where
    T: Reverse + Clone,
{
    /// Returns a new reversed instance by cloning and reversing the clone
    /// in-place.
    fn reversed(&self) -> T {
        let mut r = self.clone();
        r.reverse();
        r
    }
}

/// Trait for things that have names.
pub trait Identify {
    fn identifier(&self) -> String;
}

/// Trait for things that have documentation.
pub trait Document {
    /// Return optionally existing user-written documentation of self.
    fn doc(&self) -> Option<&String>;
}

/// Trait for things that have arbitrary, mutable documentation.
pub trait Documents: Document + Sized {
    /// Set the documentation.
    fn set_doc(&mut self, doc: impl Into<String>);

    /// Set the documentation and return self, for natural/factory patterns.
    fn with_doc(mut self, doc: impl Into<String>) -> Self {
        self.set_doc(doc);
        self
    }
}
