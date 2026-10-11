//! Orbit rotation uses retained start vectors and the actual active-XR base camera.
use super::view_controller_camera_primitives::primitive_window_size;
use super::{
    AisViewController, view_controller_navigation_motion::navigation_camera,
    view_controller_rotation_math::*,
};
use crate::v3d::V3dView;
use rustycad_tkernel::{ncollection::Vec2, standard::Failure};
use rustycad_tkmath::gp::{self, Point3, Transform3, Vector3};
use rustycad_tkservice::graphic3d::{Graphic3dCViewXr, Graphic3dCamera};
use std::f64::consts::{FRAC_PI_2, PI};
impl AisViewController {
    /// Rotate around a retained gravity pivot; `controller.handle_orbit_rotation(&view, &pivot, true)?`.
    pub fn handle_orbit_rotation(
        &mut self,
        view: &V3dView,
        pivot: &Point3,
        lock_z_up: bool,
    ) -> Result<(), Failure> {
        if !self.to_allow_rotation() {
            return Ok(());
        }
        let camera = navigation_camera(view)?;
        self.retain_orbit_rotation_start(view, &camera, pivot);
        if !self.render_input.orbit_rotation.to_rotate {
            return Ok(());
        }
        self.abort_view_animation()?;
        self.apply_orbit_rotation(view, &camera, lock_z_up)
    }
    fn apply_orbit_rotation(
        &self,
        view: &V3dView,
        camera: &Graphic3dCamera,
        lock_z_up: bool,
    ) -> Result<(), Failure> {
        if lock_z_up {
            self.apply_locked_orbit_rotation(view, camera)?;
        } else {
            self.apply_free_orbit_rotation(view, camera)?;
        }
        view.invalidate();
        view.view().synchronize_xr_base_to_posed_camera()
    }
    fn retain_orbit_rotation_start(
        &mut self,
        view: &V3dView,
        camera: &Graphic3dCamera,
        pivot: &Point3,
    ) {
        if !self.render_input.orbit_rotation.to_start {
            return;
        }
        let state = &mut self.rotation_state;
        state.pivot = *pivot;
        state.start_up = camera.up();
        state.start_direction = camera.direction();
        state.start_eye = camera.eye();
        state.start_center = camera.center();
        view.invalidate();
    }
    fn apply_locked_orbit_rotation(
        &self,
        view: &V3dView,
        camera: &Graphic3dCamera,
    ) -> Result<(), Failure> {
        let (yaw, pitch) = self.locked_orbit_angles(view)?;
        let state = &self.rotation_state;
        let transform = retained_rotation_transform(
            state.pivot,
            state.start_up,
            state.start_direction,
            Vec2::new(yaw, pitch),
        )?;
        self.set_locked_orbit_camera(camera, transform)?;
        camera.orthogonalize_up()
    }
    fn set_locked_orbit_camera(
        &self,
        camera: &Graphic3dCamera,
        transform: Option<Transform3>,
    ) -> Result<(), Failure> {
        let state = &self.rotation_state;
        if let Some(transform) = transform {
            camera.set_up(&state.start_up.transformed(transform));
            let eye = transformed_orbit_point(state.pivot, state.start_eye, transform);
            let center = transformed_orbit_point(state.pivot, state.start_center, transform);
            return camera.set_eye_and_center(&eye, &center);
        }
        camera.set_up(&state.start_up);
        camera.set_eye_and_center(&state.start_eye, &state.start_center)
    }
    fn apply_free_orbit_rotation(
        &self,
        view: &V3dView,
        camera: &Graphic3dCamera,
    ) -> Result<(), Failure> {
        let state = &self.rotation_state;
        camera.set_eye_and_center(&state.start_eye, &state.start_center)?;
        camera.set_up(&state.start_up);
        camera.set_direction_from_eye(&state.start_direction);
        let transform = self.free_orbit_transform(view, camera)?;
        camera.transform(&transform);
        Ok(())
    }
}
fn transformed_orbit_point(pivot: Point3, point: Point3, transform: Transform3) -> Point3 {
    let delta = point.xyz() - pivot.xyz();
    let delta = *Vector3::new(delta.x(), delta.y(), delta.z())
        .transformed(transform)
        .xyz();
    let point = pivot.xyz() + delta;
    Point3::new(point.x(), point.y(), point.z())
}

impl AisViewController {
    fn locked_orbit_angles(&self, view: &V3dView) -> Result<(f64, f64), Failure> {
        let (width, height) = primitive_window_size(view)?;
        let rotation = &self.render_input.orbit_rotation;
        let yaw = (rotation.point_start.x - rotation.point_to.x) / f64::from(width) * FRAC_PI_2;
        let pitch = (rotation.point_to.y - rotation.point_start.y) / f64::from(height) * FRAC_PI_2;
        let state = &self.rotation_state;
        let pitch = if pitch.abs() > gp::resolution() {
            rotation_pitch_delta(state.start_direction, pitch)
        } else {
            pitch
        };
        Ok((yaw, pitch))
    }
}

impl AisViewController {
    fn free_orbit_transform(
        &self,
        view: &V3dView,
        camera: &Graphic3dCamera,
    ) -> Result<Transform3, Failure> {
        let state = &self.rotation_state;
        let (width, height) = view.size();
        let rx = f64::from(view.convert_view_to_pixel(width)?);
        let ry = f64::from(view.convert_view_to_pixel(height)?);
        let rotation = &self.render_input.orbit_rotation;
        let dx = wrap_orbit_angle((rotation.point_to.x - rotation.point_start.x) * PI / rx);
        let dy = wrap_orbit_angle((rotation.point_start.y - rotation.point_to.y) * PI / ry);
        let mut transform = Transform3::default();
        let side = camera.up().crossed(camera.direction().reversed())?;
        multiply_rotation_axis(&mut transform, state.pivot, camera.up(), -dx)?;
        multiply_rotation_axis(&mut transform, state.pivot, side, dy)?;
        Ok(transform)
    }
}
