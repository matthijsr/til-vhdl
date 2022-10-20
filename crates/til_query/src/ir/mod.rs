use std::{sync::{Arc, Mutex}, path::PathBuf};

use tydi_common::{
    error::{Error, Result, WrapError},
    map::InsertionOrderedMap,
    name::PathName,
    traits::{Identify, Reverse},
};

use tydi_intern::Id;

use crate::{
    common::{
        logical::{
            logicaltype::{
                group::Group,
                stream::{Stream, Synchronicity},
                union::Union,
                IsNull, LogicalType,
            },
            split_streams::{SplitStreams, SplitsStreams},
        },
        stream_direction::StreamDirection,
    },
    ir::traits::InternSelf,
};

use self::{
    implementation::Implementation, interface_port::InterfacePort, interner::Interner,
    project::{Project, namespace::Namespace}, streamlet::Streamlet, traits::GetSelf,
};

pub mod annotation_keys;
pub mod connection;
pub mod db;
pub mod get_self;
pub mod implementation;
pub mod interface_port;
pub mod intern_self;
pub mod interner;
pub mod physical_properties;
pub mod project;
pub mod streamlet;
pub mod traits;

#[salsa::query_group(IrStorage)]
pub trait Ir: Interner {
    #[salsa::input]
    fn annotation(&self, intern_id: salsa::InternId, key: String) -> String;

    #[salsa::input]
    fn project(&self) -> Arc<Mutex<Project>>;

    fn all_streamlets(&self) -> Arc<Vec<Arc<Streamlet>>>;

    fn logical_type_split_streams(&self, key: Id<LogicalType>) -> Result<SplitStreams>;

    fn stream_split_streams(&self, key: Id<Stream>) -> Result<SplitStreams>;

    fn project_identifier(&self) -> String;

    fn project_output_path(&self) -> Option<PathBuf>;

    fn add_namespace(&self, namespace: Namespace) -> Result<()>;
}

fn project_identifier(db: &dyn Ir) -> String {
    let project = db.project();
    let project = project.lock().unwrap();
    project.identifier()
}

fn project_output_path(db: &dyn Ir) -> Option<PathBuf> {
    let project = db.project();
    let project = project.lock().unwrap();
    project.output_path().clone()
}

fn add_namespace(db: &dyn Ir, namespace: Namespace) -> Result<()> {
    db.project().lock().unwrap().add_namespace(db, namespace)
}

fn all_streamlets(db: &dyn Ir) -> Arc<Vec<Arc<Streamlet>>> {
    let project = db.project();
    let project = project.lock().unwrap();

    Arc::new(
        project
            .namespaces()
            .iter()
            .map(|(_, id)| id.get(db))
            .map(|namespace| {
                namespace
                    .streamlets(db)
                    .into_iter()
                    .map(|(_, streamlet)| streamlet)
                    .collect::<Vec<Arc<Streamlet>>>()
            })
            .flatten()
            .collect(),
    )
}

fn logical_type_split_streams(db: &dyn Ir, key: Id<LogicalType>) -> Result<SplitStreams> {
    fn split_fields(
        db: &dyn Ir,
        fields: &InsertionOrderedMap<PathName, Id<LogicalType>>,
    ) -> Result<(
        InsertionOrderedMap<PathName, Id<LogicalType>>,
        InsertionOrderedMap<PathName, Id<Stream>>,
    )> {
        let mut signals = InsertionOrderedMap::new();
        for (name, id) in fields.iter() {
            signals.try_insert(name.clone(), id.split_streams(db)?.signals())?;
        }
        let mut signals = InsertionOrderedMap::new();
        let mut streams = InsertionOrderedMap::new();
        for (name, id) in fields.iter() {
            let field_split = id.split_streams(db)?;
            signals.try_insert(name.clone(), field_split.signals())?;

            for (stream_name, stream_id) in field_split.streams() {
                streams.try_insert(name.with_children(stream_name.clone()), *stream_id)?;
            }
        }
        Ok((signals, streams))
    }

    Ok(match key.get(db) {
        LogicalType::Null | LogicalType::Bits(_) => {
            SplitStreams::new(key.clone(), InsertionOrderedMap::new())
        }
        LogicalType::Group(group) => {
            let (fields, streams) = split_fields(db, group.field_ids())?;
            SplitStreams::new(LogicalType::from(Group::new(fields)).intern(db), streams)
        }
        LogicalType::Union(union) => {
            let (fields, streams) = split_fields(db, union.field_ids())?;
            SplitStreams::new(LogicalType::from(Union::new(fields)).intern(db), streams)
        }
        LogicalType::Stream(stream_id) => stream_id.split_streams(db)?,
    })
}

fn stream_split_streams(db: &dyn Ir, key: Id<Stream>) -> Result<SplitStreams> {
    let this_stream = key.get(db);
    let split = this_stream.data_id().split_streams(db)?;
    let mut streams = InsertionOrderedMap::new();
    let (element, rest) = (split.signals(), split.streams());
    if this_stream.keep() || !element.is_null(db) || !this_stream.user_id().is_null(db) {
        streams.try_insert(
            PathName::new_empty(),
            Stream::new(
                element,
                this_stream.throughput(),
                this_stream.dimensionality(),
                this_stream.synchronicity(),
                this_stream.complexity().clone(),
                this_stream.direction(),
                this_stream.user_id(),
                this_stream.keep(),
            )
            .intern(db),
        )?;
    }

    for (name, stream_id) in rest.into_iter() {
        let mut stream = stream_id.get(db);
        if this_stream.direction() == StreamDirection::Reverse {
            stream.reverse();
        }
        if this_stream.flattens() {
            stream.set_synchronicity(Synchronicity::FlatDesync);
        } else {
            stream.set_dimensionality(stream.dimensionality() + this_stream.dimensionality());
        }
        stream.set_throughput(stream.throughput() * this_stream.throughput());

        streams.try_insert(name.clone(), stream.intern(db)).wrap_err(Error::InvalidArgument(
                r#"An error occurred during the SplitStreams function due to overlapping Stream names.
This is usually because a Stream contains another Stream as its Data type, and the Streams cannot be flattened.
You must ensure that only one Stream has a Keep and/or User property."#.to_string()))?;
    }

    Ok(SplitStreams::new(
        db.intern_type(LogicalType::Null),
        streams,
    ))
}

#[cfg(test)]
mod tests {
    use crate::common::logical::logicaltype::stream::Synchronicity;
    use crate::ir::db::Database;

    use super::physical_properties::InterfaceDirection;
    use super::project::namespace::Namespace;
    use super::*;
    use tydi_common::error::Result;

    // Want to make sure interning works as I expect it to (identical objects get same ID)
    #[test]
    fn verify_intern_id() -> Result<()> {
        let _db = Database::default();
        let db = &_db;
        let id1 = db.intern_type(LogicalType::try_new_bits(8)?);
        let id2 = db.intern_type(LogicalType::try_new_bits(8)?);
        assert_eq!(id1, id2);
        Ok(())
    }

    #[test]
    fn get_all_streamlets() -> Result<()> {
        let mut _db = Database::default();
        let db = &mut _db;
        let mut project = Project::new("proj", ".", None::<&str>)?;
        let mut namespace = Namespace::new("root.sub")?;
        namespace.define_type(db, "bits", 4)?;
        namespace.define_type(db, "null", LogicalType::Null)?;
        namespace.define_type(
            db,
            "stream",
            Stream::try_new(
                db,
                namespace.get_type_id("bits")?,
                1.0,
                1,
                Synchronicity::Sync,
                4,
                StreamDirection::Forward,
                namespace.get_type_id("null")?,
                false,
            )?,
        )?;
        namespace.define_streamlet(
            db,
            "streamlet",
            Streamlet::new().with_ports(
                db,
                vec![(
                    "a",
                    namespace.get_stream_id(db, "stream")?,
                    InterfaceDirection::In,
                )],
            )?,
        )?;
        namespace.define_streamlet(
            db,
            "implemented_streamlet",
            namespace
                .get_streamlet(db, "streamlet")?
                .as_ref()
                .clone()
                .with_implementation(None),
        )?;
        project.add_namespace(db, namespace)?;
        db.set_project(Arc::new(Mutex::new(project)));

        assert_eq!(db.all_streamlets().len(), 2);

        Ok(())
    }
}
