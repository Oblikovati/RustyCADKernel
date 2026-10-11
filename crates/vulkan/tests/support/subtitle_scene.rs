//! Real-font Subtitle fixtures share native camera, winding and display lifecycle.
use super::text_scene::TextScene;
#[path = "subtitle_occlusion.rs"]
mod subtitle_occlusion;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::{quantity::ColorRgba, standard::Handle};
use rustycad_tkmath::{
    gp::{Axis2, Direction3, Point3, Transform3},
    toploc::Datum3,
};
use rustycad_tkservice::{aspect::*, graphic3d::*};
pub const SUBTITLE_MODES: [&str; 25] = [
    "subtitle-basic",
    "subtitle-spaces",
    "subtitle-tabs",
    "subtitle-multiline",
    "subtitle-centered",
    "subtitle-right",
    "subtitle-top",
    "subtitle-bottom",
    "subtitle-zero-alpha",
    "subtitle-translucent",
    "subtitle-opaque-translucent",
    "subtitle-annotation",
    "subtitle-back-culled",
    "subtitle-right-back-culled",
    "subtitle-front-culled",
    "subtitle-mirror",
    "subtitle-view-double",
    "subtitle-offset",
    "subtitle-zoomable",
    "subtitle-resolution",
    "subtitle-empty",
    "subtitle-whitespace",
    "subtitle-zero-alpha-depth",
    "subtitle-zero-alpha-no-depth-write",
    "subtitle-annotation-depth",
];
pub struct SubtitleScene {
    pub text: TextScene,
}
impl SubtitleScene {
    /// Retain native-formatted background and glyph passes; `SubtitleScene::new(&driver, "subtitle-basic")`.
    pub fn new(driver: &Handle<Graphic3dRenderDriver>, mode: &str) -> Self {
        assert!(
            SUBTITLE_MODES.contains(&mode),
            "mode {mode}; expected SUBTITLE_MODES"
        );
        let text = TextScene::new(driver);
        text.text.set_position(&Point3::new(0., 0., 0.));
        text.style
            .set_text_display_type(AspectTypeOfDisplayText::Subtitle);
        text.style
            .set_edge_color_rgba(ColorRgba::new(1., 0., 0., 1.).unwrap());
        configure_subtitle_format(&text, mode);
        configure_subtitle_policy(&text, mode);
        text.apply("text-basic");
        configure_subtitle_mirror(&text, mode);
        subtitle_occlusion::append_subtitle_occluder(&text, mode);
        Self { text }
    }
}
fn configure_subtitle_mirror(text: &TextScene, mode: &str) {
    if mode == "subtitle-mirror" {
        let mirror = Transform3::mirror_point(Point3::new(0., 0., 0.));
        text.scene
            .structure
            .set_transformation(&Handle::new(Datum3::new(mirror)));
    }
}
fn configure_subtitle_format(scene: &TextScene, mode: &str) {
    configure_subtitle_words(scene, mode);
    configure_subtitle_alignment(scene, mode);
    if mode == "subtitle-zoomable" {
        scene.style.set_text_zoomable(true);
    }
    if mode == "subtitle-resolution" {
        scene
            .scene
            .view
            .change_rendering_params(&mut |p| p.resolution = 144);
    }
}
fn configure_subtitle_words(scene: &TextScene, mode: &str) {
    let words = match mode {
        "subtitle-spaces" => "AV   ",
        "subtitle-tabs" => "A\tV",
        "subtitle-multiline" => "AV\nA ",
        "subtitle-empty" => "",
        "subtitle-whitespace" => " \t ",
        _ => "AV",
    };
    scene.text.set_text_str(words);
}
fn configure_subtitle_alignment(scene: &TextScene, mode: &str) {
    let horizontal = match mode {
        "subtitle-centered" => Graphic3dHorizontalTextAlignment::Center,
        "subtitle-right" | "subtitle-right-back-culled" => Graphic3dHorizontalTextAlignment::Right,
        _ => Graphic3dHorizontalTextAlignment::Left,
    };
    let vertical = match mode {
        "subtitle-top" => Graphic3dVerticalTextAlignment::Top,
        _ => Graphic3dVerticalTextAlignment::Bottom,
    };
    scene.text.set_horizontal_alignment(horizontal);
    scene.text.set_vertical_alignment(vertical);
}
fn configure_subtitle_alpha(scene: &TextScene, mode: &str) {
    if mode.contains("translucent") || mode.starts_with("subtitle-zero-alpha") {
        let alpha = if mode.starts_with("subtitle-zero-alpha") {
            0.
        } else {
            0.4
        };
        scene
            .style
            .set_edge_color_rgba(ColorRgba::new(1., 0., 0., alpha).unwrap());
    }
}
fn configure_subtitle_policy(scene: &TextScene, mode: &str) {
    configure_subtitle_alpha(scene, mode);
    match mode {
        "subtitle-annotation" | "subtitle-annotation-depth" => scene
            .style
            .set_text_style(AspectTypeOfStyleText::Annotation),
        "subtitle-opaque-translucent" => scene
            .style
            .set_alpha_mode(Graphic3dAlphaMode::Opaque, 0.285),
        "subtitle-offset" => scene.style.set_polygon_offsets(1, 2., 3.),
        _ => {}
    }
    configure_subtitle_culling(scene, mode);
    configure_subtitle_plane(scene, mode);
}
fn configure_subtitle_culling(scene: &TextScene, mode: &str) {
    match mode {
        "subtitle-back-culled" | "subtitle-right-back-culled" | "subtitle-view-double" => scene
            .style
            .set_face_culling(Graphic3dTypeOfBackfacingModel::BackCulled),
        "subtitle-front-culled" => scene
            .style
            .set_face_culling(Graphic3dTypeOfBackfacingModel::FrontCulled),
        _ => {}
    }
    if mode == "subtitle-view-double" {
        scene
            .scene
            .cview()
            .set_backfacing_model(Graphic3dTypeOfBackfacingModel::DoubleSided);
    }
}
fn configure_subtitle_plane(scene: &TextScene, mode: &str) {
    if !mode.contains("culled") && mode != "subtitle-mirror" && mode != "subtitle-view-double" {
        return;
    }
    let normal = Direction3::new(
        0.,
        0.,
        if mode == "subtitle-view-double" {
            -1.
        } else {
            1.
        },
    )
    .unwrap();
    let plane = Axis2::with_x_direction(Point3::new(0., 0., 0.), normal, Direction3::X).unwrap();
    scene.text.set_orientation(&plane);
    scene.text.set_own_anchor_point(true);
}
