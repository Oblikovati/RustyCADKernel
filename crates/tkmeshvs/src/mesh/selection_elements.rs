// SPDX-License-Identifier: GPL-2.0-only
//! The group and element modes of `MeshVS_Mesh::ComputeSelection`.
//!
//! Native quirks kept: a face of more than four nodes is a `MeshVS_SensitiveFace` of the
//! whole `aMaxFaceNodes` point buffer (its points beyond the face are the stale points of
//! earlier entities), and the custom entity of the highlighter receives the element id as
//! its selection mode.
use super::selection::SelectionScope;
use super::{MeshVSMeshMembers, MeshVSMeshSource, MeshVSOwnerMap};
use crate::MeshVSEntityType;
use crate::data_source::{MeshVSPackedMap, MeshVSTopologyHandle};
use crate::flags::*;
use crate::mesh_entity_owner::MeshVSMeshEntityOwner;
use crate::sensitive_polyhedron::MeshVSSensitivePolyhedron;
use crate::sensitive_quad::MeshVSSensitiveQuad;
use crate::sensitive_simple::{MeshVSSensitiveFace, MeshVSSensitiveSegment};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkv3d::select_mgr::SelectMgrEntityOwnerSource;
use rustycad_tkv3d::select3d::{
    Select3DSensitiveEntity, Select3DSensitiveTriangle, Select3DTypeOfSensitivity,
};

/// The owner priority of an entity type (`MeshVS_ET_Node` only for groups).
fn priority(kind: MeshVSEntityType, with_node: bool) -> i32 {
    match kind {
        MESHVS_ET_VOLUME => 1,
        MESHVS_ET_FACE => 2,
        MESHVS_ET_LINK => 3,
        MESHVS_ET_0D => 4,
        MESHVS_ET_NODE if with_node => 5,
        _ => 0,
    }
}

/// `MeshVS_SMF_Group`: an owner per group of the data source, with the entities of its
/// selectable members (or a dummy entity without any).
pub(super) fn add_group_entities<O: MeshVSMeshSource + ?Sized>(
    scope: &mut SelectionScope<'_, O>,
) -> Result<(), Failure> {
    scope
        .object
        .mesh_base()
        .fields
        .write()
        .group_owners
        .clear(true);
    let mut groups = MeshVSPackedMap::default();
    scope.source.get_all_groups(&mut groups);
    for group in groups.iter() {
        let (mut group_kind, mut members) = (MESHVS_ET_NONE, MeshVSPackedMap::default());
        if !scope.source.get_group(group, &mut group_kind, &mut members) {
            continue;
        }
        let address = scope.source.get_group_addr(group);
        let owner: Handle<dyn SelectMgrEntityOwnerSource> = MeshVSMeshEntityOwner::new(
            &scope.selectable(),
            group,
            address,
            group_kind,
            priority(group_kind, true),
            true,
        )
        .cast();
        scope
            .object
            .mesh_base()
            .fields
            .write()
            .group_owners
            .bind(group, owner.clone());
        let mut added = false;
        for key in members.iter() {
            added |= add_group_member(scope, &owner, group_kind, key)?;
        }
        if !added {
            scope.add_dummy(&owner)?;
        }
    }
    Ok(())
}

fn add_group_member<O: MeshVSMeshSource + ?Sized>(
    scope: &mut SelectionScope<'_, O>,
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    group_kind: MeshVSEntityType,
    key: i32,
) -> Result<bool, Failure> {
    let (mut nb_nodes, mut kind) = (0, MESHVS_ET_NONE);
    if group_kind == MESHVS_ET_NODE {
        if scope
            .source
            .get_geom(key, false, &mut scope.coords, &mut nb_nodes, &mut kind)
            && scope.object.is_selectable_node(key)
        {
            scope.add_point(owner)?;
            return Ok(true);
        }
        return Ok(false);
    }
    if !scope.source.get_geom_type(key, true, &mut kind) || !scope.object.is_selectable_elem(key) {
        return Ok(false);
    }
    scope
        .source
        .get_geom(key, true, &mut scope.coords, &mut nb_nodes, &mut kind);
    add_element_entity(scope, owner, kind, key, nb_nodes)
}

/// The entity of a selectable element: a triangle, quad or face, a segment, a polyhedron,
/// else the custom entity of the highlighter; whether one was added.
fn add_element_entity<O: MeshVSMeshSource + ?Sized>(
    scope: &SelectionScope<'_, O>,
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    kind: MeshVSEntityType,
    key: i32,
    mut nb_nodes: i32,
) -> Result<bool, Failure> {
    let mut topo = MeshVSTopologyHandle::default();
    let entity: Handle<dyn Select3DSensitiveEntity> = if kind == MESHVS_ET_FACE && nb_nodes > 0 {
        face_entity(scope, owner, nb_nodes)?
    } else if kind == MESHVS_ET_LINK && nb_nodes > 0 {
        Handle::new(MeshVSSensitiveSegment::new(
            owner,
            scope.point(1),
            scope.point(2),
        ))
        .cast()
    } else if kind == MESHVS_ET_VOLUME && scope.source.get_3d_geom(key, &mut nb_nodes, &mut topo) {
        Handle::new(MeshVSSensitivePolyhedron::new(
            owner,
            &scope.points(),
            1,
            &topo,
        )?)
        .cast()
    } else {
        let hilighter = scope.hilighter.get().expect("highlighter");
        let custom = hilighter.custom_sensitive_entity(owner, key);
        if custom.is_null() {
            return Ok(false);
        }
        custom
    };
    scope.add(entity)?;
    Ok(true)
}

fn face_entity<O: MeshVSMeshSource + ?Sized>(
    scope: &SelectionScope<'_, O>,
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    nb_nodes: i32,
) -> Result<Handle<dyn Select3DSensitiveEntity>, Failure> {
    let interior = Select3DTypeOfSensitivity::Interior;
    Ok(match nb_nodes {
        3 => Handle::new(Select3DSensitiveTriangle::new(
            owner,
            scope.point(1),
            scope.point(2),
            scope.point(3),
            interior,
        ))
        .cast(),
        4 => Handle::new(MeshVSSensitiveQuad::from_points(owner, &scope.points())?).cast(),
        _ => Handle::new(MeshVSSensitiveFace::new(owner, &scope.points(), interior)?).cast(),
    })
}

/// The element modes: an owner per element of the mode type, in the 0D, link, face or volume
/// map, with its entity when selectable, else a dummy entity.
pub(super) fn add_element_entities<O: MeshVSMeshSource + ?Sized>(
    scope: &mut SelectionScope<'_, O>,
    mode: i32,
) -> Result<(), Failure> {
    {
        let mut fields = scope.object.mesh_base().fields.write();
        fields.element_owners.clear(true);
        owner_map(&mut fields, mode).clear(true);
    }
    let elements: Vec<i32> = scope.all_elements.iter().collect();
    for key in elements {
        let mut kind = MESHVS_ET_NONE;
        if !scope.source.get_geom_type(key, true, &mut kind) || mode != kind.0 {
            continue;
        }
        let mut nb_nodes = 0;
        scope
            .source
            .get_geom(key, true, &mut scope.coords, &mut nb_nodes, &mut kind);
        let address = scope.source.get_addr(key, true);
        let owner: Handle<dyn SelectMgrEntityOwnerSource> = MeshVSMeshEntityOwner::new(
            &scope.selectable(),
            key,
            address,
            kind,
            priority(kind, false),
            false,
        )
        .cast();
        owner_map(&mut scope.object.mesh_base().fields.write(), mode).bind(key, owner.clone());
        if !scope.object.is_selectable_elem(key) {
            scope.add_dummy(&owner)?;
            continue;
        }
        add_element_entity(scope, &owner, kind, key, nb_nodes)?;
    }
    Ok(())
}

/// The owner map of an element mode (`my0DOwners` for any other mode).
fn owner_map(fields: &mut super::MeshVSMeshFields, mode: i32) -> &mut MeshVSOwnerMap {
    match MeshVSEntityType(mode) {
        MESHVS_ET_LINK => &mut fields.link_owners,
        MESHVS_ET_FACE => &mut fields.face_owners,
        MESHVS_ET_VOLUME => &mut fields.volume_owners,
        _ => &mut fields.owners_0d,
    }
}
