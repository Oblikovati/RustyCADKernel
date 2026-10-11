//! Decal reuses the same formatter/camera/depth fixtures as Subtitle; only decoration differs.
use super::subtitle_scene::SubtitleScene;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::{quantity::ColorRgba, standard::Handle};
use rustycad_tkmath::gp::{Axis2, Direction3, Point3};
use rustycad_tkservice::{aspect::*, graphic3d::*};
pub const DECAL_MODES: [&str; 31] = [
    "decal-basic",
    "decal-spaces",
    "decal-tabs",
    "decal-multiline",
    "decal-centered",
    "decal-right",
    "decal-top",
    "decal-bottom",
    "decal-zero-alpha",
    "decal-translucent",
    "decal-opaque-translucent",
    "decal-annotation",
    "decal-back-culled",
    "decal-right-back-culled",
    "decal-front-culled",
    "decal-mirror",
    "decal-view-double",
    "decal-offset",
    "decal-zoomable",
    "decal-resolution",
    "decal-empty",
    "decal-whitespace",
    "decal-zero-alpha-depth",
    "decal-zero-alpha-no-depth-write",
    "decal-annotation-depth",
    "decal-rotated",
    "decal-plane-own",
    "decal-plane-local",
    "decal-foreground-alpha",
    "decal-plane-local-transform",
    "decal-plane-local-mirror",
];
pub struct DecalScene {
    pub text: super::text_scene::TextScene,
}
impl DecalScene {
    /// Share native fixtures with four outline passes; `DecalScene::new(&driver, "decal-basic")`.
    pub fn new(driver: &Handle<Graphic3dRenderDriver>, mode: &str) -> Self {
        assert!(
            DECAL_MODES.contains(&mode),
            "mode {mode}; expected DECAL_MODES"
        );
        let base = decal_base_fixture(mode);
        let subtitle = SubtitleScene::new(driver, &format!("subtitle-{base}"));
        subtitle
            .text
            .style
            .set_text_display_type(AspectTypeOfDisplayText::Dekale);
        configure_decal_extension(&subtitle, mode);
        configure_decal_placement(&subtitle, mode);
        Self {
            text: subtitle.text,
        }
    }
}
fn configure_decal_extension(scene: &SubtitleScene, mode: &str) {
    match mode {
        "decal-rotated" => scene.text.style.set_text_angle(30.),
        "decal-foreground-alpha" => scene
            .text
            .style
            .set_interior_color_rgba(ColorRgba::new(1., 1., 1., 0.4).unwrap()),
        "decal-plane-own"
        | "decal-plane-local"
        | "decal-plane-local-transform"
        | "decal-plane-local-mirror" => configure_decal_plane(scene, mode),
        _ => {}
    }
}
fn configure_decal_plane(scene: &SubtitleScene, mode: &str) {
    let normal = Direction3::new(0., 1., 1.).unwrap();
    let plane = Axis2::with_x_direction(Point3::new(0., 0., 0.), normal, Direction3::X).unwrap();
    scene.text.text.set_orientation(&plane);
    scene
        .text
        .text
        .set_own_anchor_point(mode == "decal-plane-own");
}

fn decal_base_fixture(mode: &str) -> &str {
    let suffix = mode.strip_prefix("decal-").unwrap();
    let candidate = format!("subtitle-{suffix}");
    if super::subtitle_scene::SUBTITLE_MODES.contains(&candidate.as_str()) {
        return suffix;
    }
    "basic"
}
fn configure_decal_placement(scene: &SubtitleScene, mode: &str) {
    use rustycad_tkmath::{
        gp::{Transform3, Vector3},
        toploc::Datum3,
    };
    let mut transform = Transform3::default();
    match mode {
        "decal-plane-local-transform" => {
            transform.set_scale(Point3::default(), 2.).unwrap();
            transform.set_translation_part(Vector3::new(3., 2., 1.));
        }
        "decal-plane-local-mirror" => transform = Transform3::mirror_point(Point3::default()),
        _ => return,
    }
    scene
        .text
        .scene
        .structure
        .set_transformation(&Handle::new(Datum3::new(transform)));
}
