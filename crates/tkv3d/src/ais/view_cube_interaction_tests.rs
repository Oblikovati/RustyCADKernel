//! Native constructor/picking/click closure: tests observe real owners, timers and cameras.
use super::{view_cube_interaction_fixture::*, *};
use crate::{prs_mgr::*, select_basics::*, select_mgr::*, select3d::*, v3d::*};
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::{Axis1, Direction3, Point3};
use rustycad_tkservice::{aspect::*, graphic3d::*};
use std::sync::atomic::Ordering;
#[test]
fn native_view_cube_complete_constructor_retains_animation_and_dynamic_drawer() {
    let scene = WidgetScene::new(false);
    let cube = &scene.cube;
    assert_widget_defaults(cube);
    assert_widget_camera(
        &cube.interaction.fields.read().start,
        &widget_rows(STATE, "default-start").next().unwrap(),
    );
    assert_widget_camera(
        &cube.interaction.fields.read().end,
        &widget_rows(STATE, "default-end").next().unwrap(),
    );
    assert!(cube.accept_display_mode(0) && !cube.accept_display_mode(1));
}
fn assert_widget_defaults(cube: &AisViewCube) {
    let row = widget_rows(STATE, "defaults").next().unwrap();
    let dyn_style = cube.dynamic_hilight_attributes();
    let color = dyn_style
        .shading_aspect()
        .color(AspectTypeOfFacingModel::BothSide);
    let actual = widget_constructor_channels(cube, &dyn_style, color);
    assert_eq!(
        actual.to_vec(),
        row.iter()
            .map(|v| v.parse::<f64>().unwrap())
            .collect::<Vec<_>>()
    );
}

fn widget_constructor_channels(
    cube: &AisViewCube,
    dyn_style: &crate::prs3d::Prs3dDrawer,
    color: rustycad_tkernel::quantity::Color,
) -> [f64; 14] {
    [
        cube.duration().unwrap(),
        cube.has_animation().unwrap() as i32 as f64,
        cube.to_auto_start_animation() as i32 as f64,
        cube.is_fixed_animation_loop() as i32 as f64,
        cube.to_fit_selected() as i32 as f64,
        cube.to_reset_camera_up() as i32 as f64,
        cube.is_auto_hilight() as i32 as f64,
        cube.global_sel_owner().is_null() as i32 as f64,
        dyn_style.shading_aspect().aspect().shading_model() as i32 as f64,
        dyn_style.z_layer() as f64,
        color.red(),
        color.green(),
        color.blue(),
        (dyn_style.link() == cube.attributes()) as i32 as f64,
    ]
}

#[test]
fn native_cube_actual_sensitive_owners_factors_centers_bounds_and_append_modes() {
    for policy in 0..3 {
        replay_widget_selection(policy);
    }
}
fn replay_widget_selection(policy: i32) {
    let scene = WidgetScene::new(false);
    let cube = &scene.cube;
    configure_widget_selection(cube, policy);
    let selection = Handle::new(SelectMgrSelection::new(0));
    cube.compute_selection(&selection, 0).unwrap();
    let expected = widget_rows(STATE, "sensitive")
        .filter(|row| row[0].parse::<i32>().unwrap() == policy)
        .collect::<Vec<_>>();
    assert_widget_selection_count(&selection, policy, "selection");
    assert_eq!(selection.entities().len(), expected.len());
    for (entity, row) in selection.entities().iter().zip(expected) {
        assert_widget_sensitive(&scene, &entity.base_sensitive(), &row);
    }
    cube.compute_selection(&selection, 1).unwrap();
    assert_widget_selection_count(&selection, policy, "unsupported");
}
fn configure_widget_selection(cube: &AisViewCube, policy: i32) {
    if policy == 1 {
        cube.view_cube_presentation_base()
            .settings
            .write()
            .geometry
            .round_radius = 0.2;
    }
    if policy == 2 {
        let mut settings = cube.view_cube_presentation_base().settings.write();
        settings.draw_edges = false;
        settings.draw_corners = false;
    }
}

fn assert_widget_sensitive(
    scene: &WidgetScene,
    entity: &Handle<dyn Select3DSensitiveEntity>,
    row: &[&str],
) {
    let owner = entity.owner_id().cast::<AisViewCubeOwner>();
    let numbers = [
        owner.main_orientation() as i32,
        owner.priority(),
        owner.comes_from_decomposition() as i32,
        owner.is_forced_hilight() as i32,
        entity.sensitivity_factor(),
        entity.nb_sub_elements(),
    ];
    assert_eq!(
        numbers.to_vec(),
        row[1..7]
            .iter()
            .map(|v| v.parse::<i32>().unwrap())
            .collect::<Vec<_>>()
    );
    assert_widget_sensitive_bounds(entity, row);
    assert_widget_sensitive_identity(scene, entity, &owner);
}
fn assert_widget_sensitive_identity(
    scene: &WidgetScene,
    entity: &Handle<dyn Select3DSensitiveEntity>,
    owner: &AisViewCubeOwner,
) {
    assert_eq!(
        owner.selectable(),
        scene.cube.cast::<dyn SelectMgrSelectableObject>()
    );
    assert!(!entity.get_connected().unwrap().is_null());
    entity.bvh().unwrap();
    assert!(!entity.to_build_bvh());
}
fn assert_widget_sensitive_bounds(entity: &Handle<dyn Select3DSensitiveEntity>, row: &[&str]) {
    let bounds = entity.bounding_box();
    let center = entity.center_of_geometry();
    let coordinates = [
        center.x(),
        center.y(),
        center.z(),
        bounds.corner_min().x,
        bounds.corner_min().y,
        bounds.corner_min().z,
        bounds.corner_max().x,
        bounds.corner_max().y,
        bounds.corner_max().z,
    ];
    for (actual, expected) in coordinates.iter().zip(&row[7..16]) {
        assert!((actual - expected.parse::<f64>().unwrap()).abs() < 1e-6);
    }
}

#[test]
fn native_cube_point_only_tangent_ray_rejection_and_real_hit() {
    let scene = WidgetScene::new(false);
    let owner = AisViewCubeOwner::new(&scene.cube, V3dTypeOfOrientation::Zpos, 5);
    let array = super::view_cube_selection::cube_part_selection_triangles(
        &*scene.cube,
        V3dTypeOfOrientation::Zpos,
    )
    .unwrap();
    let sensitive = AisViewCubeSensitive::new(&owner.cast(), &array).unwrap();
    for row in widget_rows(STATE, "ray") {
        assert_widget_ray(&sensitive, &row);
    }
    assert_widget_ray_selection_types(&sensitive);
}
fn assert_widget_ray_selection_types(sensitive: &AisViewCubeSensitive) {
    let mut manager = SelectMgrSelectingVolumeManager::new();
    manager.init_axis_selecting_volume(Axis1::new(
        Point3::new(0., 0., 100.),
        Direction3::Z.reversed(),
    ));
    manager.build_selecting_volume().unwrap();
    assert!(
        sensitive
            .matches(&manager, &mut SelectBasicsPickResult::default())
            .unwrap()
    );
    manager.init_box_selecting_volume(
        rustycad_tkmath::gp::Point2::new(0., 0.),
        rustycad_tkmath::gp::Point2::new(10., 10.),
    );
    assert!(!sensitive.is_valid_ray(&manager).unwrap());
}
fn assert_widget_ray(sensitive: &AisViewCubeSensitive, row: &[&str]) {
    let angle = row[0].parse::<f64>().unwrap().to_radians();
    let direction = Direction3::new(angle.sin(), 0., angle.cos()).unwrap();
    let mut manager = SelectMgrSelectingVolumeManager::new();
    manager.init_axis_selecting_volume(Axis1::new(Point3::new(0., 0., 100.), direction));
    manager.build_selecting_volume().unwrap();
    assert_eq!(sensitive.is_valid_ray(&manager).unwrap(), row[1] == "1");
    let mut result = SelectBasicsPickResult::default();
    let matched = sensitive.matches(&manager, &mut result).unwrap();
    assert_eq!(matched, row[2] == "1");
    if matched {
        assert!((result.depth() - row[3].parse::<f64>().unwrap()).abs() < 1e-6);
    }
}
#[test]
fn native_cube_all_orientation_fitting_target_and_real_camera_animation_replay() {
    for id in 0..156 {
        replay_widget_animation(id);
    }
}
fn replay_widget_animation(id: i32) {
    let scene = WidgetScene::new(id >= 78);
    let owner = configure_widget_target(&scene, id);
    let original = scene.view.camera();
    scene.cube.start_animation(&owner).unwrap();
    for (kind, camera) in [
        ("start", scene.cube.interaction.fields.read().start.clone()),
        ("end", scene.cube.interaction.fields.read().end.clone()),
    ] {
        let row = widget_rows(TARGETS, kind)
            .find(|r| r[0].parse::<i32>().unwrap() == id)
            .unwrap();
        assert_widget_camera(&camera, &row);
    }
    assert_widget_start_state(&scene, id, &original);
    for (index, pts) in [0., 0.125, 0.25, 0.5].into_iter().enumerate() {
        replay_widget_frame(&scene, id, index, pts);
    }
}
fn replay_widget_frame(scene: &WidgetScene, id: i32, index: usize, pts: f64) {
    scene.clock.set(pts);
    let continuing = scene.cube.update_animation(false).unwrap();
    let frames = if id < 78 {
        ORTHO_FRAMES
    } else {
        PERSPECTIVE_FRAMES
    };
    let steps = if id < 78 {
        ORTHO_STEPS
    } else {
        PERSPECTIVE_STEPS
    };
    let frame = widget_rows(frames, "frame")
        .find(|r| r[0].parse::<usize>().unwrap() == id as usize * 4 + index)
        .unwrap();
    assert_widget_camera(&scene.view.camera(), &frame);
    let step = widget_rows(steps, "step")
        .find(|r| r[0].parse::<i32>().unwrap() == id && r[1].parse::<f64>().unwrap() == pts)
        .unwrap();
    assert_widget_step(scene, continuing, &step);
}
fn assert_widget_step(scene: &WidgetScene, continuing: bool, step: &[&str]) {
    let actual = [
        continuing as usize,
        scene.events.frames.load(Ordering::SeqCst),
        scene.events.finished.load(Ordering::SeqCst),
        scene.events.finish_attached.load(Ordering::SeqCst) as usize,
        scene.cube.view_animation().target_view().is_null() as usize,
    ];
    assert_eq!(
        actual.to_vec(),
        step[2..]
            .iter()
            .map(|v| v.parse::<usize>().unwrap())
            .collect::<Vec<_>>()
    );
}
#[test]
fn native_cube_owner_click_disabled_manual_and_fixed_loop_policies() {
    for row in widget_rows(STATE, "click") {
        replay_widget_click(&row);
    }
}
fn replay_widget_click(row: &[&str]) {
    let policy = row[0].parse::<i32>().unwrap();
    let scene = WidgetScene::new(false);
    scene.cube.set_auto_start_animation(policy != 0);
    scene.cube.set_fixed_animation_loop(policy != 1);
    scene.events.advance_clock.store(true, Ordering::SeqCst);
    let owner = AisViewCubeOwner::new(&scene.cube, V3dTypeOfOrientation::Xpos, 5);
    let handled = owner.handle_mouse_click(
        rustycad_tkernel::ncollection::Vec2::new(13, 17),
        ASPECT_VKEY_MOUSE_RIGHT_BUTTON,
        ASPECT_VKEY_FLAGS_SHIFT,
        true,
    );
    assert_widget_click_state(&scene, handled, row);
}
fn assert_widget_click_state(scene: &WidgetScene, handled: bool, row: &[&str]) {
    let actual = [
        handled as usize,
        scene.cube.has_animation().unwrap() as usize,
        scene.events.frames.load(Ordering::SeqCst),
        scene.events.finished.load(Ordering::SeqCst),
        scene.events.finish_attached.load(Ordering::SeqCst) as usize,
        scene.cube.view_animation().target_view().is_null() as usize,
    ];
    assert_eq!(
        actual.to_vec(),
        row[1..]
            .iter()
            .map(|v| v.parse::<usize>().unwrap())
            .collect::<Vec<_>>()
    );
}

fn assert_widget_start_state(scene: &WidgetScene, id: i32, original: &Handle<Graphic3dCamera>) {
    let row = widget_rows(TARGETS, "start-state")
        .find(|r| r[0].parse::<i32>().unwrap() == id)
        .unwrap();
    let fields = scene.cube.interaction.fields.read();
    let actual = [
        scene.cube.has_animation().unwrap(),
        fields.animation.target_view() == scene.view,
        fields.animation.camera_start() == fields.start,
        fields.animation.camera_end() == fields.end,
        scene.view.camera() == *original,
    ];
    assert_eq!(
        actual.to_vec(),
        row[1..]
            .iter()
            .map(|value| *value == "1")
            .collect::<Vec<_>>()
    );
}

fn assert_widget_selection_count(selection: &SelectMgrSelection, policy: i32, kind: &str) {
    let row = widget_rows(STATE, kind)
        .find(|row| row[0].parse::<i32>().unwrap() == policy)
        .unwrap();
    assert_eq!(selection.entities().len(), row[1].parse::<usize>().unwrap());
}
