//! Native navigation fetch precedes camera dispatch, including failure and early-return state.
use super::{AisInteractiveContext, AisNavigationMode, AisViewController, AisWalkDelta};
use crate::v3d::V3dView;
use crate::v3d::V3dViewSource;
use rustycad_tkernel::standard::{Failure, Handle, exceptions::null_object};
use rustycad_tkmath::gp;
use rustycad_tkservice::graphic3d::{Graphic3dCViewMembers, Graphic3dCViewXr};
impl AisViewController {
    /// Apply timed navigation to the real view; `controller.handle_navigation_keys(&context, &view)?`.
    pub fn handle_navigation_keys(
        &mut self,
        _context: &Handle<AisInteractiveContext>,
        view: &Handle<dyn V3dViewSource>,
    ) -> Result<AisWalkDelta, Failure> {
        let run = if self.navigation_mode() == AisNavigationMode::FirstPersonFlight {
            3.0
        } else {
            1.0
        };
        let walk = self.fetch_navigation_keys(1.0, run)?;
        self.request_navigation_jump(&walk, view)?;
        if self.navigation_camera_is_blocked(&walk) {
            return Ok(walk);
        }
        self.apply_navigation_camera(&walk, navigation_view(view)?)?;
        Ok(walk)
    }
    fn request_navigation_jump(
        &mut self,
        walk: &AisWalkDelta,
        view: &Handle<dyn V3dViewSource>,
    ) -> Result<(), Failure> {
        if !walk.is_jumping() {
            return Ok(());
        }
        self.set_ask_next_frame(true);
        navigation_view(view)?.invalidate();
        Ok(())
    }
    fn navigation_camera_is_blocked(&mut self, walk: &AisWalkDelta) -> bool {
        if walk.is_empty() {
            if walk.is_defined() {
                self.set_ask_next_frame(true);
            }
            return true;
        }
        let orbit = &self.render_input.orbit_rotation;
        orbit.to_start || orbit.to_rotate
    }
    fn apply_navigation_camera(
        &mut self,
        walk: &AisWalkDelta,
        view: &V3dView,
    ) -> Result<(), Failure> {
        let speed = self.navigation_motion_speed(view)?;
        let camera = navigation_camera(view)?;
        let up = if self.to_lock_orbit_z_up() {
            gp::dz()
        } else {
            camera.orthogonalized_up()?
        };
        self.translate_navigation_walk(walk, view, &camera, up, speed)?;
        self.rotate_navigation_walk(walk, &camera, up)?;
        self.set_ask_next_frame(true);
        view.invalidate();
        view.view().synchronize_xr_base_to_posed_camera()
    }
    fn navigation_motion_speed(&self, view: &V3dView) -> Result<f64, Failure> {
        // Native queries bounds even for absolute walking: preserve its failure order.
        let bounds = view.view().min_max_values(false)?;
        let extent = if bounds.is_void() {
            gp::Xyz::default()
        } else {
            bounds.corner_max()?.xyz() - bounds.corner_min()?.xyz()
        };
        let diameter =
            native_navigation_max(native_navigation_max(extent.x(), extent.y()), extent.z());
        let diameter = if diameter <= gp::resolution() {
            0.001
        } else {
            diameter
        };
        if self.navigation_mode() == AisNavigationMode::FirstPersonWalk {
            return Ok(view.view().unit_factor() * f64::from(self.walk_speed_absolute()));
        }
        Ok(f64::from(self.walk_speed_relative()) * diameter)
    }
}
fn navigation_view(view: &Handle<dyn V3dViewSource>) -> Result<&dyn V3dViewSource, Failure> {
    // An empty fetch can use a null view; native dereferences only after its guards.
    view.get().ok_or_else(|| {
        null_object("navigation view=null; expected V3dView for camera motion or jumping")
    })
}
pub(super) fn navigation_camera(
    view: &V3dView,
) -> Result<Handle<rustycad_tkservice::graphic3d::Graphic3dCamera>, Failure> {
    let camera = if view.view().is_active_xr() {
        view.view().base_xr_camera()
    } else {
        view.camera()
    };
    if camera.is_null() {
        return Err(null_object(
            "navigation XR base camera=null; expected Graphic3dCamera for active XR",
        ));
    }
    Ok(camera)
}
fn native_navigation_max(first: f64, second: f64) -> f64 {
    // C++ std::max retains the first operand when the second is unordered.
    if first < second { second } else { first }
}
