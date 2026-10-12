//! Nonzero emission exposes the native Gouraud versus Phong texture modulation distinction.
use super::super::scene::ViewerScene;
use rustycad_tkernel::{
    quantity::{Color, ColorSpace},
    standard::Handle,
};
use rustycad_tkservice::graphic3d::*;
pub fn texture_lighting(scene: &ViewerScene, aspects: &dyn Graphic3dAspectsSource) {
    let mut material = Graphic3dMaterialAspect::new(Graphic3dNameOfMaterial::Plastified);
    material.set_material_type(Graphic3dTypeOfMaterial::Aspect);
    material.set_ambient_color(texture_rgb([0.25, 0.5, 0.75]));
    material.set_diffuse_color(texture_rgb([0.5, 0.25, 0.125]));
    material.set_specular_color(texture_rgb([0., 0., 0.]));
    material.set_emissive_color(texture_rgb([0.125, 0.25, 0.5]));
    aspects.set_front_material(material.clone());
    aspects.set_back_material(material);
    aspects.set_interior_color(texture_rgb([0.75, 0.5, 0.25]));
    let ambient = Handle::new(Graphic3dCLight::new(Graphic3dTypeOfLightSource::Ambient));
    ambient.set_color(texture_rgb([0.5, 0.25, 0.125]));
    let lights = Handle::new(Graphic3dLightSet::default());
    lights.add(&ambient).unwrap();
    scene.cview().set_lights(&lights);
}
fn texture_rgb(channels: [f64; 3]) -> Color {
    Color::new(channels[0], channels[1], channels[2], ColorSpace::Rgb).unwrap()
}
