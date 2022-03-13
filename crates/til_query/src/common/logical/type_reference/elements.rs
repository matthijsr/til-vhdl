use super::ElementManipulatingReference;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StreamElementReference<F: Clone + PartialEq> {
    pub element: ElementManipulatingReference<F>,
    pub last: Option<F>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// The signals which indicate whether data is being transfered on a given
/// element lane.
pub struct ElementActivityReference<F: Clone + PartialEq> {
    pub strb: Option<F>,
    pub stai: Option<F>,
    pub endi: Option<F>,
}
