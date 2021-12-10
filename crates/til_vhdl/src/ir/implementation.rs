#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Implementation {
    None,
    Structural,
    Link,
}
