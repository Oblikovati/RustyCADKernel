//! Native-facing policy is observable through ordinary retained text extraction.
#[path = "support/fixed_glyph_source.rs"]
mod fixed_glyph_source;
#[path = "support/recording_device.rs"]
mod recording_device;
#[allow(dead_code)]
#[path = "support/viewer_scene.rs"]
mod scene;
#[path = "support/text_culling_scene.rs"]
mod text_culling_scene;
#[path = "support/text_scene.rs"]
mod text_scene;
#[path = "support/view_setup.rs"]
mod view_setup;
use fixed_glyph_source::FixedGlyphSource;
use martensite_vulkan::*;
use recording_device::RecordingDevice;
use rustycad_tkservice::graphic3d::{Graphic3dAspectsMembers, Graphic3dStructureMembers};
use std::sync::Arc;

#[test]
fn culling_modes_keep_submitted_text_and_use_retained_live_aspects() {
    let device = RecordingDevice::default();
    let driver = Graphic3dRenderDriver::with_backend_and_fonts(
        device.clone(),
        256,
        Arc::new(FixedGlyphSource::default()),
    )
    .unwrap();
    for mode in text_culling_scene::CULLING_MODES {
        check_retained_culling_mode(&driver, &device, mode);
    }
}

fn check_retained_culling_mode(
    driver: &rustycad_tkernel::standard::Handle<Graphic3dRenderDriver>,
    device: &RecordingDevice,
    mode: &str,
) {
    let fixture = text_culling_scene::TextCullingScene::new(driver, mode);
    fixture.text.scene.render_view().try_redraw().unwrap();
    assert_eq!(device.last_frame().draws().len(), 1, "{mode}");
    fixture.text.style.set_face_culling(
        rustycad_tkservice::graphic3d::Graphic3dTypeOfBackfacingModel::DoubleSided,
    );
    fixture.text.scene.render_view().try_redraw().unwrap();
    assert_live_culling(device, mode);
    fixture.text.scene.view.remove();
}

fn assert_live_culling(device: &RecordingDevice, mode: &str) {
    let expected = match mode {
        "view-back-double" => RenderFaceCulling::Back,
        "view-front-double" => RenderFaceCulling::Front,
        _ => RenderFaceCulling::None,
    };
    assert_eq!(
        device.last_frame().draws()[0].face_culling(),
        expected,
        "{mode}"
    );
}

#[test]
fn displayed_empty_mirror_before_text_preserves_native_culling_lifecycle() {
    let device = RecordingDevice::default();
    let driver = Graphic3dRenderDriver::with_backend_and_fonts(
        device.clone(),
        256,
        Arc::new(FixedGlyphSource::default()),
    )
    .unwrap();
    let fixture = text_culling_scene::TextCullingScene::new(&driver, "back-back");
    fixture.text.scene.group.clear(true);
    mirror_empty_displayed_group(&fixture);
    fixture.text.apply("text-basic");
    fixture.text.scene.render_view().try_redraw().unwrap();
    assert!(device.last_frame().draws().is_empty());
    fixture.text.scene.view.remove();
}

fn mirror_empty_displayed_group(fixture: &text_culling_scene::TextCullingScene) {
    let reflection =
        rustycad_tkmath::gp::Transform3::mirror_point(rustycad_tkmath::gp::Point3::new(0., 0., 0.));
    fixture
        .text
        .scene
        .structure
        .set_transformation(&rustycad_tkernel::standard::Handle::new(
            rustycad_tkmath::toploc::Datum3::new(reflection),
        ));
}
