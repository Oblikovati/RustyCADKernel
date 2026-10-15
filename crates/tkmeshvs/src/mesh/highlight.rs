// SPDX-License-Identifier: GPL-2.0-only
//! `MeshVS_Mesh::HilightSelected`, `HilightOwnerWithColor` and `ClearSelected`: the
//! highlighter builder draws the selected or detected entities with the selection or
//! highlight drawer into the selection or highlight presentation.
use super::{MeshVSMeshMembers, MeshVSMeshSource};
use crate::MeshVSDrawerAttribute as Key;
use crate::data_source::{MeshVSDataSource, MeshVSPackedMap};
use crate::drawer::MeshVSDrawerSource;
use crate::flags::*;
use crate::mesh_entity_owner::MeshVSMeshEntityOwner;
use crate::mesh_owner::MeshVSMeshOwnerSource;
use crate::prs_builder::{MeshVSPrsBuilder, MeshVSPrsBuilderMembers};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkservice::graphic3d::*;
use rustycad_tkv3d::ais::AisInteractiveObjectMembers;
use rustycad_tkv3d::prs_mgr::{PrsMgrPresentableObjectMembers, PrsMgrPresentationManager};
use rustycad_tkv3d::prs3d::Prs3dDrawer;
use rustycad_tkv3d::select_mgr::{SelectMgrEntityOwnerSource, SelectMgrSelectableObjectMembers};

fn hilight_mode<O: MeshVSMeshSource + ?Sized>(object: &O) -> i32 {
    if object.has_hilight_mode() {
        object.hilight_mode()
    } else {
        0
    }
}

/// The selected node and element ids of the owners (stopping at a whole-mesh owner).
#[derive(Default)]
struct SelectedIds {
    nodes: MeshVSPackedMap,
    elements: MeshVSPackedMap,
}

/// The native `MeshVS_Mesh::HilightSelected` body.
/// Example: `meshvs_mesh_hilight_selected(&*mesh, &manager, &owners)?`.
pub fn meshvs_mesh_hilight_selected<O: MeshVSMeshSource + ?Sized>(
    object: &O,
    manager: &Handle<PrsMgrPresentationManager>,
    owners: &[Handle<dyn SelectMgrEntityOwnerSource>],
) -> Result<(), Failure> {
    let hilighter = object.get_hilighter();
    let Some(builder) = hilighter.get() else {
        return Ok(());
    };
    let prs = object.get_select_presentation(manager)?;
    let prs_value = prs.get().ok_or_else(|| null_presentation("selection"))?;
    if object.has_presentation() {
        prs_value.set_transform_persistence(
            &object
                .presentation()?
                .get()
                .map(|p| p.transform_persistence())
                .unwrap_or_default(),
        );
    }
    let global = object.global_sel_owner();
    let source = object.get_data_source();
    let mut ids = SelectedIds::default();
    for owner in owners {
        if owner.object_address() == global.object_address() {
            color_whole_object(object, manager)?;
            continue;
        }
        if collect_owner(object, &source, owner, &mut ids) {
            break;
        }
    }
    prs_value.clear(true);
    let selection_drawer = object.mesh_base().fields.read().selection_drawer.clone();
    builder.set_drawer(&selection_drawer);
    let result = build_ids(builder, &prs, &ids, MESHVS_DMF_SELECTION_PRS);
    builder.set_drawer(&Handle::default());
    result?;
    prs_value.set_z_layer(GRAPHIC3D_ZLAYER_ID_TOP);
    prs_value.set_display_priority(Graphic3dDisplayPriority::Highlight)?;
    prs_value.display();
    Ok(())
}

fn null_presentation(kind: &str) -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        &format!("MeshVS_Mesh {kind} presentation=null: expected a presentation manager"),
        "",
    )
}

/// The global owner: the manager colors the whole object with the highlight or selection style.
fn color_whole_object<O: MeshVSMeshSource + ?Sized>(
    object: &O,
    manager: &Handle<PrsMgrPresentationManager>,
) -> Result<(), Failure> {
    let own = object.hilight_attributes();
    let context = object.get_context();
    let style = match (own.is_null(), context.get()) {
        (false, _) => own,
        (true, Some(context)) => context.selection_style().clone(),
        (true, None) => Handle::default(),
    };
    match manager.get() {
        Some(m) => m.color(
            &object.presentable_base().this(),
            &style,
            hilight_mode(object),
            &Handle::default(),
            GRAPHIC3D_ZLAYER_ID_TOPMOST,
        ),
        None => Ok(()),
    }
}

/// Adds the ids of one owner; true for the whole-mesh owner, which ends the loop natively.
fn collect_owner<O: MeshVSMeshSource + ?Sized>(
    object: &O,
    source: &Handle<dyn MeshVSDataSource>,
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    ids: &mut SelectedIds,
) -> bool {
    let entity = owner.cast::<MeshVSMeshEntityOwner>();
    if let Some(entity) = entity.get() {
        collect_entity(object, source, entity, ids);
        return false;
    }
    if source
        .get()
        .is_some_and(|s| s.is_advanced_selection_enabled())
    {
        let mesh_owner = owner.cast::<dyn MeshVSMeshOwnerSource>();
        if let Some(mesh_owner) = mesh_owner.get() {
            if let Some(nodes) = mesh_owner.get_selected_nodes().get() {
                ids.nodes.assign(&nodes.map());
            }
            if let Some(elements) = mesh_owner.get_selected_elements().get() {
                ids.elements.assign(&elements.map());
            }
        }
        return false;
    }
    if !object.is_whole_mesh_owner(owner) {
        return false;
    }
    if let Some(source) = source.get() {
        for node in source
            .get_all_nodes()
            .iter()
            .filter(|&n| !object.is_hidden_node(n))
        {
            ids.nodes.add(node);
        }
        for element in source
            .get_all_elements()
            .iter()
            .filter(|&e| !object.is_hidden_elem(e))
        {
            ids.elements.add(element);
        }
    }
    true
}

/// A group adds its selectable members, an entity its own id.
fn collect_entity<O: MeshVSMeshSource + ?Sized>(
    object: &O,
    source: &Handle<dyn MeshVSDataSource>,
    entity: &MeshVSMeshEntityOwner,
    ids: &mut SelectedIds,
) {
    if !entity.is_group() {
        let target = if entity.kind() == MESHVS_ET_NODE {
            &mut ids.nodes
        } else {
            &mut ids.elements
        };
        target.add(entity.id());
        return;
    }
    let (mut kind, mut members) = (MESHVS_ET_NONE, MeshVSPackedMap::default());
    if !source
        .get()
        .is_some_and(|s| s.get_group(entity.id(), &mut kind, &mut members))
    {
        return;
    }
    for key in members.iter() {
        if kind == MESHVS_ET_NODE {
            if object.is_selectable_node(key) {
                ids.nodes.add(key);
            }
        } else if object.is_selectable_elem(key) {
            ids.elements.add(key);
        }
    }
}

fn build_ids(
    builder: &dyn MeshVSPrsBuilder,
    prs: &Handle<dyn Graphic3dStructureSource>,
    ids: &SelectedIds,
    mode: i32,
) -> Result<(), Failure> {
    if ids.nodes.extent() > 0 {
        builder.build(
            prs,
            &ids.nodes,
            &mut MeshVSPackedMap::default(),
            false,
            mode,
        )?;
    }
    if ids.elements.extent() > 0 {
        builder.build(
            prs,
            &ids.elements,
            &mut MeshVSPackedMap::default(),
            true,
            mode,
        )?;
    }
    Ok(())
}

/// The native `MeshVS_Mesh::HilightOwnerWithColor` body.
/// Example: `meshvs_mesh_hilight_owner_with_color(&*mesh, &manager, &style, &owner)?`.
pub fn meshvs_mesh_hilight_owner_with_color<O: MeshVSMeshSource + ?Sized>(
    object: &O,
    manager: &Handle<PrsMgrPresentationManager>,
    style: &Handle<Prs3dDrawer>,
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
) -> Result<(), Failure> {
    if owner.is_null() {
        return Ok(());
    }
    let color = style.color();
    if owner.object_address() == object.global_sel_owner().object_address() {
        return match manager.get() {
            Some(m) => m.color(
                &object.presentable_base().this(),
                style,
                hilight_mode(object),
                &Handle::default(),
                GRAPHIC3D_ZLAYER_ID_TOP,
            ),
            None => Ok(()),
        };
    }
    let hilighter = object.get_hilighter();
    let Some(builder) = hilighter.get() else {
        return Ok(());
    };
    let prs = object.get_hilight_presentation(manager)?;
    let prs_value = prs.get().ok_or_else(|| null_presentation("highlight"))?;
    prs_value.clear(true);
    if object.has_presentation() {
        prs_value.set_transform_persistence(
            &object
                .presentation()?
                .get()
                .map(|p| p.transform_persistence())
                .unwrap_or_default(),
        );
    }
    let disp_mode = highlight_display_mode(object);
    let hilight_drawer = object.mesh_base().fields.read().hilight_drawer.clone();
    paint_hilight_drawer(&hilight_drawer, color);
    builder.set_drawer(&hilight_drawer);
    let result = build_owner(object, builder, &prs, owner, disp_mode);
    if result.is_ok() {
        prs_value.set_z_layer(GRAPHIC3D_ZLAYER_ID_TOPMOST);
        if let Some(m) = manager.get().filter(|m| m.is_immediate_mode_on()) {
            m.add_to_immediate_list(&prs);
        }
    }
    builder.set_drawer(&Handle::default());
    result
}

/// Shading, or the own OCC mode above wireframe (owners are drawn shaded or shrunk only).
fn highlight_display_mode<O: MeshVSMeshSource + ?Sized>(object: &O) -> i32 {
    if object.has_display_mode()
        && (object.display_mode() & MESHVS_DMF_OCC_MASK) > MESHVS_DMF_WIRE_FRAME
    {
        return object.display_mode() & MESHVS_DMF_OCC_MASK;
    }
    MESHVS_DMF_SHADING
}

fn paint_hilight_drawer(
    drawer: &Handle<dyn MeshVSDrawerSource>,
    color: rustycad_tkernel::quantity::Color,
) {
    let Some(drawer) = drawer.get() else {
        return;
    };
    for key in [
        Key::InteriorColor,
        Key::BackInteriorColor,
        Key::EdgeColor,
        Key::BeamColor,
        Key::MarkerColor,
    ] {
        drawer.set_color(key as i32, &color);
    }
}

fn build_owner<O: MeshVSMeshSource + ?Sized>(
    object: &O,
    builder: &dyn MeshVSPrsBuilder,
    prs: &Handle<dyn Graphic3dStructureSource>,
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    disp_mode: i32,
) -> Result<(), Failure> {
    let mode = disp_mode | MESHVS_DMF_HILIGHT_PRS;
    let source = object.get_data_source();
    if let Some(entity) = owner.cast::<MeshVSMeshEntityOwner>().get() {
        let is_element = entity.kind() != MESHVS_ET_NODE;
        let mut ids = MeshVSPackedMap::default();
        if !entity.is_group() {
            ids.add(entity.id());
        } else {
            let mut kind = MESHVS_ET_NONE;
            if !source
                .get()
                .is_some_and(|s| s.get_group(entity.id(), &mut kind, &mut ids))
            {
                return Ok(());
            }
        }
        return builder.build(prs, &ids, &mut MeshVSPackedMap::default(), is_element, mode);
    }
    if object.is_whole_mesh_owner(owner) {
        if let Some(source) = source.get() {
            builder.build(
                prs,
                &source.get_all_elements(),
                &mut MeshVSPackedMap::default(),
                true,
                MESHVS_DMF_WIRE_FRAME,
            )?;
        }
        return Ok(());
    }
    let mesh_owner = owner.cast::<dyn MeshVSMeshOwnerSource>();
    let Some(mesh_owner) = mesh_owner.get() else {
        return Ok(());
    };
    if let Some(nodes) = mesh_owner.get_detected_nodes().get() {
        let mut map = MeshVSPackedMap::default();
        map.assign(&nodes.map());
        builder.build(prs, &map, &mut MeshVSPackedMap::default(), false, mode)?;
    }
    if let Some(elements) = mesh_owner.get_detected_elements().get() {
        let mut map = MeshVSPackedMap::default();
        map.assign(&elements.map());
        builder.build(prs, &map, &mut MeshVSPackedMap::default(), true, mode)?;
    }
    Ok(())
}

/// The native `MeshVS_Mesh::ClearSelected` body: clears the selection presentation.
/// Example: `meshvs_mesh_clear_selected(&*mesh)`.
pub fn meshvs_mesh_clear_selected<O: MeshVSMeshSource + ?Sized>(object: &O) {
    let prs = object
        .get_select_presentation(&Handle::default())
        .unwrap_or_default();
    if let Some(prs) = prs.get() {
        prs.clear(true);
    }
}
