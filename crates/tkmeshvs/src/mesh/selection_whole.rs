// SPDX-License-Identifier: GPL-2.0-only
//! The `MeshVS_SMF_Mesh` mode of `MeshVS_Mesh::ComputeSelection`: the whole-mesh owner with a
//! box, the common node entity, or the common face entity plus the free links and nodes.
use super::selection::SelectionScope;
use super::visibility::scan_faces_for_shared_nodes;
use super::{MeshVSMeshMembers, MeshVSMeshSource};
use crate::common_sensitive_entity::MeshVSCommonSensitiveEntity;
use crate::data_source::MeshVSPackedMap;
use crate::flags::{MESHVS_ET_LINK, MESHVS_ET_NONE};
use crate::selection_policy::MeshVSMeshSelectionMethod;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkv3d::select_mgr::{SelectMgrEntityOwner, SelectMgrEntityOwnerSource};
use rustycad_tkv3d::select3d::{
    Select3DSensitiveBox, Select3DSensitivePoint, Select3DSensitiveSegment,
};

/// The whole-mesh owner, created on first use with priority 0.
fn whole_mesh_owner<O: MeshVSMeshSource + ?Sized>(
    scope: &SelectionScope<'_, O>,
) -> Handle<dyn SelectMgrEntityOwnerSource> {
    let mut fields = scope.object.mesh_base().fields.write();
    if fields.whole_mesh_owner.is_null() {
        fields.whole_mesh_owner =
            SelectMgrEntityOwner::with_selectable(&scope.selectable(), 0).cast();
    }
    fields.whole_mesh_owner.clone()
}

pub(super) fn add_whole_mesh_entities<O: MeshVSMeshSource + ?Sized>(
    scope: &mut SelectionScope<'_, O>,
) -> Result<(), Failure> {
    let owner = whole_mesh_owner(scope);
    match scope.object.get_mesh_sel_method() {
        MeshVSMeshSelectionMethod::Box => {
            let mut bounds = BoundingBox3::default();
            scope.object.bounding_box(&mut bounds);
            if !bounds.is_void() {
                scope.add(Handle::new(Select3DSensitiveBox::new(&owner, &bounds)?).cast())?;
            }
        }
        MeshVSMeshSelectionMethod::Nodes => {
            let entity = MeshVSCommonSensitiveEntity::new(
                &owner,
                scope.object,
                MeshVSMeshSelectionMethod::Nodes,
            )?;
            scope.add(Handle::new(entity).cast())?;
        }
        MeshVSMeshSelectionMethod::Precise => add_precise_entities(scope, &owner)?,
        MeshVSMeshSelectionMethod::Unnamed3 => {}
    }
    Ok(())
}

/// The common face entity, then a segment per selectable link with a node of no selectable
/// face (nor earlier link), then a point per selectable free node.
fn add_precise_entities<O: MeshVSMeshSource + ?Sized>(
    scope: &mut SelectionScope<'_, O>,
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
) -> Result<(), Failure> {
    let entity =
        MeshVSCommonSensitiveEntity::new(owner, scope.object, MeshVSMeshSelectionMethod::Precise)?;
    scope.add(Handle::new(entity).cast())?;
    let mut shared = MeshVSPackedMap::default();
    scan_faces_for_shared_nodes(
        scope.object,
        &scope.all_elements,
        scope.max_nodes,
        &mut shared,
    );
    let elements: Vec<i32> = scope.all_elements.iter().collect();
    for element in elements {
        add_free_link(scope, owner, element, &mut shared)?;
    }
    let nodes: Vec<i32> = scope.all_nodes.iter().collect();
    for node in nodes {
        let (mut nb_nodes, mut kind) = (0, MESHVS_ET_NONE);
        if scope.object.is_selectable_node(node)
            && scope
                .source
                .get_geom(node, false, &mut scope.coords, &mut nb_nodes, &mut kind)
            && !shared.contains(node)
        {
            scope.add(Handle::new(Select3DSensitivePoint::new(owner, scope.point(1))).cast())?;
        }
    }
    Ok(())
}

fn add_free_link<O: MeshVSMeshSource + ?Sized>(
    scope: &mut SelectionScope<'_, O>,
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    element: i32,
    shared: &mut MeshVSPackedMap,
) -> Result<(), Failure> {
    let mut kind = MESHVS_ET_NONE;
    if !scope.object.is_selectable_elem(element)
        || !scope.source.get_geom_type(element, true, &mut kind)
        || kind != MESHVS_ET_LINK
    {
        return Ok(());
    }
    let mut nb_nodes = 0;
    scope
        .source
        .get_geom(element, true, &mut scope.coords, &mut nb_nodes, &mut kind);
    if nb_nodes == 0 {
        return Ok(());
    }
    let Ok(mut nodes) = Array1::<i32>::with_bounds(1, nb_nodes) else {
        return Ok(());
    };
    if !scope
        .source
        .get_nodes_by_element(element, &mut nodes, &mut nb_nodes)
    {
        return Ok(());
    }
    let mut points = Vec::new();
    let mut verts_shared = true;
    for i in 1..=nb_nodes {
        points.push(scope.point(i));
        let node = nodes.value(i).copied().unwrap_or_default();
        verts_shared = verts_shared && shared.contains(node);
        shared.add(node);
    }
    if !verts_shared {
        let (Some(first), Some(second)) = (points.first(), points.get(1)) else {
            return Err(Failure::with_type(
                "Standard_OutOfRange",
                &format!("MeshVS link {element} nodes={nb_nodes}: expected 2"),
                "",
            ));
        };
        scope.add(Handle::new(Select3DSensitiveSegment::new(owner, *first, *second)).cast())?;
    }
    Ok(())
}
