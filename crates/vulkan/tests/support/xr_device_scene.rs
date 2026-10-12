//! Actual AIS_XRTrackedDevice context presentations exercise retained mesh, texture and laser draws.
use rustycad_tkv3d::v3d::V3dViewSource;
#[path = "xr_device_image.rs"]
mod image_source;
use super::view_setup;
use martensite_vulkan::{Graphic3dRenderDriver, Graphic3dRenderView};
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::{Failure, Handle},
};
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::{graphic3d::*, image::ImageTextureSource};
use rustycad_tkv3d::{
    ais::*,
    prs_mgr::{PrsMgrDisplayStatus, PrsMgrPresentableObjectMembers},
    v3d::*,
};
pub const XR_CAPTURE_MODES: [&str; 8] = [
    "axes",
    "mesh",
    "texture",
    "finite-ray",
    "infinite-ray",
    "finite-zero",
    "infinite-zero",
    "color-update",
];
pub struct XrDeviceScene {
    pub viewer: Handle<V3dViewer>,
    pub view: Handle<dyn V3dViewSource>,
    pub context: Handle<AisInteractiveContext>,
    pub device: Handle<AisXrTrackedDevice>,
}
impl XrDeviceScene {
    /// Display the actual device, not copied buffers; `XrDeviceScene::new(&driver,"texture")?`.
    pub fn new(driver: &Handle<Graphic3dRenderDriver>, mode: &str) -> Result<Self, Failure> {
        validate_xr_capture_mode(mode);
        let viewer = V3dViewer::new(&driver.cast())?;
        let view = xr_capture_view(&viewer)?;
        let context = AisInteractiveContext::new(&viewer)?;
        let device = capture_xr_device(mode)?;
        context.display_with_modes(&device.cast(), 0, 0, false, PrsMgrDisplayStatus::Displayed)?;
        Ok(Self {
            viewer,
            view,
            context,
            device,
        })
    }
    /// Apply native direct laser updates after first rendering; `scene.mutate("finite-zero")?`.
    pub fn mutate(&self, mode: &str) -> Result<(), Failure> {
        if mode.ends_with("zero") {
            self.device.set_laser_length(0.)?;
        }
        if mode == "color-update" {
            self.device.set_laser_color(Color::named(NamedColor::RED))?;
        }
        Ok(())
    }
    /// Return the genuine retained renderer view; `scene.render_view().try_redraw()?`.
    pub fn render_view(&self) -> Handle<Graphic3dRenderView> {
        let cview = self.view.get().unwrap().view();
        cview.cast()
    }
}
fn capture_xr_device(mode: &str) -> Result<Handle<AisXrTrackedDevice>, Failure> {
    let device = create_capture_device(mode)?;
    device.set_unit_factor(30.);
    device.set_infinite_state(mode.starts_with("infinite"));
    device.set_laser_color(Color::named(NamedColor::GREEN))?;
    if mode.contains("ray") || mode.contains("zero") || mode == "color-update" {
        device.set_laser_length(8.)?;
    }
    if mode != "axes" {
        configure_xr_capture_shading(&device);
    }
    Ok(device)
}
fn create_capture_device(mode: &str) -> Result<Handle<AisXrTrackedDevice>, Failure> {
    if mode == "axes" {
        return AisXrTrackedDevice::new();
    }
    let image: Handle<dyn ImageTextureSource> = if mode == "texture" {
        Handle::new(image_source::XrCaptureImage::new()).cast()
    } else {
        Handle::default()
    };
    AisXrTrackedDevice::with_render_model(&xr_capture_mesh(), &image)
}
fn configure_xr_capture_shading(device: &AisXrTrackedDevice) {
    let shading = device.attributes().shading_aspect();
    let aspects = shading.aspect();
    aspects.set_shading_model(Graphic3dTypeOfShadingModel::Unlit);
    aspects.set_polygon_offsets(0, 0., 0.);
}
fn xr_capture_mesh() -> Handle<Graphic3dArrayOfTriangles> {
    let flags = GRAPHIC3D_ARRAY_FLAGS_VERTEX_NORMAL | GRAPHIC3D_ARRAY_FLAGS_VERTEX_TEXEL;
    let mesh = Handle::new(Graphic3dArrayOfTriangles::new(4, 6, flags).unwrap());
    for ([x, y], uv) in [[-5., -5.], [5., -5.], [5., 5.], [-5., 5.]]
        .into_iter()
        .zip([[0., 0.], [1., 0.], [1., 1.], [0., 1.]])
    {
        mesh.add_vertex_normal_texel([x, y, 0.], [0., 0., 1.], uv)
            .unwrap();
    }
    for rank in [1, 2, 3, 1, 3, 4] {
        mesh.add_edge(rank).unwrap();
    }
    mesh
}

fn validate_xr_capture_mode(mode: &str) {
    assert!(
        XR_CAPTURE_MODES.contains(&mode),
        "received mode={mode}; expected XR_CAPTURE_MODES"
    );
}
fn xr_capture_view(viewer: &V3dViewer) -> Result<Handle<dyn V3dViewSource>, Failure> {
    let view = viewer.create_view()?;
    view_setup::configure_view(&view);
    // Cardinal projected rays avoid the deferred oblique raster case; broad depth keeps both driver precision gates strict.
    view.camera().set_eye_and_center(
        &Point3::new(20.125, 0.125, 20.),
        &Point3::new(0.125, 0.125, 0.),
    )?;
    view.camera().set_z_range(1., 1000.)?;
    Ok(view)
}
