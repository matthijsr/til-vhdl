use std::sync::Arc;

use tydi_common::{
    error::{Error, Result, WrapError},
    map::InsertionOrderedMap,
    name::{Name, PathName},
    numbers::NonNegative,
    traits::Reverse,
};

use tydi_intern::Id;

use crate::{
    common::{
        logical::{
            logicaltype::{
                genericproperty::GenericProperty,
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
    generics::{interface::InterfaceGenericKind, GenericKind},
    implementation::{structure::streamlet_instance::GenericParameterAssignment, Implementation},
    interface_port::InterfacePort,
    interner::Interner,
    project::Project,
    streamlet::Streamlet,
    traits::GetSelf,
};

pub mod annotation_keys;
pub mod connection;
pub mod db;
pub mod generics;
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
    fn project(&self) -> Project;

    fn project_ref(&self) -> Arc<Project>;

    fn all_streamlets(&self) -> Arc<Vec<Arc<Streamlet>>>;

    fn logical_type_split_streams(&self, key: Id<LogicalType>) -> Result<SplitStreams>;

    fn stream_split_streams(&self, key: Id<Stream>) -> Result<SplitStreams>;

    fn logical_type_parameter_kinds(
        &self,
        key: Id<LogicalType>,
    ) -> Result<InsertionOrderedMap<Name, GenericKind>>;

    fn stream_parameter_kinds(
        &self,
        key: Id<Stream>,
    ) -> Result<InsertionOrderedMap<Name, GenericKind>>;

    fn stream_for_param_assignments(
        &self,
        key: Id<Stream>,
        param_assignments: InsertionOrderedMap<Name, GenericParameterAssignment>,
    ) -> Result<Id<Stream>>;

    fn type_for_param_assignments(
        &self,
        key: Id<LogicalType>,
        param_assignments: InsertionOrderedMap<Name, GenericParameterAssignment>,
    ) -> Result<Id<LogicalType>>;
}

fn project_ref(db: &dyn Ir) -> Arc<Project> {
    Arc::new(db.project())
}

fn try_add_param_kind(
    result: &mut InsertionOrderedMap<Name, GenericKind>,
    kind_name: Name,
    kind: GenericKind,
) -> Result<()> {
    if let Some(existing_kind) = result.get(&kind_name) {
        if &kind == existing_kind {
            Ok(())
        } else {
            Err(Error::ProjectError(format!(
                "Duplicate parameter name: \"{}\" is both a {} and a {}",
                &kind_name, existing_kind, &kind
            )))
        }
    } else {
        result.try_insert(kind_name, kind)
    }
}

fn stream_parameter_kinds(
    db: &dyn Ir,
    key: Id<Stream>,
) -> Result<InsertionOrderedMap<Name, GenericKind>> {
    let stream = db.lookup_intern_stream(key);
    let mut result = db.logical_type_parameter_kinds(stream.data_id())?;

    // For now, dimensionality is the only parameterized property, may need to
    // refactor this in the future.
    fn add_dim(
        result: &mut InsertionOrderedMap<Name, GenericKind>,
        prop: &GenericProperty<NonNegative>,
    ) -> Result<()> {
        match prop {
            GenericProperty::Combination(l, _, r) => {
                add_dim(result, l.as_ref())?;
                add_dim(result, r.as_ref())
            }
            GenericProperty::Fixed(_) => Ok(()),
            GenericProperty::Parameterized(n) => try_add_param_kind(
                result,
                n.clone(),
                InterfaceGenericKind::dimensionality().into(),
            ),
        }
    }

    add_dim(&mut result, stream.dimensionality())?;

    Ok(result)
}

fn logical_type_parameter_kinds(
    db: &dyn Ir,
    key: Id<LogicalType>,
) -> Result<InsertionOrderedMap<Name, GenericKind>> {
    fn try_add_params_for_fields(
        db: &dyn Ir,
        field_ids: &InsertionOrderedMap<PathName, Id<LogicalType>>,
        result: &mut InsertionOrderedMap<Name, GenericKind>,
    ) -> Result<()> {
        Ok(for (field_name, field_typ) in field_ids {
            let field_kinds = db.logical_type_parameter_kinds(*field_typ)?;
            for (field_kind_name, field_kind) in field_kinds.into_iter() {
                try_add_param_kind(result, field_kind_name, field_kind).map_err(|err| {
                    Error::ProjectError(format!("Issue with field {}: {}", field_name, err))
                })?;
            }
        })
    }

    let mut result = InsertionOrderedMap::new();
    let typ = db.lookup_intern_type(key);
    match typ {
        LogicalType::Null => (),
        // No generic parameter support for Bits yet.
        LogicalType::Bits(_) => (),
        LogicalType::Group(g) => {
            try_add_params_for_fields(db, g.field_ids(), &mut result)?;
        }
        LogicalType::Union(u) => {
            try_add_params_for_fields(db, u.field_ids(), &mut result)?;
        }
        LogicalType::Stream(s) => result = db.stream_parameter_kinds(s)?,
    }
    Ok(result)
}

fn all_streamlets(db: &dyn Ir) -> Arc<Vec<Arc<Streamlet>>> {
    let project = db.project_ref();

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
                this_stream.dimensionality().clone(),
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
            stream.set_dimensionality(
                stream.dimensionality().clone() + this_stream.dimensionality().clone(),
            );
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

fn type_for_param_assignments(
    db: &dyn Ir,
    key: Id<LogicalType>,
    param_assignments: InsertionOrderedMap<Name, GenericParameterAssignment>,
) -> Result<Id<LogicalType>> {
    let typ_params = db.logical_type_parameter_kinds(key)?;
    let mut to_assign = InsertionOrderedMap::new();
    for (param_name, param_assignment) in param_assignments {
        if typ_params.contains(&param_name) {
            to_assign.try_insert(param_name, param_assignment)?;
        }
    }
    if to_assign.len() > 0 {
        let typ = db.lookup_intern_type(key);
        match typ {
            LogicalType::Null => Err(Error::BackEndError(
                "Found a parameter to assign for Null, when none should exist".to_string(),
            )),
            LogicalType::Bits(_) => Err(Error::BackEndError(
                "Found a parameter to assign for Bits, when none should exist".to_string(),
            )),
            LogicalType::Group(g) => {
                let modified_fields = g.field_ids().clone().try_map_convert(|field_id| {
                    db.type_for_param_assignments(field_id, to_assign.clone())
                })?;
                Ok(db.intern_type(LogicalType::Group(Group::new(modified_fields))))
            }
            LogicalType::Union(u) => {
                let modified_fields = u.field_ids().clone().try_map_convert(|field_id| {
                    db.type_for_param_assignments(field_id, to_assign.clone())
                })?;
                Ok(db.intern_type(LogicalType::Union(Union::new(modified_fields))))
            }
            LogicalType::Stream(s) => Ok(db.intern_type(LogicalType::Stream(
                db.stream_for_param_assignments(s, to_assign)?,
            ))),
        }
    } else {
        Ok(key)
    }
}

fn stream_for_param_assignments(
    db: &dyn Ir,
    key: Id<Stream>,
    param_assignments: InsertionOrderedMap<Name, GenericParameterAssignment>,
) -> Result<Id<Stream>> {
    let stream_params = db.stream_parameter_kinds(key)?;
    let mut to_assign = vec![];
    for (param_name, param_assignment) in param_assignments {
        if stream_params.contains(&param_name) {
            to_assign.push((param_name, param_assignment));
        }
    }
    if to_assign.len() > 0 {
        let mut stream = db.lookup_intern_stream(key);
        for (param_name, param_assignment) in to_assign {
            stream.try_assign(&param_name, param_assignment)?;
        }
        Ok(db.intern_stream(stream))
    } else {
        Ok(key)
    }
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
        namespace.define_type_no_params(db, "bits", 4)?;
        namespace.define_type_no_params(db, "null", LogicalType::Null)?;
        namespace.define_type_no_params(
            db,
            "stream",
            Stream::try_new(
                db,
                namespace.get_type_id_no_assignments(db, "bits")?,
                1.0,
                1,
                Synchronicity::Sync,
                4,
                StreamDirection::Forward,
                namespace.get_type_id_no_assignments(db, "null")?,
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
                    namespace.get_stream_id_no_assignments(db, "stream")?,
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
        db.set_project(project);

        assert_eq!(db.all_streamlets().len(), 2);

        Ok(())
    }
}
