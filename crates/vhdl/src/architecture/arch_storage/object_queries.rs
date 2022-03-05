use super::interner::Interner;

#[salsa::query_group(ObjectStorage)]
pub trait ObjectQueries: Interner {}
