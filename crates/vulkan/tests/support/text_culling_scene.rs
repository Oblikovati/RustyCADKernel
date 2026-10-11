//! Real-font culling workloads keep plane, view override and mirrored structure cases distinct.
use super::text_scene::TextScene;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::{
    gp::{Axis2, Direction3, Point3, Transform3},
    toploc::Datum3,
};
use rustycad_tkservice::graphic3d::*;

pub const CULLING_MODES: [&str; 19] = [
    "front-back",
    "back-back",
    "front-front",
    "back-front",
    "front-double",
    "back-double",
    "billboard-back",
    "billboard-front",
    "own-front-back",
    "own-back-back",
    "mirror-front-back",
    "mirror-back-back",
    "view-double-back",
    "view-back-double",
    "view-front-double",
    "auto-closed-opaque-back",
    "auto-closed-mask-back",
    "auto-closed-maskblend-back",
    "auto-open-opaque-back",
];
pub struct TextCullingScene {
    pub text: TextScene,
}
impl TextCullingScene {
    /// Retains a real-font native-matched scene; `TextCullingScene::new(&driver, "front-back")`.
    pub fn new(driver: &Handle<Graphic3dRenderDriver>, mode: &str) -> Self {
        let text = TextScene::new(driver);
        text.text.set_position(&Point3::new(0., 0., 0.));
        text.text
            .set_horizontal_alignment(Graphic3dHorizontalTextAlignment::Center);
        text.text
            .set_vertical_alignment(Graphic3dVerticalTextAlignment::Center);
        configure_culling_plane(&text, mode);
        configure_culling_policy(&text, mode);
        text.apply("text-basic");
        configure_culling_mirror(&text, mode);
        Self { text }
    }
}
fn configure_culling_plane(scene: &TextScene, mode: &str) {
    if mode.starts_with("billboard") {
        return;
    }
    let back = mode.starts_with("back")
        || mode.contains("-back-")
        || mode.starts_with("auto")
        || mode == "view-double-back"
        || mode == "view-back-double";
    let normal = Direction3::new(0., 0., if back { -1. } else { 1. }).unwrap();
    let axis = Axis2::with_x_direction(
        Point3::new(0., 0., 0.),
        normal,
        Direction3::new(1., 0., 0.).unwrap(),
    )
    .unwrap();
    scene.text.set_orientation(&axis);
    scene.text.set_own_anchor_point(mode.starts_with("own"));
}

fn configure_culling_mirror(scene: &TextScene, mode: &str) {
    if mode.starts_with("mirror") {
        let reflection = Transform3::mirror_point(Point3::new(0., 0., 0.));
        scene
            .scene
            .structure
            .set_transformation(&Handle::new(Datum3::new(reflection)));
    }
}
fn configure_culling_policy(scene: &TextScene, mode: &str) {
    scene.style.set_face_culling(culling_aspect_mode(mode));
    scene
        .scene
        .group
        .set_closed(mode.starts_with("auto-closed"));
    scene.style.set_alpha_mode(culling_alpha_mode(mode), 0.285);
    if mode.starts_with("view-") {
        configure_view_culling(scene, mode);
    }
}

fn culling_aspect_mode(mode: &str) -> Graphic3dTypeOfBackfacingModel {
    if mode.starts_with("auto") {
        return Graphic3dTypeOfBackfacingModel::Auto;
    }
    match mode.rsplit('-').next().unwrap() {
        "front" => Graphic3dTypeOfBackfacingModel::FrontCulled,
        "double" => Graphic3dTypeOfBackfacingModel::DoubleSided,
        _ => Graphic3dTypeOfBackfacingModel::BackCulled,
    }
}

fn culling_alpha_mode(mode: &str) -> Graphic3dAlphaMode {
    if mode.contains("opaque") {
        return Graphic3dAlphaMode::Opaque;
    }
    if mode.contains("mask-back") {
        return Graphic3dAlphaMode::Mask;
    }
    Graphic3dAlphaMode::MaskBlend
}

fn configure_view_culling(scene: &TextScene, mode: &str) {
    let face = match mode {
        "view-double-back" => Graphic3dTypeOfBackfacingModel::DoubleSided,
        "view-back-double" => Graphic3dTypeOfBackfacingModel::BackCulled,
        "view-front-double" => Graphic3dTypeOfBackfacingModel::FrontCulled,
        other => panic!("mode {other}; expected a view culling override"),
    };
    scene.scene.cview().set_backfacing_model(face);
}
