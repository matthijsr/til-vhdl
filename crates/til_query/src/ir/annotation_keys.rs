use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AnnotationKey {
    StreamletComponentName,
}

impl fmt::Display for AnnotationKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnnotationKey::StreamletComponentName => write!(f, "StreamletComponentName"),
        }
    }
}
