//! Native XOR conversion/overlap boundaries complement the existing text family matrix.
use super::decal_scene::DecalScene;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::{
    quantity::{Color, ColorRgba, NamedColor},
    standard::Handle,
};
use rustycad_tkservice::{aspect::*, graphic3d::*};
pub const BLEND_EXTRA_MODES: [&str; 8] = [
    "blend-colored",
    "blend-colored-alpha",
    "blend-alpha-zero",
    "blend-double",
    "blend-colored-overlap",
    "blend-normal-after",
    "blend-palette",
    "blend-highlight",
];
/// Retain distinct native XOR/ordinary aspects; `blend_extra_fixture(&driver, "blend-double")` cancels glyph RGB.
pub fn blend_extra_fixture(driver: &Handle<Graphic3dRenderDriver>, mode: &str) -> DecalScene {
    assert!(
        BLEND_EXTRA_MODES.contains(&mode),
        "mode {mode}; expected BLEND_EXTRA_MODES"
    );
    let scene = DecalScene::new(driver, "decal-basic");
    let text = &scene.text;
    text.style
        .set_text_display_type(AspectTypeOfDisplayText::Blend);
    text.style.set_interior_color_rgba(blend_first_color(mode));
    configure_blend_palette(&scene, mode);
    append_blend_extras(&scene, mode);
    configure_blend_highlight(&scene, mode);
    scene
}
fn blend_first_color(mode: &str) -> ColorRgba {
    let colored = matches!(
        mode,
        "blend-colored" | "blend-colored-alpha" | "blend-colored-overlap" | "blend-palette"
    );
    let alpha = if mode == "blend-alpha-zero" {
        0.
    } else if mode == "blend-colored-alpha" {
        0.4
    } else {
        1.
    };
    if colored {
        return ColorRgba::new(0.18, 0.4, 0.7, alpha).unwrap();
    }
    ColorRgba::new(1., 1., 1., alpha).unwrap()
}
fn append_blend_extras(scene: &DecalScene, mode: &str) {
    let (display, color) = match mode {
        "blend-double" => (AspectTypeOfDisplayText::Blend, [1., 1., 1., 1.]),
        "blend-colored-overlap" => (AspectTypeOfDisplayText::Blend, [0.7, 0.2, 0.4, 1.]),
        "blend-normal-after" => (AspectTypeOfDisplayText::Normal, [0.18, 0.4, 0.7, 1.]),
        _ => return,
    };
    let style = Handle::new(Graphic3dAspectText3d::new(
        Color::named(NamedColor::WHITE),
        Some("DejaVu Sans"),
        1.,
        0.,
        AspectTypeOfStyleText::Normal,
        display,
    ));
    style.set_interior_color_rgba(ColorRgba::new(color[0], color[1], color[2], color[3]).unwrap());
    scene.text.scene.group.set_primitives_aspect(&style.cast());
    scene.text.scene.group.add_text(&scene.text.text, true);
}

fn configure_blend_highlight(scene: &DecalScene, mode: &str) {
    if mode != "blend-highlight" {
        return;
    }
    let style = Handle::new(Graphic3dPresentationAttributes::default());
    style.set_color(*ColorRgba::new(0.18, 0.4, 0.7, 1.).unwrap().rgb());
    scene.text.scene.structure.highlight(&style.cast(), true);
}

fn configure_blend_palette(scene: &DecalScene, mode: &str) {
    if mode != "blend-palette" {
        return;
    }
    let color = ColorRgba::new(0.03, 0.2, 0.5, 1.).unwrap();
    scene.text.scene.view.set_background_color(*color.rgb());
}
