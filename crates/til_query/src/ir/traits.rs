use std::sync::Arc;

use super::Ir;
use tydi_common::error::TryResult;
use tydi_common::{error::Result, name::Name};
use tydi_intern::Id;

pub trait GetSelf<T> {
    fn get(&self, db: &dyn Ir) -> T;
}

pub trait InternSelf: Sized {
    fn intern(self, db: &dyn Ir) -> Id<Self>;
}

pub trait InternArc: Sized {
    fn intern_arc(self, db: &dyn Ir) -> Id<Arc<Self>>;
}

impl<T> InternArc for T
where
    Arc<T>: InternSelf,
{
    fn intern_arc(self, db: &dyn Ir) -> Id<Arc<Self>> {
        Arc::new(self).intern(db)
    }
}

pub trait InternAs<T> {
    fn intern_as(self, db: &dyn Ir) -> Id<T>;
}

pub trait TryIntern<T> {
    fn try_intern(self, db: &dyn Ir) -> Result<Id<T>>;
}

pub trait MoveDb<T>: Sized {
    /// Move (parts) of an object from one database to another.
    /// The prefix parameter can be used to prefix names with the name of the original project, to avoid conflicts.
    fn move_db(&self, original_db: &dyn Ir, target_db: &dyn Ir, prefix: &Option<Name>)
        -> Result<T>;
}

impl<T> MoveDb<Id<T>> for Id<T>
where
    Id<T>: GetSelf<T>,
    T: MoveDb<Id<T>>,
{
    fn move_db(
        &self,
        original_db: &dyn Ir,
        target_db: &dyn Ir,
        prefix: &Option<Name>,
    ) -> Result<Id<T>> {
        self.get(original_db)
            .move_db(original_db, target_db, prefix)
    }
}

impl<T, U> TryIntern<T> for U
where
    U: TryResult<T>,
    T: InternSelf,
{
    fn try_intern(self, db: &dyn Ir) -> Result<Id<T>> {
        Ok(self.try_result()?.intern(db))
    }
}
