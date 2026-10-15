// SPDX-License-Identifier: GPL-2.0-only
//! `MeshVS_Mesh::ComputeSelection`: one mesh owner and entity with advanced selection, else
//! node owners, the whole-mesh owner, group owners or element owners by mode.
use super::compute::drawer_flag;
use super::visibility::max_face_nodes;
use super::{MeshVSMeshMembers, MeshVSMeshSource};
use crate::MeshVSDrawerAttribute as Key;
use crate::data_source::normals::coords_buffer;
use crate::data_source::{MeshVSDataSource, MeshVSPackedMap};
use crate::flags::{MESHVS_DMF_OCC_MASK, MESHVS_ET_NONE};
use crate::mesh_entity_owner::MeshVSMeshEntityOwner;
use crate::mesh_owner::{MeshVSMeshOwner, MeshVSMeshOwnerSource};
use crate::prs_builder::MeshVSPrsBuilder;
use crate::selection_policy::{MESHVS_SMF_GROUP, MESHVS_SMF_MESH, MESHVS_SMF_NODE};
use crate::sensitive_mesh::MeshVSSensitiveMesh;
use crate::sensitive_simple::MeshVSDummySensitiveEntity;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::Point3;
use rustycad_tkv3d::prs_mgr::PrsMgrPresentableObjectMembers;
use rustycad_tkv3d::select_mgr::{
    SelectMgrEntityOwnerSource, SelectMgrSelectableObject, SelectMgrSelection,
};
use rustycad_tkv3d::select3d::{Select3DSensitiveEntity, Select3DSensitivePoint};
use std::time::Instant;

/// The shared state of one `ComputeSelection` call.
pub(super) struct SelectionScope<'a, O: MeshVSMeshSource + ?Sized> {
    pub object: &'a O,
    pub source: &'a dyn MeshVSDataSource,
    pub selection: &'a Handle<SelectMgrSelection>,
    pub hilighter: Handle<dyn MeshVSPrsBuilder>,
    pub max_nodes: i32,
    /// The native coordinate buffer of `aMaxFaceNodes` points, reused (stale values
    /// included) by every entity.
    pub coords: Array1<'static, f64>,
    pub all_nodes: MeshVSPackedMap,
    pub all_elements: MeshVSPackedMap,
}

impl<O: MeshVSMeshSource + ?Sized> SelectionScope<'_, O> {
    /// The mesh as the selectable object of its owners.
    pub fn selectable(&self) -> Handle<dyn SelectMgrSelectableObject> {
        self.object.presentable_base().this().cast()
    }
    /// The point of index `index` (one-based) of the coordinate buffer (`aPntArray`).
    pub fn point(&self, index: i32) -> Point3 {
        let at = |i: i32| self.coords.value(i).copied().unwrap_or(0.0);
        Point3::new(at(3 * index - 2), at(3 * index - 1), at(3 * index))
    }
    /// The whole point buffer (`aPntArray`, every `aMaxFaceNodes` point).
    pub fn points(&self) -> Vec<Point3> {
        (1..=self.max_nodes).map(|i| self.point(i)).collect()
    }
    pub fn add(&self, entity: Handle<dyn Select3DSensitiveEntity>) -> Result<(), Failure> {
        self.selection.add(&entity)
    }
    pub fn add_dummy(&self, owner: &Handle<dyn SelectMgrEntityOwnerSource>) -> Result<(), Failure> {
        self.add(Handle::new(MeshVSDummySensitiveEntity::new(owner)).cast())
    }
    pub fn add_point(&self, owner: &Handle<dyn SelectMgrEntityOwnerSource>) -> Result<(), Failure> {
        self.add(Handle::new(Select3DSensitivePoint::new(owner, self.point(1))).cast())
    }
}

/// The native `MeshVS_Mesh::ComputeSelection` body; with `MeshVS_DA_ComputeSelectionTime`
/// (or without the key) the mode and the elapsed time are printed on standard output.
/// Example: `meshvs_mesh_compute_selection(&*mesh, &selection, MESHVS_SMF_NODE.0)?`.
pub fn meshvs_mesh_compute_selection<O: MeshVSMeshSource + ?Sized>(
    object: &O,
    selection: &Handle<SelectMgrSelection>,
    mode: i32,
) -> Result<(), Failure> {
    let show_time = drawer_flag(object, Key::ComputeSelectionTime as i32);
    let start = Instant::now();
    let source = object.get_data_source();
    let (Some(source), Some(max_nodes)) = (source.get(), max_face_nodes(object)) else {
        return Ok(());
    };
    let display = if object.has_display_mode() {
        object.display_mode()
    } else {
        object.default_display_mode()
    };
    let hilighter = object.get_hilighter();
    if hilighter.is_null() || (display & MESHVS_DMF_OCC_MASK) == 0 {
        return Ok(());
    }
    let mut scope = SelectionScope {
        object,
        source,
        selection,
        hilighter,
        max_nodes,
        coords: coords_buffer(max_nodes)?,
        all_nodes: source.get_all_nodes(),
        all_elements: source.get_all_elements(),
    };
    if source.is_advanced_selection_enabled() {
        add_mesh_owner_entity(&scope, mode)?;
    } else if mode == MESHVS_SMF_NODE.0 {
        add_node_entities(&mut scope)?;
    } else if mode == MESHVS_SMF_MESH.0 {
        super::selection_whole::add_whole_mesh_entities(&mut scope)?;
    } else if mode == MESHVS_SMF_GROUP.0 {
        super::selection_elements::add_group_entities(&mut scope)?;
    } else {
        super::selection_elements::add_element_entities(&mut scope, mode)?;
    }
    if show_time {
        let seconds = start.elapsed().as_secs_f64();
        println!("SelectionMode : {mode}");
        println!("Compute selection: {seconds} sec");
        println!("Compute selection CPU : {seconds} sec\n");
    }
    Ok(())
}

/// The advanced selection: the first mesh owner of the mesh owner map, else a new one of
/// priority 5 bound as owner 1, and one sensitive mesh of the mode.
fn add_mesh_owner_entity<O: MeshVSMeshSource + ?Sized>(
    scope: &SelectionScope<'_, O>,
    mode: i32,
) -> Result<(), Failure> {
    let existing = scope
        .object
        .get_owner_maps(false)
        .values()
        .map(|owner| owner.cast::<dyn MeshVSMeshOwnerSource>())
        .find(|owner| !owner.is_null());
    let owner: Handle<dyn SelectMgrEntityOwnerSource> = match existing {
        Some(owner) => owner.cast(),
        None => {
            MeshVSMeshOwner::new(&scope.selectable(), &scope.object.get_data_source(), 5).cast()
        }
    };
    {
        let mut fields = scope.object.mesh_base().fields.write();
        if !fields.mesh_owners.is_bound(&1) {
            fields.mesh_owners.bind(1, owner.clone());
        }
    }
    scope.add(Handle::new(MeshVSSensitiveMesh::new(&owner, mode)?).cast())
}

/// `MeshVS_SMF_Node`: an owner of priority 5 per node of known geometry, with a point when
/// the node is selectable, else a dummy entity.
fn add_node_entities<O: MeshVSMeshSource + ?Sized>(
    scope: &mut SelectionScope<'_, O>,
) -> Result<(), Failure> {
    scope
        .object
        .mesh_base()
        .fields
        .write()
        .node_owners
        .clear(true);
    let nodes = scope.all_nodes.clone();
    for key in nodes.iter() {
        let (mut nb_nodes, mut kind) = (0, MESHVS_ET_NONE);
        if !scope
            .source
            .get_geom(key, false, &mut scope.coords, &mut nb_nodes, &mut kind)
        {
            continue;
        }
        let address = scope.source.get_addr(key, false);
        let owner: Handle<dyn SelectMgrEntityOwnerSource> =
            MeshVSMeshEntityOwner::new(&scope.selectable(), key, address, kind, 5, false).cast();
        scope
            .object
            .mesh_base()
            .fields
            .write()
            .node_owners
            .bind(key, owner.clone());
        if scope.object.is_selectable_node(key) {
            scope.add_point(&owner)?;
        } else {
            scope.add_dummy(&owner)?;
        }
    }
    Ok(())
}
