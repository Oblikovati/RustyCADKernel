//! Real point/spot cap sources exercise the section's homogeneous vertices.
use super::scene::ViewerScene;
use rustycad_tkernel::{
    quantity::{Color, ColorSpace},
    standard::Handle,
};
use rustycad_tkservice::graphic3d::*;

pub fn fixed_cap_light_source(scene: &ViewerScene, mode: &str) {
    if !mode.contains("source-") {
        return;
    }
    let kind = if mode.contains("source-point") {
        Graphic3dTypeOfLightSource::Positional
    } else if mode.contains("source-spot") {
        Graphic3dTypeOfLightSource::Spot
    } else {
        Graphic3dTypeOfLightSource::Directional
    };
    let light = Handle::new(Graphic3dCLight::new(kind));
    light.set_color(Color::new(0.75, 0.5, 0.25, ColorSpace::Rgb).unwrap());
    configure_fixed_cap_source(&light, kind);
    scene.cview().lights().add(&light).unwrap();
}
fn configure_fixed_cap_source(light: &Graphic3dCLight, kind: Graphic3dTypeOfLightSource) {
    if kind != Graphic3dTypeOfLightSource::Positional {
        light.set_direction_coords(0., 0., -1.).unwrap();
    }
    if kind != Graphic3dTypeOfLightSource::Directional {
        light.set_position_coords(0., 0., 12.).unwrap();
        light.set_attenuation(0.5, 0.125).unwrap();
    }
    if kind == Graphic3dTypeOfLightSource::Spot {
        light.set_angle(0.75).unwrap();
        light.set_concentration(0.125).unwrap();
    }
}
