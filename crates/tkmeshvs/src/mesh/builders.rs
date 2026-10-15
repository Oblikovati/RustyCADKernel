// SPDX-License-Identifier: GPL-2.0-only
//! The builder sequence of `MeshVS_Mesh`, kept by decreasing priority, and its highlighter.
use super::MeshVSMeshBase;
use crate::data_source::MeshVSPackedMap;
use crate::prs_builder::{MeshVSPrsBuilder, MeshVSPrsBuilderMembers};
use rustycad_tkernel::standard::{Handle, RuntimeType};

/// A copy of the builder sequence, so no lock is held while a builder runs.
pub(super) fn builders_of(base: &MeshVSMeshBase) -> Vec<Handle<dyn MeshVSPrsBuilder>> {
    base.fields.read().builders.clone()
}

/// `GetBuilder`: one-based, null outside the sequence.
pub(super) fn builder_at(base: &MeshVSMeshBase, index: i32) -> Handle<dyn MeshVSPrsBuilder> {
    let builders = builders_of(base);
    if index < 1 || index as usize > builders.len() {
        return Handle::default();
    }
    builders[index as usize - 1].clone()
}

/// `GetBuilderById`: the first builder of the id (a null builder in the sequence is a native
/// null dereference; it is skipped here).
pub(super) fn builder_by_id(base: &MeshVSMeshBase, id: i32) -> Handle<dyn MeshVSPrsBuilder> {
    builders_of(base)
        .into_iter()
        .find(|b| b.get().is_some_and(|b| b.get_id() == id))
        .unwrap_or_default()
}

/// `GetFreeId`: the smallest non-negative id of no builder.
pub(super) fn free_id(base: &MeshVSMeshBase) -> i32 {
    let mut ids = MeshVSPackedMap::default();
    for builder in builders_of(base).iter().filter_map(Handle::get) {
        ids.add(builder.get_id());
    }
    let mut current = 0;
    while ids.contains(current) {
        current += 1;
    }
    current
}

/// `AddBuilder`: before the first builder of a strictly lower priority, else appended.
pub(super) fn add_builder(
    base: &MeshVSMeshBase,
    builder: &Handle<dyn MeshVSPrsBuilder>,
    treat_as_hilighter: bool,
) {
    let Some(added) = builder.get() else {
        return;
    };
    let priority = added.get_priority();
    let mut fields = base.fields.write();
    let position = fields
        .builders
        .iter()
        .position(|b| b.get().is_some_and(|b| b.get_priority() < priority))
        .unwrap_or(fields.builders.len());
    fields.builders.insert(position, builder.clone());
    if treat_as_hilighter {
        fields.hilighter = builder.clone();
    }
}

/// `RemoveBuilder`: removes a builder of a valid index, forgetting it as highlighter.
pub(super) fn remove_builder(base: &MeshVSMeshBase, index: i32) {
    let builder = builder_at(base, index);
    if builder.is_null() {
        return;
    }
    let mut fields = base.fields.write();
    if fields.hilighter.object_address() == builder.object_address() {
        fields.hilighter = Handle::default();
    }
    fields.builders.remove(index as usize - 1);
}

/// `RemoveBuilderById`: removes the first builder of an id.
pub(super) fn remove_builder_by_id(base: &MeshVSMeshBase, id: i32) {
    let builders = builders_of(base);
    let Some(position) = builders
        .iter()
        .position(|b| b.get().is_some_and(|b| b.get_id() == id))
    else {
        return;
    };
    let index = position as i32 + 1;
    let mut fields = base.fields.write();
    if fields.hilighter.object_address() == builders[position].object_address() {
        fields.hilighter = Handle::default();
    }
    drop(fields);
    remove_builder(base, index);
}

/// `FindBuilder(theType)`: the first builder of the kind.
pub(super) fn find_builder(
    base: &MeshVSMeshBase,
    kind: &RuntimeType,
) -> Handle<dyn MeshVSPrsBuilder> {
    builders_of(base)
        .into_iter()
        .find(|b| b.get().is_some_and(|b| b.is_kind(kind)))
        .unwrap_or_default()
}
