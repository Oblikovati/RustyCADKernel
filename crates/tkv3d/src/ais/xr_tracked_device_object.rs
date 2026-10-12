//! Native tracked-device constructors and concrete inherited virtual dispatch.
use super::*;
use crate::{prs_mgr::*, select_mgr::*};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkservice::{graphic3d::*, image::ImageTextureSource};
/// Auxiliary tracked-device mesh and laser; `AisXrTrackedDevice::new()?`.
pub struct AisXrTrackedDevice {
    transient: StandardTransient,
    base: AisXrTrackedDeviceBase,
}
rustycad_tkernel::impl_transient!(AisXrTrackedDevice,"AIS_XRTrackedDevice",dyn AisInteractiveObject,transient,
    interfaces:[dyn AisXrTrackedDeviceSource,dyn AisInteractiveObject,dyn SelectMgrSelectableObject,dyn PrsMgrPresentableObject]);
impl AisXrTrackedDevice {
    /// Creates the native axes-only device; `AisXrTrackedDevice::new()?`.
    pub fn new() -> Result<Handle<Self>, Failure> {
        let device = Handle::new(Self {
            transient: StandardTransient::default(),
            base: AisXrTrackedDeviceBase::new(),
        });
        prs_mgr_presentable_bind(&device.clone().cast());
        Ok(device)
    }
    /// Retains model identity and delegates optional image reads; `AisXrTrackedDevice::with_render_model(&mesh,&texture)?`.
    pub fn with_render_model(
        triangles: &Handle<Graphic3dArrayOfTriangles>,
        texture: &Handle<dyn ImageTextureSource>,
    ) -> Result<Handle<Self>, Failure> {
        let device = Self::new()?;
        let mut state = device.base.parameters.write();
        state.triangles = triangles.clone();
        state.show_axes = false;
        drop(state);
        super::xr_tracked_device_texture::initialize_xr_shading(&*device, texture)?;
        Ok(device)
    }
}
impl AisXrTrackedDeviceSource for AisXrTrackedDevice {
    fn xr_tracked_device_base(&self) -> &AisXrTrackedDeviceBase {
        // Every inherited view borrows the same composite native state.
        &self.base
    }
}
impl AisInteractiveObject for AisXrTrackedDevice {
    fn interactive_base(&self) -> &AisInteractiveObjectBase {
        // Keep inherited owner/context state shared across all interface projections.
        &self.base.interactive
    }
}
impl PrsMgrPresentableObject for AisXrTrackedDevice {
    fn presentable_base(&self) -> &PrsMgrPresentableObjectBase {
        // Native protected drawer/infinite state belongs to the actual parent subobject.
        &self.base.interactive.presentable
    }
    fn accept_display_mode(&self, mode: i32) -> bool {
        // Native XR devices accept only mode zero.
        mode == 0
    }
    fn compute(
        &self,
        _: &Handle<PrsMgrPresentationManager>,
        prs: &Handle<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), Failure> {
        super::ais_xr_tracked_device_compute(self, prs, mode)
    }
    crate::ais_presentable_overrides!();
}
impl SelectMgrSelectableObject for AisXrTrackedDevice {
    fn selectable_base(&self) -> &SelectMgrSelectableObjectBase {
        // Selection inherits the same actual interactive subobject.
        &self.base.interactive.selectable
    }
    fn compute_selection(
        &self,
        selection: &Handle<SelectMgrSelection>,
        mode: i32,
    ) -> Result<(), Failure> {
        super::ais_xr_tracked_device_compute_selection(self, selection, mode)
    }
}
