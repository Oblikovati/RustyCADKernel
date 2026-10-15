// SPDX-License-Identifier: GPL-2.0-only
//! The selectable node map, the owner maps and the shared face nodes of `MeshVS_Mesh`.
use super::members::{add_to_map, copy_map};
use super::{MeshVSMeshMembers, MeshVSMeshSource, MeshVSOwnerMap};
use crate::MeshVSDrawerAttribute as Key;
use crate::data_source::normals::coords_buffer;
use crate::data_source::{MeshVSDataSource, MeshVSPackedMap};
use crate::flags::{MESHVS_ET_FACE, MESHVS_ET_NONE};
use rustycad_tkernel::ncollection::{Array1, packed_map_algo};
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::tcol_std::HPackedMapOfInteger;

/// `MeshVS_DA_MaxFaceNodes` of the current drawer when positive.
pub(super) fn max_face_nodes<O: MeshVSMeshSource + ?Sized>(object: &O) -> Option<i32> {
    let drawer = object.get_drawer();
    let drawer = drawer.get()?;
    let mut max_nodes = 0;
    let found = drawer.get_integer(Key::MaxFaceNodes as i32, &mut max_nodes);
    (found && max_nodes > 0).then_some(max_nodes)
}

/// Runs `UpdateSelectableNodes` when the drawer sets `MeshVS_DA_SelectableAuto`.
pub(super) fn auto_update_selectable<O: MeshVSMeshSource + ?Sized>(object: &O) {
    let drawer = object.get_drawer();
    let mut auto = false;
    if drawer
        .get()
        .is_some_and(|d| d.get_boolean(Key::SelectableAuto as i32, &mut auto))
        && auto
    {
        object.update_selectable_nodes();
    }
}

/// `UpdateSelectableNodes`: a new map (kept even when nothing can fill it), the visible
/// nodes (none without hidden map), then the nodes of the visible elements.
pub(super) fn update_selectable_nodes<O: MeshVSMeshSource + ?Sized>(object: &O) {
    let selectable = Handle::new(HPackedMapOfInteger::default());
    object.set_selectable_nodes(&selectable);
    let source = object.get_data_source();
    let (Some(source), Some(max_nodes)) = (source.get(), max_face_nodes(object)) else {
        return;
    };
    let hidden = object.get_hidden_nodes();
    if let Some(hidden) = hidden.get() {
        let all_nodes = source.get_all_nodes();
        packed_map_algo::subtraction(&mut selectable.change_map(), &all_nodes, &hidden.map());
    }
    for key in source.get_all_elements().iter() {
        if object.is_hidden_elem(key) {
            continue;
        }
        let Ok(mut nodes) = Array1::<i32>::with_bounds(1, max_nodes) else {
            continue;
        };
        let mut nb_nodes = 0;
        if !source.get_nodes_by_element(key, &mut nodes, &mut nb_nodes) {
            continue;
        }
        let mut map = selectable.change_map();
        for i in 1..=nb_nodes {
            if let Ok(node) = nodes.value(i) {
                map.add(*node);
            }
        }
    }
}

/// `GetOwnerMaps`: see [`MeshVSMeshMembers::get_owner_maps`].
pub(super) fn owner_maps<O: MeshVSMeshSource + ?Sized>(
    object: &O,
    is_element: bool,
) -> MeshVSOwnerMap {
    let source = object.get_data_source();
    let advanced = source
        .get()
        .is_some_and(|s| s.is_advanced_selection_enabled());
    let mut fields = object.mesh_base().fields.write();
    if advanced {
        return copy_map(&fields.mesh_owners);
    }
    if !is_element {
        return copy_map(&fields.node_owners);
    }
    if fields.element_owners.is_empty() {
        let fields = &mut *fields;
        add_to_map(&mut fields.element_owners, &fields.owners_0d);
        add_to_map(&mut fields.element_owners, &fields.link_owners);
        add_to_map(&mut fields.element_owners, &fields.face_owners);
        add_to_map(&mut fields.element_owners, &fields.volume_owners);
    }
    copy_map(&fields.element_owners)
}

/// The protected `scanFacesForSharedNodes`: the nodes of every selectable face.
/// Example: `scan_faces_for_shared_nodes(&*mesh, &elements, 10, &mut shared)`.
pub fn scan_faces_for_shared_nodes<O: MeshVSMeshSource + ?Sized>(
    object: &O,
    all_elements: &MeshVSPackedMap,
    nb_max_face_nodes: i32,
    shared_nodes: &mut MeshVSPackedMap,
) {
    shared_nodes.clear();
    let source = object.get_data_source();
    let Some(source) = source.get() else {
        return;
    };
    let Ok(mut coords) = coords_buffer(nb_max_face_nodes) else {
        return;
    };
    for face in all_elements.iter() {
        let mut kind = MESHVS_ET_NONE;
        if !object.is_selectable_elem(face)
            || !source.get_geom_type(face, true, &mut kind)
            || kind != MESHVS_ET_FACE
        {
            continue;
        }
        add_face_nodes(source, face, &mut coords, shared_nodes);
    }
}

fn add_face_nodes(
    source: &dyn MeshVSDataSource,
    face: i32,
    coords: &mut Array1<'_, f64>,
    shared_nodes: &mut MeshVSPackedMap,
) {
    let (mut nb_nodes, mut kind) = (0, MESHVS_ET_NONE);
    // The native result of GetGeom is ignored; only its node count is read.
    source.get_geom(face, true, coords, &mut nb_nodes, &mut kind);
    if nb_nodes == 0 {
        return;
    }
    let Ok(mut nodes) = Array1::<i32>::with_bounds(1, nb_nodes) else {
        return;
    };
    if !source.get_nodes_by_element(face, &mut nodes, &mut nb_nodes) {
        return;
    }
    for i in 1..=nb_nodes {
        if let Ok(node) = nodes.value(i) {
            shared_nodes.add(*node);
        }
    }
}
