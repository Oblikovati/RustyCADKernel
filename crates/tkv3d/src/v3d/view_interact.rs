//! The interactive members of `V3d_View`: panning, the zoom and the rotation by the mouse,
//! the focal length, the eye moves and the translations.
use super::view::bad_value_if;
use super::view_camera::screen_axis;
use super::view_mapping::limited_zoom_coef;
use super::{V3dTypeOfAxe, V3dView, v3d_bad_value};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::exceptions::program_error;
use rustycad_tkmath::gp::Point3;
use std::f64::consts::PI;

/// Interactive panning, zoom, rotation, eye moves and translations.
impl V3dView {
    /// `Panning`: pans by view distances and zooms around the start camera; a zoom factor
    /// <= 0 raises `Standard_ProgramError`. Example: `view.panning(10.0, 0.0, 1.0, true)?`.
    pub fn panning(
        &self,
        dxv: f64,
        dyv: f64,
        zoom_factor: f64,
        to_start: bool,
    ) -> Result<(), Failure> {
        if zoom_factor <= 0.0 {
            return Err(program_error(&format!(
                "Bad zoom factor {zoom_factor}; expected > 0"
            )));
        }
        let camera = self.camera();
        if to_start {
            let mut state = self.state.write();
            (
                state.cam_start_op_dir,
                state.cam_start_op_eye,
                state.cam_start_op_center,
            ) = (camera.direction(), camera.eye(), camera.center());
        }
        let was_update_enabled = self.set_immediate_update(false);
        let dims = camera.view_dimensions();
        let (dir, eye, center) = {
            let state = self.state.read();
            (
                state.cam_start_op_dir,
                state.cam_start_op_eye,
                state.cam_start_op_center,
            )
        };
        camera.set_eye_and_center(&eye, &center)?;
        camera.set_direction_from_eye(&dir);
        self.translate_camera(&camera, -dxv, -dyv)?;
        self.scale_camera(&camera, dims.x() / zoom_factor, dims.y() / zoom_factor)?;
        self.set_immediate_update(was_update_enabled);
        self.immediate_update()
    }
    /// `Pan`: pans by pixels; `view.pan(10, -5, 1.0, true)?`.
    pub fn pan(&self, dxp: i32, dyp: i32, zoom_factor: f64, to_start: bool) -> Result<(), Failure> {
        let (dxv, dyv) = (
            self.convert_pixel_to_view(dxp)?,
            self.convert_pixel_to_view(dyp)?,
        );
        self.panning(dxv, dyv, zoom_factor, to_start)
    }
    /// `Zoom`: zooms by the length of a pixel drag; `view.zoom(0, 0, 50, 0)?`.
    pub fn zoom(&self, xp1: i32, yp1: i32, xp2: i32, yp2: i32) -> Result<(), Failure> {
        let (d_x, d_y) = (xp2 - xp1, yp2 - yp1);
        if d_x == 0 && d_y == 0 {
            return Ok(());
        }
        let mut coeff = f64::from(d_x * d_x + d_y * d_y).sqrt() / 100.0 + 1.0;
        coeff = if d_x > 0 { coeff } else { 1.0 / coeff };
        self.set_zoom(coeff, true)
    }
    /// `StartZoomAtPoint`; `view.start_zoom_at_point(100, 100)`.
    pub fn start_zoom_at_point(&self, xp: i32, yp: i32) {
        let mut state = self.state.write();
        (state.zoom_at_point_x, state.zoom_at_point_y) = (xp, yp);
    }
    /// `ZoomAtPoint`: zooms by a drag keeping the start point fixed; `view.zoom_at_point(0, 0, 10, 10)?`.
    pub fn zoom_at_point(
        &self,
        start_x: i32,
        start_y: i32,
        end_x: i32,
        end_y: i32,
    ) -> Result<(), Failure> {
        let was_update_enabled = self.set_immediate_update(false);
        let dxy = f64::from((end_x + end_y) - (start_x + start_y));
        let mut d_zoom = dxy.abs() / 100.0 + 1.0;
        d_zoom = if dxy > 0.0 { d_zoom } else { 1.0 / d_zoom };
        bad_value_if(
            d_zoom <= 0.0,
            &format!("V3d_View::ZoomAtPoint, bad coefficient {d_zoom}"),
        )?;
        let camera = self.camera();
        let coef = limited_zoom_coef(&camera, d_zoom);
        let (zx, zy) = {
            let state = self.state.read();
            (state.zoom_at_point_x, state.zoom_at_point_y)
        };
        let (zoom_xv, zoom_yv) = self.convert_to_view(zx, zy)?;
        let (dxv, dyv) = (zoom_xv / coef, zoom_yv / coef);
        camera.set_scale(camera.scale() / coef);
        self.translate_camera(&camera, zoom_xv - dxv, zoom_yv - dyv)?;
        self.set_immediate_update(was_update_enabled);
        self.immediate_update()
    }
    /// `AxialScale(Dx, Dy, Axis)`: scales one axis by a drag; `view.axial_scale_drag(10, 0, V3dTypeOfAxe::X)?`.
    pub fn axial_scale_drag(&self, d_x: i32, d_y: i32, axis: V3dTypeOfAxe) -> Result<(), Failure> {
        if d_x == 0 && d_y == 0 {
            return Ok(());
        }
        let (mut sx, mut sy, mut sz) = self.axial_scale();
        let mut dscale = f64::from(d_x * d_x + d_y * d_y).sqrt() / 100.0 + 1.0;
        dscale = if d_x > 0 { dscale } else { 1.0 / dscale };
        match axis {
            V3dTypeOfAxe::X => sx = dscale,
            V3dTypeOfAxe::Y => sy = dscale,
            V3dTypeOfAxe::Z => sz = dscale,
        }
        self.set_axial_scale(sx, sy, sz)
    }
    /// `StartRotation`: the start pixel and the gravity point of an interactive rotation.
    /// Example: `view.start_rotation(100, 100, 0.0)?`.
    pub fn start_rotation(&self, x: i32, y: i32, z_rotation_threshold: f64) -> Result<(), Failure> {
        let (width, height) = self.size();
        let (rx, ry) = (
            f64::from(self.convert_view_to_pixel(width)?),
            f64::from(self.convert_view_to_pixel(height)?),
        );
        let gravity = self.gravity_point()?;
        {
            let mut state = self.state.write();
            (state.sx, state.sy, state.rx, state.ry, state.rotate_gravity) =
                (x, y, rx, ry, gravity);
        }
        self.rotate_about(0.0, 0.0, 0.0, gravity.x(), gravity.y(), gravity.z(), true)?;
        let mut z_rotation = false;
        if z_rotation_threshold > 0.0 {
            let (dx, dy) = (
                (f64::from(x) - rx / 2.0).abs(),
                (f64::from(y) - ry / 2.0).abs(),
            );
            let dd = z_rotation_threshold * (rx + ry) / 2.0;
            z_rotation = dx > dd || dy > dd;
        }
        self.state.write().z_rotation = z_rotation;
        Ok(())
    }
    /// `Rotation`: continues an interactive rotation to a pixel; `view.rotation(120, 100)?`.
    pub fn rotation(&self, x: i32, y: i32) -> Result<(), Failure> {
        let state = self.state.read().clone();
        if state.rx == 0.0 || state.ry == 0.0 {
            return self.start_rotation(x, y, 0.0);
        }
        let (sx, sy) = (f64::from(state.sx), f64::from(state.sy));
        let (mut d_x, mut d_y, mut d_z) = (0.0, 0.0, 0.0);
        if state.z_rotation {
            d_z = (f64::from(x) - state.rx / 2.0).atan2(state.ry / 2.0 - f64::from(y))
                - (sx - state.rx / 2.0).atan2(state.ry / 2.0 - sy);
        } else {
            d_x = (f64::from(x) - sx) * PI / state.rx;
            d_y = (sy - f64::from(y)) * PI / state.ry;
        }
        let g = state.rotate_gravity;
        self.rotate_about(d_x, d_y, d_z, g.x(), g.y(), g.z(), false)
    }
    /// `SetFocale`: the focal length of a perspective camera; `view.set_focale(50.0)?`.
    pub fn set_focale(&self, focale: f64) -> Result<(), Failure> {
        let camera = self.camera();
        if camera.is_orthographic() {
            return Ok(());
        }
        let fov_y_rad = (focale / (camera.distance() * 2.0)).atan();
        camera.set_fov_y(fov_y_rad * (360.0 / PI));
        self.immediate_update()
    }
    /// `Focale`: 0 for an orthographic camera; `view.focale()`.
    pub fn focale(&self) -> f64 {
        let camera = self.camera();
        if camera.is_orthographic() {
            return 0.0;
        }
        camera.distance() * 2.0 * (camera.fov_y() * PI / 360.0).tan()
    }
    /// The screen axes of a move or translation start; `V3d_BadValue` when aligned.
    fn start_screen_axes(&self) -> Result<(), Failure> {
        let camera = self.camera();
        let axes = screen_axis(&camera.direction().reversed(), &camera.up());
        {
            let mut state = self.state.write();
            (
                state.x_screen_axis,
                state.y_screen_axis,
                state.z_screen_axis,
            ) = (axes.1, axes.2, axes.3);
        }
        if !axes.0 {
            return Err(v3d_bad_value("V3d_View::Translate, alignment of Eye,At,Up"));
        }
        Ok(())
    }
    /// `Move(Dx, Dy, Dz, Start)`: moves the eye along the screen axes.
    /// Example: `view.move_eye(1.0, 0.0, 0.0, true)?`.
    pub fn move_eye(&self, d_x: f64, d_y: f64, d_z: f64, start: bool) -> Result<(), Failure> {
        let camera = self.camera();
        if start {
            self.state.write().cam_start_op_eye = camera.eye();
            self.start_screen_axes()?;
        }
        let state = self.state.read().clone();
        camera.set_eye(&state.cam_start_op_eye)?;
        let shifted = camera.eye().xyz()
            + d_x * *state.x_screen_axis.xyz()
            + d_y * *state.y_screen_axis.xyz()
            + d_z * *state.z_screen_axis.xyz();
        camera.set_eye(&Point3::from(shifted))?;
        self.immediate_update()
    }
    /// `Move(Length, Start)`: moves the eye along the default view axis.
    /// Example: `view.move_eye_length(10.0, true)?`.
    pub fn move_eye_length(&self, length: f64, start: bool) -> Result<(), Failure> {
        let camera = self.camera();
        if start {
            self.state.write().cam_start_op_eye = camera.eye();
        }
        let state = self.state.read().clone();
        camera.set_eye(&state.cam_start_op_eye)?;
        camera.set_eye(&Point3::from(
            camera.eye().xyz() + length * state.default_view_axis.xyz(),
        ))?;
        self.immediate_update()
    }
    /// `Move(Axe, Length, Start)`; `view.move_axe(V3dTypeOfAxe::Z, 5.0, true)?`.
    pub fn move_axe(&self, axe: V3dTypeOfAxe, length: f64, start: bool) -> Result<(), Failure> {
        match axe {
            V3dTypeOfAxe::X => self.move_eye(length, 0.0, 0.0, start),
            V3dTypeOfAxe::Y => self.move_eye(0.0, length, 0.0, start),
            V3dTypeOfAxe::Z => self.move_eye(0.0, 0.0, length, start),
        }
    }
    /// `Translate(Dx, Dy, Dz, Start)`: moves eye and center against the screen axes.
    /// Example: `view.translate(1.0, 0.0, 0.0, true)?`.
    pub fn translate(&self, d_x: f64, d_y: f64, d_z: f64, start: bool) -> Result<(), Failure> {
        let camera = self.camera();
        if start {
            {
                let mut state = self.state.write();
                (state.cam_start_op_eye, state.cam_start_op_center) =
                    (camera.eye(), camera.center());
            }
            self.start_screen_axes()?;
        }
        let state = self.state.read().clone();
        camera.set_eye(&state.cam_start_op_eye)?;
        camera.set_center(&state.cam_start_op_center)?;
        let shift = |p: Point3| {
            Point3::from(
                p.xyz()
                    - d_x * *state.x_screen_axis.xyz()
                    - d_y * *state.y_screen_axis.xyz()
                    - d_z * *state.z_screen_axis.xyz(),
            )
        };
        camera.set_center(&shift(camera.center()))?;
        camera.set_eye(&shift(camera.eye()))?;
        self.immediate_update()
    }
    /// `Translate(Axe, Length, Start)`; `view.translate_axe(V3dTypeOfAxe::X, 1.0, true)?`.
    pub fn translate_axe(
        &self,
        axe: V3dTypeOfAxe,
        length: f64,
        start: bool,
    ) -> Result<(), Failure> {
        match axe {
            V3dTypeOfAxe::X => self.translate(length, 0.0, 0.0, start),
            V3dTypeOfAxe::Y => self.translate(0.0, length, 0.0, start),
            V3dTypeOfAxe::Z => self.translate(0.0, 0.0, length, start),
        }
    }
    /// `Translate(Length, Start)`: moves the center against the default view axis.
    /// Example: `view.translate_length(10.0, true)?`.
    pub fn translate_length(&self, length: f64, start: bool) -> Result<(), Failure> {
        let camera = self.camera();
        if start {
            self.state.write().cam_start_op_center = camera.center();
        }
        let state = self.state.read().clone();
        let center =
            Point3::from(state.cam_start_op_center.xyz() - state.default_view_axis.xyz() * length);
        camera.set_center(&center)?;
        self.immediate_update()
    }
    /// `Place`: pans a pixel to the window center and zooms; `view.place(100, 100, 1.0)?`.
    pub fn place(&self, xp: i32, yp: i32, zoom_factor: f64) -> Result<(), Failure> {
        let (width, height) = self.view().window().get().map_or((0, 0), |w| w.size());
        let (center_x, center_y) = (width / 2, height / 2);
        self.pan(
            center_x - xp,
            -(center_y - yp),
            zoom_factor / self.scale(),
            true,
        )
    }
}
