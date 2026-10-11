//! Paired opacity transitions exercise automatic, explicit and back material policies.
use rustycad_tkservice::graphic3d::{
    Graphic3dAlphaMode, Graphic3dAspects, Graphic3dAspectsMembers,
};
/// Configure the alpha fixture; `configure(&aspect, "lit-distinct-back")`.
pub fn configure(aspect: &Graphic3dAspects, mode: &str) {
    let mut front = aspect.front_material();
    let mut back = aspect.back_material();
    front
        .set_transparency(if mode == "lit-forced-blend" { 0. } else { 0.5 })
        .unwrap();
    back.set_transparency(0.75).unwrap();
    aspect.set_front_material(front);
    aspect.set_back_material(back);
    if mode == "lit-front-opaque" {
        aspect.set_alpha_mode(Graphic3dAlphaMode::Opaque, 0.5);
    }
    if mode == "lit-forced-blend" {
        aspect.set_alpha_mode(Graphic3dAlphaMode::Blend, 0.5);
    }
}
/// Change opacity after an initial redraw; `mutate(&aspect, "lit-alpha-pass-updated")`.
pub fn mutate(aspect: &Graphic3dAspects, mode: &str) {
    let transparency = match mode {
        "lit-alpha-opacity-updated" => 0.25,
        "lit-alpha-pass-updated" => 0.,
        _ => return,
    };
    set_front_transparency(aspect, transparency);
}
/// Keep the source aspect live; `set_front_transparency(&aspect, 0.5)`.
pub fn set_front_transparency(aspect: &Graphic3dAspects, transparency: f32) {
    let mut material = aspect.front_material();
    material.set_transparency(transparency).unwrap();
    aspect.set_front_material(material);
}
