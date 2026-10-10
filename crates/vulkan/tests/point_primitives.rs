//! CPU submissions prove sprite validation, AIS appearance and batched geometry.
#[path = "support/ais_point_scene.rs"]
mod point_scene;
#[path = "support/recording_device.rs"]
mod recording_device;
#[allow(dead_code)]
#[path = "support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::*;
use recording_device::RecordingDevice;
fn setup() -> (point_scene::PointScene, RecordingDevice) {
    let device = RecordingDevice::default();
    let driver = Graphic3dRenderDriver::with_backend(device.clone(), 128).unwrap();
    (point_scene::PointScene::new(&driver), device)
}
#[test]
fn invalid_sprite_shapes_and_components_are_rejected() {
    for (size, extent, pixels) in [
        (0., [1, 1], vec![[1.; 4]]),
        (f32::NAN, [1, 1], vec![[1.; 4]]),
        (1., [0, 1], vec![]),
        (1., [2, 1], vec![[1.; 4]]),
        (1., [1, 1], vec![[2.; 4]]),
        (1., [1, 1], vec![[f32::NAN; 4]]),
    ] {
        assert!(PointStyle::new(size, extent, pixels).is_err());
    }
}
#[test]
fn indexed_point_submission_preserves_order_and_rejects_missing_vertices() {
    let vertices = vec![
        RenderVertex::new([0.; 3], [1.; 4]),
        RenderVertex::new([1.; 3], [1.; 4]),
    ];
    let style = PointStyle::new(3., [1, 1], vec![[1.; 4]]).unwrap();
    let draw = PrimitiveDraw::points(vertices.clone(), vec![1, 0], identity_clip(), style.clone())
        .unwrap();
    assert_eq!(draw.topology(), PrimitiveTopology::Points);
    assert_eq!(draw.indices(), &[1, 0]);
    assert_eq!(draw.point_style().unwrap().parameters(), (3., [1, 1]));
    assert_eq!(draw.point_style().unwrap().pixels(), &[[1.; 4]]);
    assert!(PrimitiveDraw::points(vertices, vec![2], identity_clip(), style).is_err());
}
#[test]
fn all_point_modes_lower_to_sprites_or_explicitly_empty_frames() {
    for mode in point_scene::MODES {
        let (scene, device) = setup();
        scene.apply(mode).unwrap();
        scene.render_view().try_redraw().unwrap();
        let frame = device.last_frame();
        assert_eq!(
            frame.draws().is_empty(),
            matches!(mode, "point-empty" | "point-erased"),
            "{mode}"
        );
        for draw in frame.draws() {
            assert_eq!(draw.topology(), PrimitiveTopology::Points, "{mode}");
            assert!(draw.point_style().is_some(), "{mode}");
        }
        scene.view.remove();
    }
}
#[test]
fn batch_retains_one_draw_for_1024_points() {
    let (scene, device) = setup();
    scene.batch().unwrap();
    scene.render_view().try_redraw().unwrap();
    let frame = device.last_frame();
    assert_eq!(frame.draws().len(), 1);
    assert_eq!(frame.draws()[0].indices().len(), 1024);
    assert_eq!(frame.draws()[0].vertices().len(), 1024);
    scene.view.remove();
}
