//! Actual ordinary groups bind images before a native cap reuses its retained resource.
use super::CappingTextureScene;
use rustycad_tkernel::{quantity::ColorRgba, standard::Handle};
use rustycad_tkservice::{aspect::AspectTypeOfLine, graphic3d::*};
pub fn install_cap_edge_boundary(fixture: &CappingTextureScene, mode: &str) {
    if mode.contains("source-edges") {
        let aspect = fixture.scene.group.aspects();
        aspect.set_draw_edges(true);
        if mode.contains("edges-disabled") {
            aspect.set_draw_edges(false);
        }
        aspect.set_edge_color_rgba(ColorRgba::new(0., 1., 1., 1.).unwrap());
        if mode.contains("line-offset") {
            aspect.set_polygon_offsets(2, 2., 3.);
        }
        if mode.contains("fill-offset") {
            aspect.set_polygon_offsets(1, 2., 3.);
        }
        if mode.contains("both-offset") {
            aspect.set_polygon_offsets(3, 2., 3.);
        }
        if mode.contains("dashed") {
            aspect.set_line_type(AspectTypeOfLine::Dash);
        }
    }
    if mode.contains("mapped-line") {
        append_cap_shared_line(fixture);
    }
}
fn append_cap_shared_line(fixture: &CappingTextureScene) {
    let aspect = Handle::new(Graphic3dAspects::default());
    aspect.set_shading_model(Graphic3dTypeOfShadingModel::Unlit);
    aspect.set_texture_map(fixture.texture.cast());
    aspect.set_texture_map_on(true);
    aspect.set_interior_color_rgba(ColorRgba::new(0., 1., 1., 1.).unwrap());
    fixture.scene.group.set_primitives_aspect(&aspect.cast());
    let lines = Handle::new(
        Graphic3dArrayOfPrimitives::new(Graphic3dTypeOfPrimitiveArray::Segments, 2, 0, 0, 0)
            .unwrap(),
    );
    lines.add_vertex(-8., 8., -1.).unwrap();
    lines.add_vertex(8., 8., -1.).unwrap();
    fixture.scene.group.add_primitive_array(&lines, true);
}
