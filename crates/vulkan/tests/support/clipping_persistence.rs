//! Clipping must classify persistent anchors before disabling global fragment planes.
use super::{clip_plane, single_plane};
use crate::scene::ViewerScene;
use Graphic3dTransModeFlags as Flags;
use rustycad_tkernel::{
    ncollection::Vec2,
    quantity::{Color, NamedColor},
    standard::Handle,
};
use rustycad_tkmath::gp::{Point3, Transform3, Vector3};
use rustycad_tkservice::{aspect::*, graphic3d::*};
pub(super) fn configure(scene: &ViewerScene, mode: &str) {
    if mode.starts_with("clip-text-") {
        configure_text(scene, mode);
        return;
    }
    if !mode.contains("pers") {
        return;
    }
    let x = if matches!(mode, "clip-pers-out" | "clip-pers-override") {
        -1.
    } else {
        1.
    };
    let zoom: Handle<dyn Graphic3dTransformPersSource> = Handle::new(
        Graphic3dTransformPers::with_point(Flags::ZOOM_PERS, &Point3::new(x, 0., 0.)).unwrap(),
    )
    .cast();
    if matches!(mode, "clip-group-pers" | "clip-nested-pers") {
        scene.group.set_transform_persistence(&zoom);
        scene
            .group
            .set_transformation(&Transform3::translation_vector(Vector3::new(2., 3., 0.)));
    } else {
        scene.structure.set_transform_persistence(&zoom);
    }
    if matches!(mode, "clip-pers-2d" | "clip-nested-pers") {
        scene.structure.set_transform_persistence(&corner());
    }
    configure_local(scene, mode);
}
fn corner() -> Handle<dyn Graphic3dTransformPersSource> {
    Handle::new(
        Graphic3dTransformPers::with_corner(
            Flags::TMF_2D,
            AspectTypeOfTriedronPosition(
                AspectTypeOfTriedronPosition::LEFT.0 | AspectTypeOfTriedronPosition::BOTTOM.0,
            ),
            Vec2::new(18, 18),
        )
        .unwrap(),
    )
    .cast()
}
fn configure_local(scene: &ViewerScene, mode: &str) {
    if matches!(
        mode,
        "clip-pers-local" | "clip-pers-override" | "clip-nested-pers"
    ) {
        let local = single_plane(clip_plane(0., 1., 0.));
        local.set_override_global(mode == "clip-pers-override");
        scene.structure.set_clip_planes(&local);
    }
}
fn configure_text(scene: &ViewerScene, mode: &str) {
    let style = Handle::new(Graphic3dAspectText3d::new(
        Color::named(NamedColor::WHITE),
        Some("DejaVu Sans"),
        1.,
        0.,
        AspectTypeOfStyleText::Normal,
        AspectTypeOfDisplayText::Normal,
    ));
    let text = Handle::new(Graphic3dText::new(16.));
    text.set_text_str("AV");
    text.set_position(&Point3::new(-5., 0., 1.));
    scene.group.set_primitives_aspect(&style.cast());
    scene.group.add_text(&text, true);
    if mode == "clip-text-baseline" {
        scene.view.set_clip_planes(&Handle::default());
    } else if mode != "clip-text-cut" {
        scene
            .view
            .set_clip_planes(&single_plane(clip_plane(1., 0., -20.)));
    }
    if mode == "clip-text-no-box" {
        scene.structure.cstructure().set_bnd_box_clip_check(false);
    }
}
