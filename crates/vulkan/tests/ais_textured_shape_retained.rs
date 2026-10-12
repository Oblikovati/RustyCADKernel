//! Actual AIS class lowering exercises native modes before independent GPU replay.
#[path = "support/recording_device.rs"]
mod recording_device;
#[allow(dead_code)]
#[path = "support/viewer_scene.rs"]
mod scene;
#[path = "support/ais_textured_shape_scene.rs"]
mod textured_scene;
#[allow(dead_code)]
#[path = "support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::Graphic3dStructureMembers;

#[test]
fn concrete_textured_shape_retained_modes_submit_real_geometry() {
    for mode in textured_scene::modes::TEXTURED_SHAPE_SUPPORTED_MODES {
        let device = recording_device::RecordingDevice::default();
        let driver = Graphic3dRenderDriver::with_backend(device.clone(), 128).unwrap();
        let fixture = textured_scene::AisTexturedShapeScene::with_mode(&driver, mode).unwrap();
        fixture.scene.render_view().try_redraw().unwrap();
        fixture.mutate(mode).unwrap();
        fixture.scene.render_view().try_redraw().unwrap();
        assert!(
            !device.last_frame().draws().is_empty(),
            "actual class mode={mode}"
        );
        fixture.scene.view.remove();
    }
}

#[test]
fn concrete_textured_shape_mapping_errors_identify_received_values() {
    let driver =
        Graphic3dRenderDriver::with_backend(recording_device::RecordingDevice::default(), 128)
            .unwrap();
    for mode in ["map-invalid", "map-8"] {
        let failure = textured_scene::AisTexturedShapeScene::with_mode(&driver, mode)
            .err()
            .unwrap();
        assert!(failure.to_string().contains(mode));
        assert!(failure.to_string().contains("map-0 through map-7"));
    }
}

#[test]
fn concrete_textured_meshed_face_retains_real_boundary_groups() {
    let driver =
        Graphic3dRenderDriver::with_backend(recording_device::RecordingDevice::default(), 128)
            .unwrap();
    let fixture =
        textured_scene::AisTexturedShapeScene::with_mode(&driver, "face-boundaries-meshed")
            .unwrap();
    let presentation = fixture
        .manager
        .presentation(&fixture.object.cast(), 3, false, &Handle::default())
        .unwrap();
    // Native meshing provides real boundary polygons; this must not pass by emitting only shading.
    assert!(presentation.groups().len() >= 2);
    fixture.scene.view.remove();
}
