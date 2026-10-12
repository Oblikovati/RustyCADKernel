//! Explicit native Set2D fixture; ordinary retained AddText remains 3D.
use super::decal_scene::DecalScene;
use martensite_vulkan::*;
use rustycad_tkernel::{ncollection::Mat4, quantity::ColorRgba};
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::graphic3d::*;
/// Build actual pixel mode with the production font source; `pixel_fixture(&scene, false)?`.
pub fn pixel_fixture(scene: &DecalScene, reuse: bool) -> Result<RenderFrame, RenderError> {
    pixel_fixture_with_source(scene, reuse, &FreeTypeTextSource::default())
}
/// Inject named font I/O for CPU replay; `pixel_fixture_with_source(&scene, false, &glyphs)?`.
pub fn pixel_fixture_with_source(
    scene: &DecalScene,
    reuse: bool,
    source: &dyn TextLayoutSource,
) -> Result<RenderFrame, RenderError> {
    configure_pixel_fixture(scene, reuse);
    let params = scene.text.scene.view.rendering_params();
    let layout = source.layout(&scene.text.text, &*scene.text.style, &params)?;
    pixel_layout_frame(scene, layout)
}
/// Preserve native no-Reset geometry while consuming live placement/colors; `pixel_cached_fixture(&scene, &fonts)?`.
pub fn pixel_cached_fixture(
    scene: &DecalScene,
    source: &dyn TextLayoutSource,
) -> Result<RenderFrame, RenderError> {
    configure_pixel_fixture(scene, false);
    let params = scene.text.scene.view.rendering_params();
    let cached = source.layout(&scene.text.text, &*scene.text.style, &params)?;
    configure_pixel_reuse(scene);
    pixel_layout_frame(scene, cached)
}
fn pixel_layout_frame(
    scene: &DecalScene,
    layout: Option<GlyphLayout>,
) -> Result<RenderFrame, RenderError> {
    let Some(layout) = layout else {
        return RenderFrame::new(128, 128, [0., 0., 1., 1.], vec![]);
    };
    let view = pixel_fixture_view(scene);
    let draws = lower_pixel_text(&scene.text.text, &*scene.text.style, &layout, &view)?;
    RenderFrame::new(128, 128, [0., 0., 1., 1.], draws)
}

fn configure_pixel_fixture(scene: &DecalScene, reuse: bool) {
    use rustycad_tkservice::aspect::AspectTypeOfDisplayText;
    scene
        .text
        .style
        .set_text_display_type(AspectTypeOfDisplayText::Blend);
    let camera = scene.text.scene.view.camera();
    camera.set_scale(128.);
    camera.set_z_range(-100., 100.).unwrap();
    scene.text.text.set_position(&Point3::new(-20., 20., 5.));
    if reuse {
        configure_pixel_reuse(scene);
    }
}
fn configure_pixel_reuse(scene: &DecalScene) {
    let text = &scene.text.text;
    text.set_text_str("V");
    text.set_position(&Point3::new(12., -8., 99.));
    text.set_horizontal_alignment(Graphic3dHorizontalTextAlignment::Right);
    scene
        .text
        .style
        .set_interior_color_rgba(ColorRgba::new(0.18, 0.4, 0.7, 1.).unwrap());
}
fn pixel_fixture_view(scene: &DecalScene) -> PixelTextView {
    let camera = scene.text.scene.view.camera();
    PixelTextView {
        projection: Mat4::from_other(&camera.projection_matrix_f()),
        world_view: Mat4::from_other(&camera.orientation_matrix_f()),
        model_world: pixel_fixture_model(scene),
        face_culling: pixel_fixture_culling(scene),
        mirrored: scene
            .text
            .scene
            .structure
            .transformation()
            .get()
            .is_some_and(|d| d.transformation().is_negative()),
    }
}

fn pixel_fixture_model(scene: &DecalScene) -> Mat4<f64> {
    let mut matrix = Mat4::<f64>::identity();
    if let Some(datum) = scene.text.scene.structure.transformation().get() {
        datum.transformation().get_mat4(&mut matrix);
    }
    Mat4::from_other(&Mat4::<f32>::from_other(&matrix))
}

fn pixel_fixture_culling(scene: &DecalScene) -> RenderFaceCulling {
    if scene.text.scene.view.back_facing_model() == Graphic3dTypeOfBackfacingModel::DoubleSided {
        return RenderFaceCulling::None;
    }
    match scene.text.style.face_culling() {
        Graphic3dTypeOfBackfacingModel::BackCulled => RenderFaceCulling::Back,
        Graphic3dTypeOfBackfacingModel::FrontCulled => RenderFaceCulling::Front,
        _ => RenderFaceCulling::None,
    }
}

/// Select an actual native reuse mode; `pixel_case_frame(&scene, "pixel-reuse-cached")?` retains its first layout.
pub fn pixel_case_frame(scene: &DecalScene, mode: &str) -> Result<RenderFrame, RenderError> {
    if mode == "pixel-reuse-cached" {
        return pixel_cached_fixture(scene, &FreeTypeTextSource::default());
    }
    if mode == "pixel-reuse" || super::decal_scene::DECAL_MODES.contains(&mode) {
        return pixel_fixture(scene, mode == "pixel-reuse");
    }
    Err(RenderError::rejected(
        mode,
        "DECAL_MODES, pixel-reuse or pixel-reuse-cached",
    ))
}
