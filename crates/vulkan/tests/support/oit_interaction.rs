//! OIT uses the same retained texture and lighting contracts as ordinary surface fixtures.
use super::super::{scene::ViewerScene, surface_texture_scene};
use rustycad_tkernel::quantity::ColorRgba;
use rustycad_tkservice::graphic3d::*;

pub fn configure_interaction(
    scene: &ViewerScene,
    aspects: &Graphic3dAspects,
    rank: usize,
    fixture: &str,
) {
    if fixture.starts_with("texture-") {
        let mode = fixture.strip_prefix("texture-").unwrap();
        let texture = surface_texture_scene::texture_handle("blend");
        surface_texture_scene::lighting::texture_lighting(scene, aspects);
        surface_texture_scene::modes::configure_texture_mode(&texture, aspects, mode);
        aspects.set_texture_map(texture.cast());
        aspects.set_texture_map_on(true);
        restore_transparent_palette(aspects, rank);
    }
    surface_texture_scene::texture_view::configure_texture_view(scene, fixture);
}
fn restore_transparent_palette(aspects: &Graphic3dAspects, rank: usize) {
    let color = super::oit_palette(rank, "two");
    aspects
        .set_interior_color_rgba(ColorRgba::new(color[0], color[1], color[2], color[3]).unwrap());
    let mut material = aspects.front_material();
    material.set_transparency(1. - color[3]).unwrap();
    aspects.set_front_material(material.clone());
    aspects.set_back_material(material);
    aspects.set_alpha_mode(Graphic3dAlphaMode::Blend, 0.5);
}
