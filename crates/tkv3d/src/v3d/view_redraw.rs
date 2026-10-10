//! `V3d_View` update, redraw, invalidation and the automatic Z fit.
use super::{V3dView, V3dViewer};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::exceptions::program_error;
use rustycad_tkservice::graphic3d::{
    Graphic3dCViewComputed as _, Graphic3dCViewMembers as _, Graphic3dCViewStructures as _,
};

/// Update, redraw and invalidation.
impl V3dView {
    /// `Update`: recomputes the structures, fits Z and redraws an active view.
    /// Example: `view.update()?`.
    pub fn update(&self) -> Result<(), Failure> {
        if !self.view.is_defined() || !self.view.is_active() {
            return Ok(());
        }
        self.state.write().is_invalidated_immediate = false;
        self.view
            .update(rustycad_tkservice::graphic3d::GRAPHIC3D_ZLAYER_ID_UNKNOWN);
        self.view.compute();
        self.auto_z_fit()?;
        self.view.redraw();
        Ok(())
    }
    /// `Redraw`: fits Z and redraws, recomputing once after a lost device; `view.redraw()`.
    pub fn redraw(&self) {
        if !self.view.is_defined() || !self.view.is_active() {
            return;
        }
        self.state.write().is_invalidated_immediate = false;
        let Some(viewer) = self.viewer().get().map(V3dViewer::structure_manager) else {
            return;
        };
        let Some(manager) = viewer.get() else {
            return;
        };
        for _ in 0..2 {
            if manager.is_device_lost() {
                manager.recompute_structures();
            }
            // A failed Z fit leaves the camera range; the native void member reports nothing.
            let _ = self.auto_z_fit();
            self.view.redraw();
            if !manager.is_device_lost() {
                return;
            }
        }
    }
    /// `RedrawImmediate`: redraws the immediate layers; `view.redraw_immediate()`.
    pub fn redraw_immediate(&self) {
        if !self.view.is_defined() || !self.view.is_active() {
            return;
        }
        self.state.write().is_invalidated_immediate = false;
        self.view.redraw_immediate();
    }
    /// `Invalidate`: marks the view for a redraw; `view.invalidate()`.
    pub fn invalidate(&self) {
        if self.view.is_defined() {
            self.view.invalidate();
        }
    }
    /// `IsInvalidated`; `view.is_invalidated()`.
    pub fn is_invalidated(&self) -> bool {
        !self.view.is_defined() || self.view.is_invalidated()
    }
    /// `IsInvalidatedImmediate`; `view.is_invalidated_immediate()`.
    pub fn is_invalidated_immediate(&self) -> bool {
        self.state.read().is_invalidated_immediate
    }
    /// `InvalidateImmediate`; `view.invalidate_immediate()`.
    pub fn invalidate_immediate(&self) {
        self.state.write().is_invalidated_immediate = true;
    }
    /// `MustBeResized`: the driver view and the camera aspect follow the window.
    /// Example: `view.must_be_resized()`.
    pub fn must_be_resized(&self) {
        if !self.view.is_defined() {
            return;
        }
        self.view.resized();
        self.set_ratio();
        if self.state.read().immediate_update {
            self.redraw();
        }
    }
    /// `DoMapping`: maps the window; `view.do_mapping()`.
    pub fn do_mapping(&self) {
        if !self.view.is_defined() {
            return;
        }
        if let Some(window) = self.view.window().get() {
            window.do_mapping();
        }
    }
    /// `IsEmpty`: no displayed structure; `view.is_empty()`.
    pub fn is_empty(&self) -> bool {
        !self.view.is_defined() || self.view.number_of_displayed_structures() <= 0
    }
    /// `SetAutoZFitMode`; a scale factor <= 0 raises `Standard_ProgramError` (the native
    /// release-mode assertion). Example: `view.set_auto_z_fit_mode(true, 1.0)?`.
    pub fn set_auto_z_fit_mode(&self, is_on: bool, scale_factor: f64) -> Result<(), Failure> {
        if scale_factor <= 0.0 {
            return Err(program_error(&format!(
                "Zero or negative scale factor is not allowed: {scale_factor}"
            )));
        }
        let mut state = self.state.write();
        (state.auto_z_fit_scale_factor, state.auto_z_fit_is_on) = (scale_factor, is_on);
        Ok(())
    }
    /// `AutoZFitMode`; `view.auto_z_fit_mode()`.
    pub fn auto_z_fit_mode(&self) -> bool {
        self.state.read().auto_z_fit_is_on
    }
    /// `AutoZFitScaleFactor`; `view.auto_z_fit_scale_factor()`.
    pub fn auto_z_fit_scale_factor(&self) -> f64 {
        self.state.read().auto_z_fit_scale_factor
    }
    /// `AutoZFit`: `ZFitAll` with the scale factor when the mode is on; `view.auto_z_fit()?`.
    pub fn auto_z_fit(&self) -> Result<(), Failure> {
        let (on, factor) = (self.auto_z_fit_mode(), self.auto_z_fit_scale_factor());
        if on {
            self.z_fit_all(factor)?;
        }
        Ok(())
    }
    /// `ZFitAll`: fits the camera Z range to the displayed bounds; `view.z_fit_all(1.0)?`.
    pub fn z_fit_all(&self, scale_factor: f64) -> Result<(), Failure> {
        let (min_max, graphic) = self.view.z_fit_all_bounds()?;
        self.camera().z_fit_all(scale_factor, &min_max, &graphic)?;
        Ok(())
    }
}
