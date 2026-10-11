//! First-person rotation retains start vectors and updates roll/buffers before animation stop.
use super::view_controller_camera_primitives::primitive_window_size;
use super::{AisViewController, view_controller_rotation_math::*};
use crate::v3d::V3dView;
use rustycad_tkernel::{ncollection::Vec2, standard::Failure};
use rustycad_tkmath::gp::{self, Direction3, Point3, Transform3};
use rustycad_tkservice::graphic3d::Graphic3dCamera;
use std::f64::consts::FRAC_PI_2;
impl AisViewController {
    /// Apply retained first-person yaw/pitch/roll; `controller.handle_view_rotation(&view, 0.1, 0.0, 0.0, true)?`.
    pub fn handle_view_rotation(
        &mut self,
        view: &V3dView,
        yaw: f64,
        pitch: f64,
        roll: f64,
        restart: bool,
    ) -> Result<(), Failure> {
        if !self.to_allow_rotation() {
            return Ok(());
        }
        let camera = view.camera();
        let transition = self.prepare_view_rotation(&camera, yaw, pitch, roll, restart);
        if !self.render_input.view_rotation.to_rotate {
            return Ok(());
        }
        self.finish_view_rotation(view, &camera, yaw, pitch, transition)
    }
    fn finish_view_rotation(
        &mut self,
        view: &V3dView,
        camera: &Graphic3dCamera,
        yaw: f64,
        pitch: f64,
        transition: Option<(f64, f64)>,
    ) -> Result<(), Failure> {
        self.abort_view_animation()?;
        self.apply_retained_view_rotation(view, camera, yaw, pitch, transition)?;
        view.invalidate();
        Ok(())
    }
    fn retain_view_rotation_start(
        &mut self,
        camera: &Graphic3dCamera,
        increment: bool,
        roll: f64,
        restart: bool,
    ) {
        if increment && restart {
            let rotation = &mut self.render_input.view_rotation;
            rotation.to_start = true;
            rotation.point_to = rotation.point_start;
        }
        if self.render_input.view_rotation.to_start {
            self.retain_view_camera_vectors(camera);
        }
        if increment {
            self.render_input.view_rotation.to_rotate = true;
            self.rotation_state.current_roll = roll;
        }
    }
    fn apply_retained_view_rotation(
        &self,
        view: &V3dView,
        camera: &Graphic3dCamera,
        yaw: f64,
        pitch: f64,
        roll: Option<(f64, f64)>,
    ) -> Result<(), Failure> {
        let (mut up, direction) = self.retained_view_directions(view, yaw, pitch)?;
        if let Some((old, new)) = roll {
            up = retained_view_roll(up, direction, old, new)?;
        }
        camera.set_up(&up);
        camera.set_direction_from_eye(&direction);
        camera.orthogonalize_up()
    }
}
fn retained_view_roll(
    mut up: Direction3,
    direction: Direction3,
    old: f64,
    new: f64,
) -> Result<Direction3, Failure> {
    let mut inverse = Transform3::default();
    multiply_rotation_axis(&mut inverse, Point3::default(), direction, -old)?;
    up = up.transformed(inverse);
    let mut roll = Transform3::default();
    multiply_rotation_axis(&mut roll, Point3::default(), direction, new)?;
    Ok(up.transformed(roll))
}

impl AisViewController {
    fn prepare_view_rotation(
        &mut self,
        camera: &Graphic3dCamera,
        yaw: f64,
        pitch: f64,
        roll: f64,
        restart: bool,
    ) -> Option<(f64, f64)> {
        let old_roll = self.rotation_state.current_roll;
        let changed = (roll - old_roll).abs() > gp::resolution();
        let increment = yaw.abs() > gp::resolution() || pitch.abs() > gp::resolution() || changed;
        self.retain_view_rotation_start(camera, increment, roll, restart);
        if changed {
            Some((old_roll, roll))
        } else {
            None
        }
    }
    fn retained_view_directions(
        &self,
        view: &V3dView,
        yaw: f64,
        pitch: f64,
    ) -> Result<(Direction3, Direction3), Failure> {
        let transform = self.retained_view_transform(view, yaw, pitch)?;
        let state = &self.rotation_state;
        let direction = transform.map_or(state.start_direction, |t| {
            state.start_direction.transformed(t)
        });
        let up = transform.map_or(state.start_up, |t| state.start_up.transformed(t));
        Ok((up, direction))
    }
}

impl AisViewController {
    fn retain_view_camera_vectors(&mut self, camera: &Graphic3dCamera) {
        self.rotation_state.start_up = camera.up();
        self.rotation_state.start_direction = camera.direction();
    }
}

impl AisViewController {
    fn retained_view_transform(
        &self,
        view: &V3dView,
        yaw: f64,
        pitch: f64,
    ) -> Result<Option<Transform3>, Failure> {
        let angles = self.view_rotation_angles(view, yaw, pitch)?;
        let state = &self.rotation_state;
        retained_rotation_transform(
            Point3::default(),
            state.start_up,
            state.start_direction,
            angles,
        )
    }
}

impl AisViewController {
    fn view_rotation_angles(
        &self,
        view: &V3dView,
        yaw: f64,
        pitch: f64,
    ) -> Result<Vec2<f64>, Failure> {
        let (width, height) = primitive_window_size(view)?;
        let rotation = &self.render_input.view_rotation;
        let yaw =
            (rotation.point_start.x - rotation.point_to.x) / f64::from(width) * FRAC_PI_2 + yaw;
        let pitch =
            -(rotation.point_start.y - rotation.point_to.y) / f64::from(height) * FRAC_PI_2 + pitch;
        let state = &self.rotation_state;
        let pitch = rotation_pitch_delta(state.start_direction, pitch);
        Ok(Vec2::new(yaw, pitch))
    }
}
