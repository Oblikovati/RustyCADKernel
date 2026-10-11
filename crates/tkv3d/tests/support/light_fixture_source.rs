//! Shared native fixture light types preserve their native zero-direction constructor state.
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::*;
pub fn fixture_light(kind: usize) -> Handle<Graphic3dCLight> {
    let kinds = [
        Graphic3dTypeOfLightSource::Ambient,
        Graphic3dTypeOfLightSource::Directional,
        Graphic3dTypeOfLightSource::Positional,
        Graphic3dTypeOfLightSource::Spot,
    ];
    Handle::new(Graphic3dCLight::new(kinds[kind]))
}
