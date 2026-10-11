//! Cached highlight presentation observes real native part submission and early-return policy.
use super::{color_scale_recording::*, view_cube_interaction_fixture::*, *};
use crate::{prs_mgr::*, prs3d::*, select_mgr::*, v3d::*};
use rustycad_tkernel::standard::{Handle, SharedState};
use rustycad_tkservice::graphic3d::*;
use std::sync::Arc;
#[test]
fn native_cube_immediate_highlight_geometry_layer_persistence_and_affinity() {
    for row in widget_rows(STATE, "highlight") {
        replay_cube_highlight(&row);
    }
}
fn replay_cube_highlight(row: &[&str]) {
    let id = row[0].parse::<i32>().unwrap();
    let scene = WidgetScene::new(false);
    configure_highlight_geometry(&scene.cube, id);
    let manager = PrsMgrPresentationManager::new(&scene.viewer.structure_manager());
    let prs = scene.cube.get_hilight_presentation(&manager).unwrap();
    let trace = Arc::new(SharedState::new(Vec::new()));
    install_scale_recording(&prs, &trace, true);
    let style = cube_highlight_style(&scene.cube, id);
    let orientation = V3dTypeOfOrientation::ALL[[0, 6, 18][(id % 3) as usize]];
    let owner = AisViewCubeOwner::new(&scene.cube, orientation, 5);
    if id < 9 {
        manager.begin_immediate_draw();
    }
    scene
        .cube
        .hilight_owner_with_color(&manager, &style, &owner.cast())
        .unwrap();
    assert_cube_highlight(&scene.cube, &prs, &trace, row);
    assert_eq!(manager.immediate.read().list.len(), usize::from(id < 9));
}
fn configure_highlight_geometry(cube: &AisViewCube, id: i32) {
    let mut settings = cube.view_cube_presentation_base().settings.write();
    if id % 6 >= 3 {
        settings.geometry.round_radius = 0.2;
    }
    if id >= 6 {
        settings.draw_edges = false;
        settings.draw_corners = false;
    }
}
fn cube_highlight_style(cube: &AisViewCube, id: i32) -> Handle<Prs3dDrawer> {
    let style = Handle::new(Prs3dDrawer::new());
    style.link_to(&cube.attributes());
    style.set_shading_aspect(Handle::new(Prs3dShadingAspect::new().unwrap()));
    style.set_z_layer(if id % 2 == 1 {
        7
    } else {
        GRAPHIC3D_ZLAYER_ID_UNKNOWN
    });
    style
}
fn assert_cube_highlight(
    cube: &AisViewCube,
    prs: &Handle<dyn Graphic3dStructureSource>,
    trace: &ScaleDrawTrace,
    row: &[&str],
) {
    let counts = cube_highlight_counts(trace);
    let actual = [
        prs.groups().len() as i32,
        counts[0],
        counts[1],
        counts[2],
        prs.get_z_layer(),
        (prs.transform_persistence() == cube.transform_persistence()) as i32,
        cube_highlight_affinity_matches(cube, prs) as i32,
    ];
    assert_eq!(
        actual.to_vec(),
        row[1..]
            .iter()
            .map(|v| v.parse::<i32>().unwrap())
            .collect::<Vec<_>>()
    );
}
fn cube_highlight_counts(trace: &ScaleDrawTrace) -> [i32; 3] {
    let mut counts = [0, 0, 0];
    for request in trace.read().iter() {
        if let ScaleDrawRequest::Array {
            vertices, indices, ..
        } = request
        {
            counts[0] += 1;
            counts[1] += vertices.len() as i32;
            counts[2] += indices.len() as i32;
        }
    }
    counts
}

#[test]
fn cube_ignores_wrong_null_and_selected_owners_without_replacing_cached_highlight() {
    let scene = WidgetScene::new(false);
    let manager = PrsMgrPresentationManager::new(&scene.viewer.structure_manager());
    manager.begin_immediate_draw();
    let style = cube_highlight_style(&scene.cube, 0);
    let wrong = SelectMgrEntityOwner::new(5);
    submit_cube_ignored_owners(&scene.cube, &manager, &style, &wrong);
    scene.cube.clear_selected();
    assert!(manager.immediate.read().list.is_empty());
    assert!(!scene.cube.has_animation().unwrap());
    assert!(
        scene
            .cube
            .selectable_base()
            .fields
            .read()
            .selection_prs
            .is_null()
    );
}

fn submit_cube_ignored_owners(
    cube: &AisViewCube,
    manager: &Handle<PrsMgrPresentationManager>,
    style: &Handle<Prs3dDrawer>,
    wrong: &Handle<SelectMgrEntityOwner>,
) {
    cube.hilight_owner_with_color(manager, style, &Handle::default())
        .unwrap();
    cube.hilight_owner_with_color(manager, style, &wrong.cast())
        .unwrap();
    cube.hilight_selected(manager, &[wrong.cast()]).unwrap();
}

fn cube_highlight_affinity_matches(
    cube: &AisViewCube,
    prs: &Handle<dyn Graphic3dStructureSource>,
) -> bool {
    prs.cstructure()
        .cstructure_base()
        .fields
        .read()
        .view_affinity
        == cube.view_affinity()
}
