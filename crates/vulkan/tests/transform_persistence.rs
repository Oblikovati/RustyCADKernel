//! Persistence reaches the real viewer draw stream, preserving viewport and local-anchor rules.
#[path = "support/persistence_scene.rs"]
mod persistence_scene;
#[path = "support/recording_device.rs"]
mod recording_device;
#[path = "support/viewer_scene.rs"]
mod scene;
#[path = "support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::*;
use recording_device::RecordingDevice;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::{
    gp::{Point3, Transform3, Vector3},
    toploc::Datum3,
};
use rustycad_tkservice::graphic3d::*;
fn setup() -> (scene::ViewerScene, RecordingDevice) {
    let device = RecordingDevice::default();
    let driver = Graphic3dRenderDriver::with_backend(device.clone(), 512).unwrap();
    (scene::ViewerScene::new(&driver), device)
}
#[test]
fn all_persistence_modes_reach_the_viewer_and_select_native_depth_policy() {
    for mode in persistence_scene::MODES {
        let (scene, device) = setup();
        persistence_scene::configure_persistence(
            &scene.structure,
            &*scene.group,
            &scene.vertices,
            &scene.view.camera(),
            mode,
        );
        scene.render_view().try_redraw().unwrap();
        let frame = device.last_frame();
        assert_eq!(frame.draws().len(), 1, "{mode}");
        assert_eq!(
            frame.draws()[0].depth_clamp(),
            mode.starts_with("pers-camera"),
            "{mode}"
        );
        assert!(
            frame.draws()[0]
                .clip_transform()
                .iter()
                .all(|v| v.is_finite())
        );
        assert_eq!(scene.viewer.active_views().len(), 1);
        assert!(scene.cview().is_active());
    }
}
#[test]
fn local_anchor_includes_model_translation_once() {
    let (scene, device) = setup();
    // Recalculated group-only persistence bounds are void in native OCCT;
    // inspect its model matrix with frustum culling explicitly disabled.
    scene.view.change_rendering_params(&mut |params| {
        params.ray_tracing.frustum_culling_state = Graphic3dRenderingParamsFrustumCulling::Off;
    });
    let camera = scene.view.camera();
    let persistence = Handle::new(
        Graphic3dTransformPers::with_point(
            Graphic3dTransModeFlags::ZOOM_PERS,
            &Point3::new(1., 0., 0.),
        )
        .unwrap(),
    )
    .cast();
    scene.group.set_transform_persistence(&persistence);
    scene
        .structure
        .set_transformation(&Handle::new(Datum3::new(Transform3::translation_vector(
            Vector3::new(2., 0., 0.),
        ))));
    scene
        .group
        .set_transformation(&Transform3::translation_vector(Vector3::new(3., 0., 0.)));
    scene.render_view().try_redraw().unwrap();
    let frame = device.last_frame();
    let matrix = frame.draws()[0].clip_transform();
    assert!((matrix[12] - 0.6).abs() < 1e-6);
    assert!((matrix[0] - 2. / 64.).abs() < 1e-6);
    assert_eq!(camera.eye(), Point3::new(0., 0., 20.));
}
#[test]
fn persistence_tracks_live_anchor_and_capture_extent() {
    let (scene, device) = setup();
    let persistence = persistence_scene::scene_persistence("pers-zoom");
    scene.structure.set_transform_persistence(&persistence);
    scene.render_view().try_redraw().unwrap();
    let before = *device.last_frame().draws()[0].clip_transform();
    persistence
        .set_anchor_point(&Point3::new(2., 0., 0.))
        .unwrap();
    scene.window.set_size(128, 128);
    scene.view.must_be_resized();
    scene.render_view().try_redraw().unwrap();
    let after = *device.last_frame().draws()[0].clip_transform();
    assert!((before[0] / after[0] - 2.).abs() < 1e-6);
    assert_ne!(before[12], after[12]);
    scene
        .structure
        .set_transform_persistence(&Handle::default());
    scene.render_view().try_redraw().unwrap();
    assert!((device.last_frame().draws()[0].clip_transform()[0] - 0.1).abs() < 1e-6);
}
#[test]
fn direct_persistent_extraction_requires_a_valid_viewport() {
    let (scene, _) = setup();
    scene
        .structure
        .set_transform_persistence(&persistence_scene::scene_persistence("pers-zoom"));
    let group = scene.group.cast::<Graphic3dDrawGroup>();
    assert!(
        group
            .triangle_draws(&scene.view.camera())
            .unwrap_err()
            .to_string()
            .contains("viewport")
    );
    for extent in [[0, 64], [64, 0], [u32::MAX, 64]] {
        assert!(
            group
                .triangle_draws_for_view(&scene.view.camera(), extent)
                .is_err()
        );
    }
    assert!(
        group
            .triangle_draws_for_view(&scene.view.camera(), [64, 64])
            .is_ok()
    );
}

#[test]
fn orthographic_persistence_does_not_mutate_the_live_perspective_camera() {
    let (scene, device) = setup();
    let camera = scene.view.camera();
    camera.set_projection_type(Graphic3dCameraProjection::Perspective);
    let eye = camera.eye();
    let projection = camera.projection_matrix();
    scene
        .structure
        .set_transform_persistence(&persistence_scene::scene_persistence("pers-ortho"));
    scene.render_view().try_redraw().unwrap();
    assert_eq!(
        camera.projection_type(),
        Graphic3dCameraProjection::Perspective
    );
    assert_eq!(camera.eye(), eye);
    assert_eq!(camera.projection_matrix(), projection);
    assert!((device.last_frame().draws()[0].clip_transform()[15] - 1.).abs() < 1e-5);
}
