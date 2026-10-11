//! Native brackets preserve live camera extraction and the selection flipper.
#[path = "support/capping_scene.rs"]
mod capping_scene;
#[path = "support/group_flipping_scene.rs"]
mod group_flipping_scene;
#[path = "support/recording_device.rs"]
mod recording_device;
#[path = "support/viewer_scene.rs"]
mod scene;
#[path = "support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::*;
use rustycad_tkmath::gp::{Axis2, Direction3};
use rustycad_tkservice::graphic3d::*;
fn setup() -> (scene::ViewerScene, recording_device::RecordingDevice) {
    let device = recording_device::RecordingDevice::default();
    let driver = Graphic3dRenderDriver::with_backend(device.clone(), 512).unwrap();
    let scene = scene::ViewerScene::new(&driver);
    scene.window.set_size(64, 64);
    (scene, device)
}
#[test]
fn every_flipping_mode_reaches_retained_draws() {
    for mode in group_flipping_scene::MODES {
        let (scene, device) = setup();
        group_flipping_scene::apply(&scene, mode, 2.);
        scene.render_view().try_redraw().unwrap();
        let frame = device.last_frame();
        assert!(!frame.draws().is_empty(), "{mode}");
        assert!(
            frame
                .draws()
                .iter()
                .flat_map(|draw| draw.clip_transform())
                .all(|v| v.is_finite()),
            "{mode}"
        );
        assert!(scene.structure.cstructure().has_group_flipping());
        assert!(!scene.group.flipper().is_null());
        assert_eq!(scene.viewer.active_views().len(), 1);
        if mode == "flip-lit" {
            assert!(frame.draws()[0].lighting().is_some());
        }
    }
}
#[test]
fn native_fill_filter_keeps_cap_masks_unflipped_and_bypasses_group_transform() {
    let (scene, device) = setup();
    group_flipping_scene::apply(&scene, "flip-cap", 2.);
    scene.render_view().try_redraw().unwrap();
    let frame = device.last_frame();
    let ordinary = frame
        .draws()
        .iter()
        .find(|draw| draw.capping_pass() == RenderCappingPass::None)
        .unwrap();
    let mask = frame
        .draws()
        .iter()
        .find(|draw| draw.capping_pass() == RenderCappingPass::MaskStart)
        .unwrap();
    // OpenGl_CappingAlgo enables FillModeOnly, rejecting OpenGl_Flipper (3d097a0).
    let ordinary_model = ordinary.clipping().unwrap().world_from_object();
    let mask_model = mask.clipping().unwrap().world_from_object();
    assert_eq!([ordinary_model[0], ordinary_model[12]], [-1., 9.]);
    assert_eq!([mask_model[0], mask_model[12]], [1., 2.]);
    assert!(mask.lighting().is_none());
}
#[test]
fn ordered_nested_commands_restore_ordinary_geometry_and_keep_selection_flipper() {
    let (scene, device) = setup();
    scene
        .view
        .camera()
        .set_up(&Direction3::new(0., -1., 0.).unwrap());
    scene.group.set_flipping_options(true, &Axis2::default());
    scene.group.add_primitive_array(&scene.vertices, true);
    scene.group.set_flipping_options(true, &Axis2::default());
    scene.group.add_primitive_array(&scene.vertices, true);
    scene.group.set_flipping_options(false, &Axis2::default());
    scene.group.set_flipping_options(false, &Axis2::default());
    scene.group.add_primitive_array(&scene.vertices, true);
    scene.render_view().try_redraw().unwrap();
    let frame = device.last_frame();
    assert_eq!(
        frame.draws()[0].clip_transform(),
        frame.draws()[3].clip_transform()
    );
    assert_eq!(
        frame.draws()[1].clip_transform(),
        frame.draws()[2].clip_transform()
    );
    assert_ne!(
        frame.draws()[0].clip_transform(),
        frame.draws()[1].clip_transform()
    );
    assert!(!scene.group.flipper().is_null());
}
#[test]
fn flipping_is_camera_dependent_and_clear_removes_the_brackets() {
    let (scene, device) = setup();
    group_flipping_scene::apply(&scene, "flip-front", 0.);
    scene.render_view().try_redraw().unwrap();
    let before = device.last_frame();
    scene
        .view
        .camera()
        .set_up(&Direction3::new(0., -1., 0.).unwrap());
    scene.render_view().try_redraw().unwrap();
    assert_eq!(
        before.draws()[0].clip_transform(),
        device.last_frame().draws()[0].clip_transform()
    );
    scene.group.clear(false);
    scene
        .group
        .set_group_primitives_aspect(&scene::unlit_style([1., 0., 0., 1.]));
    scene.group.add_primitive_array(&scene.vertices, true);
    scene.render_view().try_redraw().unwrap();
    assert_ne!(
        before.draws()[0].clip_transform(),
        device.last_frame().draws()[0].clip_transform()
    );
}
