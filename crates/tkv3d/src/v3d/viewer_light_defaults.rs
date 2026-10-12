//! Preserve native default light construction and its intentional local owning handles.
use super::{V3dAmbientLight, V3dDirectionalLight, V3dTypeOfOrientation, V3dViewer};
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::{Failure, Handle},
    tcollection::AsciiString,
};
use rustycad_tkservice::graphic3d::Graphic3dCLight;
pub(super) fn define_default_viewer_lights(viewer: &V3dViewer) -> Result<(), Failure> {
    let white = Color::named(NamedColor::WHITE);
    let directional = V3dDirectionalLight::new(V3dTypeOfOrientation::Zneg, white, false)?;
    directional.set_name(&AsciiString::from("headlight"));
    directional.set_headlight(true)?;
    let ambient = V3dAmbientLight::new(white);
    ambient.set_name(&AsciiString::from("amblight"));
    let directional: Handle<Graphic3dCLight> = Handle::new(directional).cast();
    let ambient: Handle<Graphic3dCLight> = Handle::new(ambient).cast();
    viewer.add_light(&directional);
    viewer.add_light(&ambient);
    viewer.set_light_on_one(&directional)?;
    viewer.set_light_on_one(&ambient)
}
