use core::fmt;
use std::{convert::TryFrom, sync::Arc};

use crate::{
    common::physical::fields::Fields,
    ir::{
        traits::{GetSelf, InternSelf, MoveDb},
        Ir,
    },
};
use indexmap::IndexMap;
use tydi_common::{
    error::{Error, Result, TryResult},
    name::{Name, PathName},
    numbers::{BitCount, NonNegative, Positive},
};

pub mod bits;
pub mod group;
pub mod stream;

use tydi_intern::Id;

use self::{group::Group, stream::Stream, union::Union};

use super::split_streams::{SplitStreams, SplitsStreams};
pub mod union;

pub trait IsNull {
    fn is_null(&self, db: &dyn Ir) -> bool;
}

/// Types of logical streams.
///
/// This structure is at the heart of the logical stream specification. It is
/// used both to specify the type of a logical stream and internally for the
/// process of lowering the recursive structure down to physical streams and
/// signals.
///
/// The logical stream type is defined recursively by means of a number of
/// stream types. Two classes of stream types are defined: stream-manipulating
/// types, and element-manipulating types.
///
/// # Examples
///
/// ```rust
/// ```
///
/// [Reference](https://abs-tudelft.github.io/tydi/specification/logical.html#logical-stream-type)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LogicalType {
    /// The Null stream type indicates the transferrence of one-valued data: it
    /// is only valid value is ∅ (null).
    ///
    /// [Reference](https://abs-tudelft.github.io/tydi/specification/logical.html#null)
    Null,
    /// The Bits stream type, defined as `Bits(b)`, indicates the transferrence
    /// of `2^b`-valued data carried by means of a group of `b` bits, where`b`
    /// is a positive integer.
    ///
    /// [Reference](https://abs-tudelft.github.io/tydi/specification/logical.html#bits)
    Bits(Positive),
    /// The Group stream type acts as a product type (composition).
    ///
    /// [Reference](https://abs-tudelft.github.io/tydi/specification/logical.html#group)
    Group(Group),
    /// The Union stream type acts as a sum type (exclusive disjunction).
    ///
    /// [Reference](https://abs-tudelft.github.io/tydi/specification/logical.html#union)
    Union(Union),
    /// The Stream type is used to define a new physical stream.
    ///
    /// [Reference](https://abs-tudelft.github.io/tydi/specification/logical.html#stream)
    Stream(Id<Stream>),
}

impl LogicalType {
    /// Returns a new Bits stream type with the provided bit count as number of
    /// bits. Returns an error when the bit count is zero.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tydi_common::error::Error;
    /// use tydi_common::numbers::Positive;
    /// use til_query::common::logical::logicaltype::LogicalType;
    ///
    /// let bits = LogicalType::try_new_bits(4);
    /// let zero = LogicalType::try_new_bits(0);
    ///
    /// assert_eq!(bits, Ok(LogicalType::Bits(Positive::new(4).unwrap())));
    /// assert_eq!(zero, Err(Error::InvalidArgument("bit count cannot be zero".to_string())));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn try_new_bits(bit_count: NonNegative) -> Result<Self> {
        Ok(LogicalType::Bits(Positive::new(bit_count).ok_or_else(
            || Error::InvalidArgument("bit count cannot be zero".to_string()),
        )?))
    }

    /// Returns a new Group stream type from the provided iterator of names and
    /// stream types. Returns an error when the values cannot be converted into
    /// valid names, or valid logical stream types as required by [`Group`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tydi_common::error::Error;
    /// use til_query::common::logical::logicaltype::LogicalType;
    /// use til_query::ir::{db::Database, Ir, interner::Interner};
    ///
    /// let db = Database::default();
    ///
    /// let a = db.intern_type(LogicalType::try_new_bits(4)?);
    /// let b = db.intern_type(LogicalType::try_new_bits(12)?);
    ///
    /// let group = LogicalType::try_new_group(
    ///     None,
    ///     vec![
    ///         ("a", a),
    ///         ("b", b),
    ///     ]
    /// )?;
    ///
    /// assert!(match group {
    ///     LogicalType::Group(_) => true,
    ///     _ => false,
    /// });
    ///
    /// assert_eq!(
    ///     LogicalType::try_new_group(None, vec![("1badname", a)]),
    ///     Err(Error::InvalidArgument("1badname: name cannot start with a digit".to_string()))
    /// );
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// [`Group`]: ./struct.Group.html
    pub fn try_new_group(
        parent_id: Option<PathName>,
        group: impl IntoIterator<Item = (impl TryResult<Name>, Id<LogicalType>)>,
    ) -> Result<Self> {
        Group::try_new(parent_id, group).map(Into::into)
    }

    pub fn try_new_union(
        parent_id: Option<PathName>,
        union: impl IntoIterator<Item = (impl TryResult<Name>, Id<LogicalType>)>,
    ) -> Result<Self> {
        Union::try_new(parent_id, union).map(Into::into)
    }

    /// Returns true if this logical type consists of only element-
    /// manipulating nodes. This recursively checks all inner logical
    /// types.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use til_query::common::logical::logicaltype::LogicalType;
    /// use til_query::ir::db::Database;
    ///
    /// let db = Database::default();
    ///
    /// assert!(LogicalType::Null.is_element_only(&db));
    /// assert!(LogicalType::try_new_bits(3)?.is_element_only(&db));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn is_element_only(&self, db: &dyn Ir) -> bool {
        match self {
            LogicalType::Null | LogicalType::Bits(_) => true,
            LogicalType::Group(group) => group
                .fields(db)
                .iter()
                .all(|(_, typ)| typ.is_element_only(db)),
            LogicalType::Union(union) => union
                .fields(db)
                .iter()
                .all(|(_, typ)| typ.is_element_only(db)),
            LogicalType::Stream(_) => false,
        }
    }

    /// Flattens a logical stream type consisting of Null, Bits, Group and
    /// Union stream types into a [`Fields`].
    ///
    /// [Reference](https://abs-tudelft.github.io/tydi/specification/logical.html#field-conversion-function)
    ///
    /// [`Fields`]: ./struct.Fields.html
    pub(crate) fn fields(&self, db: &dyn Ir) -> Fields<BitCount> {
        let mut fields = Fields::new_empty();
        match self {
            LogicalType::Null | LogicalType::Stream(_) => fields,
            LogicalType::Bits(b) => {
                fields.insert(PathName::new_empty(), *b).unwrap();
                fields
            }
            LogicalType::Group(group) => {
                for (name, typ) in group.ordered_fields(db).iter() {
                    typ.fields(db).iter().for_each(|(path_name, bit_count)| {
                        fields
                            .insert(path_name.with_parents(name.clone()), *bit_count)
                            .unwrap();
                    })
                }
                fields
            }
            LogicalType::Union(union) => {
                if let Some(tag) = union.tag() {
                    fields
                        .insert(PathName::try_new(vec!["tag"]).unwrap(), tag)
                        .unwrap();
                }
                let b = union.field_ids().iter().fold(0, |acc, (_, id)| {
                    acc.max(
                        id.get(db)
                            .fields(db)
                            .values()
                            .fold(0, |acc, count| acc.max(count.get())),
                    )
                });
                if b > 0 {
                    fields
                        .insert(
                            PathName::try_new(vec!["union"]).unwrap(),
                            BitCount::new(b).unwrap(),
                        )
                        .unwrap();
                }
                fields
            }
        }
    }

    pub fn null_id(db: &dyn Ir) -> Id<Self> {
        LogicalType::Null.intern(db)
    }
}

impl SplitsStreams for Id<LogicalType> {
    fn split_streams(&self, db: &dyn Ir) -> Result<SplitStreams> {
        fn split_fields(
            db: &dyn Ir,
            fields: Arc<IndexMap<PathName, Id<LogicalType>>>,
        ) -> Result<(
            IndexMap<PathName, Id<LogicalType>>,
            IndexMap<PathName, Id<Stream>>,
        )> {
            let mut signals = IndexMap::new();
            for (name, id) in fields.iter() {
                signals.insert(name.clone(), id.split_streams(db)?.signals());
            }
            let mut signals = IndexMap::new();
            let mut streams = IndexMap::new();
            for (name, id) in fields.iter() {
                let field_split = id.split_streams(db)?;
                signals.insert(name.clone(), field_split.signals());

                for (stream_name, stream_id) in field_split.streams() {
                    streams.insert(name.with_children(stream_name.clone()), *stream_id);
                }
            }
            Ok((signals, streams))
        }

        Ok(match self.get(db) {
            LogicalType::Null | LogicalType::Bits(_) => {
                SplitStreams::new(self.clone(), IndexMap::new())
            }
            LogicalType::Group(group) => {
                let (fields, streams) = split_fields(db, group.ordered_field_ids())?;
                SplitStreams::new(LogicalType::from(Group::new(fields)).intern(db), streams)
            }
            LogicalType::Union(union) => {
                let (fields, streams) = split_fields(db, union.ordered_field_ids())?;
                SplitStreams::new(LogicalType::from(Union::new(fields)).intern(db), streams)
            }
            LogicalType::Stream(stream_id) => stream_id.split_streams(db)?,
        })
    }
}

impl IsNull for Id<LogicalType> {
    fn is_null(&self, db: &dyn Ir) -> bool {
        self.get(db).is_null(db)
    }
}

impl IsNull for LogicalType {
    /// Returns true if and only if this logical stream does not result in any
    /// signals.
    ///
    /// [Reference](https://abs-tudelft.github.io/tydi/specification/logical.html#null-detection-function)
    fn is_null(&self, db: &dyn Ir) -> bool {
        match self {
            LogicalType::Null => true,
            LogicalType::Group(group) => {
                group.field_ids().into_iter().all(|(_, id)| id.is_null(db))
            }
            LogicalType::Union(union) => {
                union.tag() == None && union.field_ids().into_iter().all(|(_, id)| id.is_null(db))
            }
            LogicalType::Stream(stream) => stream.is_null(db),
            LogicalType::Bits(_) => false,
        }
    }
}

impl MoveDb<Id<LogicalType>> for LogicalType {
    fn move_db(
        &self,
        original_db: &dyn Ir,
        target_db: &dyn Ir,
        prefix: &Option<Name>,
    ) -> Result<Id<Self>> {
        Ok(match self {
            LogicalType::Null => self.clone().intern(target_db),
            LogicalType::Bits(_) => self.clone().intern(target_db),
            LogicalType::Group(group) => group.move_db(original_db, target_db, prefix)?,
            LogicalType::Union(union) => union.move_db(original_db, target_db, prefix)?,
            LogicalType::Stream(stream) => {
                LogicalType::Stream(stream.move_db(original_db, target_db, prefix)?)
                    .intern(target_db)
            }
        })
    }
}

impl fmt::Display for LogicalType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogicalType::Null => write!(f, "Null"),
            LogicalType::Bits(b) => write!(f, "Bits({})", b),
            LogicalType::Group(group) => write!(f, "Group{}", group),
            LogicalType::Union(union) => write!(f, "Union{}", union),
            LogicalType::Stream(stream_id) => write!(f, "Stream(Id: {})", stream_id),
        }
    }
}

impl TryFrom<LogicalType> for Id<Stream> {
    type Error = Error;

    fn try_from(value: LogicalType) -> Result<Self> {
        match &value {
            LogicalType::Stream(id) => Ok(*id),
            _ => Err(Error::InvalidTarget(format!(
                "Type is not a Stream, but a {}",
                value
            ))),
        }
    }
}
