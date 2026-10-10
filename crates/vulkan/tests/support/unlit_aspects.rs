//! Opaque unlit aspects shared by direct and viewer-driven acceptance scenes.
use rustycad_tkernel::{quantity::ColorRgba, standard::Handle};
use rustycad_tkservice::graphic3d::*;
pub fn unlit_style(color: [f32; 4]) -> Handle<dyn Graphic3dAspectsSource> {
    let style = Handle::new(Graphic3dAspects::default());
    style.set_polygon_offsets(0, 0., 0.);
    style.set_shading_model(Graphic3dTypeOfShadingModel::Unlit);
    style.set_interior_color_rgba(ColorRgba::new(color[0], color[1], color[2], color[3]).unwrap());
    style.cast()
}
