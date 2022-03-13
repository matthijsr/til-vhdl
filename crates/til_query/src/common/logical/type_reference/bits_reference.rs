use tydi_common::numbers::Positive;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BitsReference<F: Clone + PartialEq> {
    pub bits: Positive,
    pub field: F,
}
