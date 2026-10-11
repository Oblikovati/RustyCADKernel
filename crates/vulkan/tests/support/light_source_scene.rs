//! Shared real Martensite light scene for functional captures and completed-frame timings.
#[allow(dead_code)]
#[path = "view_setup.rs"]
pub(crate) mod view_setup;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::Handle,
};
use rustycad_tkmath::gp::{Direction3, Point3};
use rustycad_tkservice::graphic3d::*;
use rustycad_tkv3d::{ais::*, v3d::*};
/// Configure a real AIS light capture view; `light_view(&driver)`.
pub fn light_view(driver: &Handle<Graphic3dRenderDriver>) -> (Handle<V3dViewer>, Handle<V3dView>) {
    let viewer = V3dViewer::new(&driver.cast()).unwrap();
    let view = viewer.create_view().unwrap();
    view_setup::configure_view(&view);
    view.camera()
        .set_eye(&Point3::new(10.0, 8.0, 20.0))
        .unwrap();
    (viewer, view)
}
/// Create native light geometry for a rendering case; `rendered_light(0)?`.
pub fn rendered_light(kind: usize) -> Result<Handle<AisLightSource>, Box<dyn std::error::Error>> {
    let light = capture_light(kind)?;
    let object = AisLightSource::new(&light)?;
    // Geometry-only legacy captures remain available alongside the named Shadow-text probe.
    object.set_display_name(false);
    object.set_zoomable(true);
    object.set_size(6.0);
    object.set_nb_splits_arrow(8);
    object.set_nb_splits_quadric(16);
    Ok(object)
}
fn capture_light(kind: usize) -> Result<Handle<Graphic3dCLight>, Box<dyn std::error::Error>> {
    let kinds = [
        Graphic3dTypeOfLightSource::Ambient,
        Graphic3dTypeOfLightSource::Directional,
        Graphic3dTypeOfLightSource::Positional,
        Graphic3dTypeOfLightSource::Spot,
    ];
    let light = Handle::new(Graphic3dCLight::new(kinds[kind]));
    light.set_color(Color::named(NamedColor::YELLOW));
    if kind == 1 || kind == 3 {
        light.set_direction(&Direction3::Z.reversed())?;
    }
    if kind >= 2 {
        light.set_range(5.0)?;
    }
    Ok(light)
}
