//! Martensite native tracked-device state and the genuine interactive-object subclass seam.
use super::{AisInteractiveObject, AisInteractiveObjectBase};
use crate::prs_mgr::PrsMgrTypeOfPresentation3d;
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::{Handle, SharedState},
};
use rustycad_tkservice::{aspect::AspectXRTrackedDeviceRole, graphic3d::*};
pub(super) struct XrTrackedDeviceParameters {
    pub triangles: Handle<Graphic3dArrayOfTriangles>,
    pub laser_color: Color,
    pub laser_length: f32,
    pub unit_factor: f32,
    pub role: AspectXRTrackedDeviceRole,
    pub show_axes: bool,
    pub ray_group: Handle<dyn Graphic3dGroup>,
}
/// Embedded native XR state for application subclasses; `AisXrTrackedDeviceBase::new()`.
pub struct AisXrTrackedDeviceBase {
    /// Inherited interactive state retains the actual object owner and presentation callbacks.
    pub interactive: AisInteractiveObjectBase,
    pub(super) parameters: SharedState<XrTrackedDeviceParameters>,
}
impl AisXrTrackedDeviceBase {
    /// Constructs the native empty device with axes enabled; `AisXrTrackedDeviceBase::new()`.
    pub fn new() -> Self {
        Self {
            interactive: AisInteractiveObjectBase::new(PrsMgrTypeOfPresentation3d::AllView),
            parameters: SharedState::new(XrTrackedDeviceParameters::new()),
        }
    }
}
impl Default for AisXrTrackedDeviceBase {
    fn default() -> Self {
        // Native empty construction and the Rust default share the same axes policy.
        Self::new()
    }
}
impl XrTrackedDeviceParameters {
    fn new() -> Self {
        Self {
            triangles: Handle::default(),
            laser_color: Color::named(NamedColor::BLUE),
            laser_length: 0.,
            unit_factor: 1.,
            role: AspectXRTrackedDeviceRole::Other,
            show_axes: true,
            ray_group: Handle::default(),
        }
    }
}
/// A tracked device preserving inherited virtual presentation and selection dispatch.
/// Example: implement `xr_tracked_device_base()` alongside `AisInteractiveObject`.
pub trait AisXrTrackedDeviceSource: AisInteractiveObject {
    /// Borrows the same native state for base calls; `device.xr_tracked_device_base()`.
    fn xr_tracked_device_base(&self) -> &AisXrTrackedDeviceBase;
}
rustycad_tkernel::impl_transient_interface!(
    dyn AisXrTrackedDeviceSource,
    "AIS_XRTrackedDevice",
    dyn AisInteractiveObject
);
