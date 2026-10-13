//! Non-fill primitives keep ordinary native ordering and independent alpha-to-coverage state.
use super::super::scene::ViewerScene;
use rustycad_tkernel::{quantity::ColorRgba, standard::Handle};
use rustycad_tkservice::{aspect::AspectTypeOfMarker, graphic3d::*};

pub fn install_overlay(scene: &ViewerScene, fixture: &str) {
    if fixture.starts_with("text-") {
        install_colored_text(scene, fixture);
        return;
    }
    if !matches!(
        fixture,
        "sprite-mask" | "sprite-low-mask" | "sprite-opaque" | "line-mask"
    ) {
        return;
    }
    let aspects = Handle::new(Graphic3dAspects::default());
    aspects.set_interior_color_rgba(ColorRgba::new(1., 1., 1., 0.5).unwrap());
    aspects.set_shading_model(Graphic3dTypeOfShadingModel::Unlit);
    aspects.set_marker_type(AspectTypeOfMarker::Plus);
    aspects.set_marker_scale(3.).unwrap();
    aspects.set_alpha_mode(
        if fixture == "sprite-opaque" {
            Graphic3dAlphaMode::Opaque
        } else {
            Graphic3dAlphaMode::Mask
        },
        if fixture == "sprite-low-mask" {
            0.1
        } else {
            0.5
        },
    );
    let group = scene.structure.new_group().unwrap();
    group.set_group_primitives_aspect(&aspects.cast());
    group.add_primitive_array(&overlay_vertices(fixture), true);
}
fn overlay_vertices(fixture: &str) -> Handle<Graphic3dArrayOfPrimitives> {
    let points = fixture.starts_with("sprite-");
    let kind = if points {
        Graphic3dTypeOfPrimitiveArray::Points
    } else {
        Graphic3dTypeOfPrimitiveArray::Segments
    };
    let vertices = Handle::new(Graphic3dArrayOfPrimitives::new(kind, 2, 0, 0, 0).unwrap());
    vertices
        .add_vertex(if points { 0. } else { -6. }, 0., 4.)
        .unwrap();
    if !points {
        vertices.add_vertex(6., 0., 4.).unwrap();
    }
    vertices
}

fn install_colored_text(scene: &ViewerScene, fixture: &str) {
    let style = colored_oit_text_style(fixture);
    let text = Handle::new(Graphic3dText::new(16.));
    text.set_text_str("AV");
    text.set_position(&rustycad_tkmath::gp::Point3::new(0., 0., 4.));
    text.set_horizontal_alignment(Graphic3dHorizontalTextAlignment::Center);
    text.set_vertical_alignment(Graphic3dVerticalTextAlignment::Center);
    let group = scene.structure.new_group().unwrap();
    group.set_primitives_aspect(&style.cast());
    group.add_text(&text, true);
}
fn colored_oit_text_style(fixture: &str) -> Handle<Graphic3dAspectText3d> {
    use rustycad_tkernel::quantity::{Color, ColorSpace};
    use rustycad_tkservice::aspect::AspectTypeOfStyleText;
    let style = Handle::new(Graphic3dAspectText3d::new(
        Color::new(0.18, 0.4, 0.7, ColorSpace::Rgb).unwrap(),
        Some("DejaVu Sans"),
        1.,
        0.,
        AspectTypeOfStyleText::Normal,
        colored_oit_display_mode(fixture),
    ));
    style.set_edge_color_rgba(ColorRgba::new(0.25, 0.5, 0.75, 0.5).unwrap());
    style
}
fn colored_oit_display_mode(fixture: &str) -> rustycad_tkservice::aspect::AspectTypeOfDisplayText {
    use rustycad_tkservice::aspect::AspectTypeOfDisplayText;
    match fixture {
        "text-shadow" => AspectTypeOfDisplayText::Shadow,
        "text-subtitle" => AspectTypeOfDisplayText::Subtitle,
        "text-decal" => AspectTypeOfDisplayText::Dekale,
        "text-xor" => AspectTypeOfDisplayText::Blend,
        _ => AspectTypeOfDisplayText::Normal,
    }
}
