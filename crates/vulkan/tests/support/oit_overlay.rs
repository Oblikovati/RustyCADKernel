//! Non-fill primitives keep ordinary native ordering and independent alpha-to-coverage state.
use super::super::scene::ViewerScene;
use rustycad_tkernel::{quantity::ColorRgba, standard::Handle};
use rustycad_tkservice::{aspect::AspectTypeOfMarker, graphic3d::*};

pub fn install_overlay(scene: &ViewerScene, fixture: &str) {
    if !matches!(fixture, "sprite-mask" | "sprite-low-mask" | "sprite-opaque" | "line-mask") {
        return;
    }
    let aspects = Handle::new(Graphic3dAspects::default());
    aspects.set_interior_color_rgba(ColorRgba::new(1., 1., 1., 0.5).unwrap());
    aspects.set_shading_model(Graphic3dTypeOfShadingModel::Unlit);
    aspects.set_marker_type(AspectTypeOfMarker::Plus);
    aspects.set_marker_scale(3.).unwrap();
    aspects.set_alpha_mode(
        if fixture == "sprite-opaque" { Graphic3dAlphaMode::Opaque } else { Graphic3dAlphaMode::Mask },
        if fixture == "sprite-low-mask" { 0.1 } else { 0.5 },
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
