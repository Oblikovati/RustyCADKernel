//! Native immediate camera primitives retain requests until frame dispatch consumes them.
use super::AisViewController;
use crate::v3d::V3dView;
use rustycad_tkernel::{
    precision::is_infinite,
    standard::{Failure, exceptions::null_object},
};
use rustycad_tkmath::gp::{Axis3, Point3, Transform3, Vector3};
use rustycad_tkservice::{aspect::AspectScrollDelta, graphic3d::*};
impl AisViewController {
    /// Inspect the native finite-X anchor sentinel; `controller.has_panning_anchor_point()`.
    pub fn has_panning_anchor_point(&self) -> bool {
        // Precision::IsInfinite intentionally permits a NaN anchor like native OCCT.
        !is_infinite(self.panning_anchor.x())
    }
    /// Borrow the retained perspective pan anchor; `controller.panning_anchor_point()`.
    pub fn panning_anchor_point(&self) -> &Point3 {
        // Anchor identity survives camera gestures until explicitly replaced.
        &self.panning_anchor
    }
    /// Retain a world anchor without clearing requests; `controller.set_panning_anchor_point(&point)`.
    pub fn set_panning_anchor_point(&mut self, point: &Point3) {
        // Native setters do not validate other coordinates or schedule redraw.
        self.panning_anchor = *point;
    }
    /// Apply queued pan to the actual view camera; `controller.handle_panning(&view)?`.
    pub fn handle_panning(&mut self, view: &V3dView) -> Result<(), Failure> {
        if !self.render_input.panning.to_pan || !self.to_allow_panning() {
            return Ok(());
        }
        self.abort_view_animation()?;
        if view.camera().is_orthographic() || !self.has_panning_anchor_point() {
            let delta = self.render_input.panning.delta;
            view.pan(delta.x, delta.y, 1.0, true)?;
        } else {
            self.pan_at_retained_anchor(view)?;
        }
        invalidate_and_sync_primitive(view)
    }
    fn pan_at_retained_anchor(&self, view: &V3dView) -> Result<(), Failure> {
        let camera = view.camera();
        let (width, _) = primitive_window_size(view)?;
        let depth = (self.panning_anchor.xyz() - camera.eye().xyz()).dot(camera.direction().xyz());
        let dimensions = camera.view_dimensions_at(depth);
        let delta = self.render_input.panning.delta;
        // OCCT 3d097a0 uses viewport width for both axes in perspective anchor pan.
        translate_primitive_camera(
            &camera,
            -dimensions.x() * f64::from(delta.x) / f64::from(width),
            -dimensions.x() * f64::from(delta.y) / f64::from(width),
        )
    }
    /// Apply queued screen Z rotation; `controller.handle_z_rotate(&view)?`.
    pub fn handle_z_rotate(&mut self, view: &V3dView) -> Result<(), Failure> {
        if !self.render_input.z_rotate.to_rotate || !self.to_allow_rotation() {
            return Ok(());
        }
        self.abort_view_animation()?;
        let (width, height) = primitive_window_size(view)?;
        let x = 0.99 * f64::from(width);
        let y = 0.5 * f64::from(height);
        view.start_rotation(x as i32, y as i32, 0.4)?;
        view.rotation(
            x as i32,
            (y + self.render_input.z_rotate.angle * f64::from(height)) as i32,
        )?;
        invalidate_and_sync_primitive(view)
    }
    /// Adjust stereo focus under native strict bounds; `controller.handle_z_focus_scroll(&view, &scroll)?`.
    pub fn handle_z_focus_scroll(
        &self,
        view: &V3dView,
        scroll: &AspectScrollDelta,
    ) -> Result<(), Failure> {
        let camera = view.camera();
        if !self.to_allow_z_focus() || !camera.is_stereo() {
            return Ok(());
        }
        let focus = camera.z_focus() + if scroll.delta > 0.0 { 0.05 } else { -0.05 };
        if focus > 0.2 && focus < 2.0 {
            camera.set_z_focus(camera.z_focus_type(), focus);
            view.invalidate();
        }
        Ok(())
    }
}
pub(super) fn primitive_window_size(view: &V3dView) -> Result<(i32, i32), Failure> {
    let window = view.window();
    let window = window.get().ok_or_else(|| {
        null_object("camera primitive window=null; expected a mapped AspectWindow")
    })?;
    Ok(window.size())
}
pub(super) fn primitive_camera_frame(camera: &Graphic3dCamera) -> Result<Axis3, Failure> {
    let direction = camera.direction();
    let side = direction.crossed(camera.up())?;
    Axis3::new(camera.center(), direction.reversed(), side)
}
pub(super) fn translate_primitive_camera(
    camera: &Graphic3dCamera,
    x: f64,
    y: f64,
) -> Result<(), Failure> {
    let frame = primitive_camera_frame(camera)?;
    let displacement = frame.x_direction().xyz() * x + frame.y_direction().xyz() * y;
    let mut transform = Transform3::default();
    transform.set_translation(Vector3::new(
        displacement.x(),
        displacement.y(),
        displacement.z(),
    ));
    camera.transform(&transform);
    Ok(())
}
pub(super) fn invalidate_and_sync_primitive(view: &V3dView) -> Result<(), Failure> {
    view.invalidate();
    view.view().synchronize_xr_posed_to_base_camera()
}
