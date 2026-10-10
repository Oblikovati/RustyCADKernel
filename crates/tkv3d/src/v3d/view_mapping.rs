//! The mapping members of `V3d_View`: size, Z size, zoom and scale and the fitting;
//! the interactive members are in view_interact.rs.
use super::V3dView;
use super::view::bad_value_if;
use rustycad_tkernel::precision::confusion;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::exceptions::program_error;
use rustycad_tkernel::standard::short_real::short_real_digits;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{Axis3, Point3, Transform3, Vector3, Xyz};
use rustycad_tkservice::graphic3d::{Graphic3dCViewComputed as _, Graphic3dCamera};

/// `std::min(a, b)`: `b` only when `b < a` (NaN operands keep the native choice).
pub(crate) fn std_min(a: f64, b: f64) -> f64 {
    if b < a { b } else { a }
}

/// `std::max(a, b)`: `b` only when `a < b`.
pub(crate) fn std_max(a: f64, b: f64) -> f64 {
    if a < b { b } else { a }
}

/// One `Umin = min(U, Umin); Umax = max(U, Umax) ...` step of the projected ranges.
pub(crate) fn extend_range(range: [f64; 6], u: f64, v: f64, w: f64) -> [f64; 6] {
    [
        std_min(u, range[0]),
        std_max(u, range[1]),
        std_min(v, range[2]),
        std_max(v, range[3]),
        std_min(w, range[4]),
        std_max(w, range[5]),
    ]
}

/// The coefficient of `SetZoom` and `ZoomAtPoint`, limited so that the view stays within
/// `[Precision::Confusion, 1e12]`.
pub(crate) fn limited_zoom_coef(camera: &Graphic3dCamera, coef: f64) -> f64 {
    let dims = camera.view_dimensions();
    let (width, height) = (dims.x(), dims.y());
    let mut coef = coef;
    if width < coef * confusion() {
        coef = width / confusion();
    } else if width > coef * 1e12 {
        coef = width / 1e12;
    }
    if height < coef * confusion() {
        coef = height / confusion();
    } else if height > coef * 1e12 {
        coef = height / 1e12;
    }
    coef
}

impl V3dView {
    /// `SetCenter`: centers the view on a pixel; `view.set_center(&100, 50)?`.
    pub fn set_center(&self, xp: i32, yp: i32) -> Result<(), Failure> {
        let (xv, yv) = self.convert_to_view(xp, yp)?;
        self.translate_camera(&self.camera(), xv, yv)?;
        self.immediate_update()
    }
    /// `SetSize`: the larger view dimension; a size <= 0 raises `V3d_BadValue`.
    /// Example: `view.set_size(1000.0)?`.
    pub fn set_size(&self, size: f64) -> Result<(), Failure> {
        bad_value_if(
            size <= 0.0,
            &format!("V3d_View::SetSize, Window Size is NULL: {size}"),
        )?;
        let camera = self.camera();
        let aspect = camera.aspect();
        camera.set_scale(if aspect >= 1.0 { size / aspect } else { size });
        self.immediate_update()
    }
    /// `SetZSize`: the Z range around the distance, widened by the float precision.
    /// Example: `view.set_z_size(2000.0)?`.
    pub fn set_z_size(&self, size: f64) -> Result<(), Failure> {
        let camera = self.camera();
        let distance = camera.distance();
        let z_max = if size <= 0.0 { distance } else { size / 2.0 };
        let precision = 1.0 / 10.0f64.powf(f64::from(short_real_digits() - 1));
        let mut z_far = z_max + distance * 2.0;
        let mut z_near = -z_max + distance;
        z_near -= z_near.abs() * precision;
        z_far += z_far.abs() * precision;
        if !camera.is_orthographic() {
            if z_far < precision {
                (z_near, z_far) = (precision, precision * 2.0);
            } else if z_near < z_far.abs() * precision {
                z_near = z_far.abs() * precision;
            }
        }
        if z_far < z_near + z_far.abs() * precision {
            z_far = z_near + z_far.abs() * precision;
        }
        camera.set_z_range(z_near, z_far)?;
        if self.state.read().immediate_update {
            self.redraw();
        }
        Ok(())
    }
    /// `SetZoom`: scales the view by a coefficient (from the start camera when asked); a
    /// coefficient <= 0 raises `V3d_BadValue`. Example: `view.set_zoom(2.0, true)?`.
    pub fn set_zoom(&self, coef: f64, to_start: bool) -> Result<(), Failure> {
        bad_value_if(
            coef <= 0.0,
            &format!("V3d_View::SetZoom, bad coefficient {coef}; expected > 0"),
        )?;
        let camera = self.camera();
        if to_start {
            let mut state = self.state.write();
            (state.cam_start_op_eye, state.cam_start_op_center) = (camera.eye(), camera.center());
        }
        let coef = limited_zoom_coef(&camera, coef);
        let (eye, center) = {
            let state = self.state.read();
            (state.cam_start_op_eye, state.cam_start_op_center)
        };
        camera.set_eye(&eye)?;
        camera.set_center(&center)?;
        camera.set_scale(camera.scale() / coef);
        self.immediate_update()
    }
    /// `SetScale`: the scale relative to the default camera; a coefficient <= 0 raises
    /// `V3d_BadValue`. Example: `view.set_scale(2.0)?`.
    pub fn set_scale(&self, coef: f64) -> Result<(), Failure> {
        bad_value_if(
            coef <= 0.0,
            &format!("V3d_View::SetScale, bad coefficient {coef}; expected > 0"),
        )?;
        let camera = self.camera();
        let default_camera = self.default_camera();
        let default_scale = default_camera.scale();
        camera.set_aspect(default_camera.aspect());
        camera.set_scale(default_scale / coef);
        self.immediate_update()
    }
    /// `SetAxialScale`; a coefficient <= 0 raises `V3d_BadValue`. `view.set_axial_scale(1.0, 2.0, 1.0)?`.
    pub fn set_axial_scale(&self, sx: f64, sy: f64, sz: f64) -> Result<(), Failure> {
        bad_value_if(
            sx <= 0.0 || sy <= 0.0 || sz <= 0.0,
            &format!("V3d_View::SetAxialScale, bad coefficient ({sx}, {sy}, {sz})"),
        )?;
        self.camera().set_axial_scale(Xyz::new(sx, sy, sz))?;
        Ok(())
    }
    /// `SetRatio` (private): the camera aspects follow the window size.
    pub(crate) fn set_ratio(&self) {
        let window = self.window();
        let Some(window) = window.get() else {
            return;
        };
        let (width, height) = window.size();
        if width > 0 && height > 0 {
            let ratio = f64::from(width) / f64::from(height);
            self.camera().set_aspect(ratio);
            self.default_camera().set_aspect(ratio);
        }
    }
    /// `FitAll(margin, update)`: fits the displayed bounds; `view.fit_all(0.01, true)?`.
    pub fn fit_all(&self, margin: f64, to_update: bool) -> Result<(), Failure> {
        let bounds = self.view().min_max_values(false)?;
        self.fit_all_box(&bounds, margin, to_update)
    }
    /// `FitAll(box, margin, update)`; a margin outside `[0, 1)` raises `Standard_ProgramError`.
    /// Example: `view.fit_all_box(&bounds, 0.01, true)?`.
    pub fn fit_all_box(
        &self,
        bounds: &BoundingBox3,
        margin: f64,
        to_update: bool,
    ) -> Result<(), Failure> {
        if !(0.0..1.0).contains(&margin) {
            return Err(program_error(&format!(
                "Invalid margin coefficient {margin}; expected [0, 1)"
            )));
        }
        if self.view().number_of_displayed_structures() == 0 {
            return Ok(());
        }
        if !self.fit_min_max(&self.camera(), bounds, margin, 10.0 * confusion(), true)? {
            return Ok(());
        }
        if self.state.read().immediate_update || to_update {
            self.update()?;
        }
        Ok(())
    }
    /// `FitMinMax`: fits a camera to a box with a margin and the zoom persistence objects.
    /// Example: `view.fit_min_max(&camera, &bounds, 0.01, 0.0, true)?`.
    pub fn fit_min_max(
        &self,
        camera: &Graphic3dCamera,
        bounds: &BoundingBox3,
        margin: f64,
        resolution: f64,
        to_enlarge_if_line: bool,
    ) -> Result<bool, Failure> {
        if !camera.fit_min_max(bounds, resolution, to_enlarge_if_line)? {
            return Ok(false);
        }
        let zoom_coef = self.view().consider_zoom_persistence_objects()?;
        let dims = camera.view_dimensions();
        self.scale_camera(
            camera,
            dims.x() * (zoom_coef + margin),
            dims.y() * (zoom_coef + margin),
        )?;
        Ok(true)
    }
    /// `Scale(camera, SizeXv, SizeYv)` (protected): sets the camera scale to show a view size.
    /// Example: `view.scale_camera(&camera, 200.0, 100.0)?`.
    pub fn scale_camera(
        &self,
        camera: &Graphic3dCamera,
        size_xv: f64,
        size_yv: f64,
    ) -> Result<(), Failure> {
        let aspect = camera.aspect();
        let scale = if aspect > 1.0 {
            std_max(size_xv / aspect, size_yv)
        } else {
            std_max(size_xv, size_yv * aspect)
        };
        camera.set_scale(scale);
        self.invalidate();
        Ok(())
    }
    /// `Translate(camera, DXv, DYv)` (protected): pans a camera by view distances.
    /// Example: `view.translate_camera(&camera, 10.0, 0.0)?`.
    pub fn translate_camera(
        &self,
        camera: &Graphic3dCamera,
        dxv: f64,
        dyv: f64,
    ) -> Result<(), Failure> {
        let (center, direction, up) = (camera.center(), camera.direction(), camera.up());
        let camera_cs = Axis3::new(center, direction.reversed(), direction.crossed(up)?)?;
        let pan = Vector3::from(camera_cs.x_direction()) * dxv
            + Vector3::from(camera_cs.y_direction()) * dyv;
        let mut pan_trsf = Transform3::default();
        pan_trsf.set_translation(pan);
        camera.transform(&pan_trsf);
        self.invalidate();
        Ok(())
    }
    /// `DepthFitAll`: Z size and depth that hold the projected bounds.
    /// Example: `view.depth_fit_all(0.01, 0.01)?`.
    pub fn depth_fit_all(&self, aspect: f64, margin: f64) -> Result<(), Failure> {
        let count = self.view().number_of_displayed_structures();
        if count <= 0 || aspect < 0.0 || margin < 0.0 || margin > 1.0 {
            return self.immediate_update();
        }
        let bounds = self.view().min_max_values(false)?;
        if bounds.is_void() {
            return self.immediate_update();
        }
        let [xmin, ymin, zmin, xmax, ymax, zmax] = bounds.get()?;
        let corners = [
            (xmin, ymin, zmax),
            (xmax, ymin, zmax),
            (xmax, ymin, zmin),
            (xmax, ymax, zmin),
            (xmin, ymax, zmax),
            (xmin, ymax, zmin),
        ];
        let (u, v, w) = self.project3(xmin, ymin, zmin);
        let (u1, v1, w1) = self.project3(xmax, ymax, zmax);
        let mut range = [
            std_min(u, u1),
            std_max(u, u1),
            std_min(v, v1),
            std_max(v, v1),
            std_min(w, w1),
            std_max(w, w1),
        ];
        for (x, y, z) in corners {
            let (u, v, w) = self.project3(x, y, z);
            range = extend_range(range, u, v, w);
        }
        let w_max = std_max(range[4].abs(), range[5].abs());
        let d_z = 2.0 * w_max + margin * w_max;
        let mut d_x = (range[1] - range[0]).abs();
        let mut d_y = (range[3] - range[2]).abs();
        d_x += margin * d_x;
        d_y += margin * d_y;
        let size = (d_x * d_x + d_y * d_y + d_z * d_z).sqrt();
        if size > 0.0 {
            self.set_z_size(size)?;
            self.set_depth(aspect * size / 2.0)?;
        }
        self.immediate_update()
    }
    /// `FitAll(Xmin, Ymin, Xmax, Ymax)`: fits a view rectangle keeping the aspect.
    /// Example: `view.fit_all_rect(-1.0, -1.0, 1.0, 1.0)?`.
    pub fn fit_all_rect(&self, xmin: f64, ymin: f64, xmax: f64, ymax: f64) -> Result<(), Failure> {
        let camera = self.camera();
        let aspect = camera.aspect();
        let (mut size_u, mut size_v) = ((xmax - xmin).abs(), (ymax - ymin).abs());
        if size_u / size_v >= aspect {
            size_v = size_u / aspect;
        } else {
            size_u = size_v * aspect;
        }
        self.translate_camera(&camera, (xmin + xmax) * 0.5, (ymin + ymax) * 0.5)?;
        self.scale_camera(&camera, size_u, size_v)?;
        self.immediate_update()
    }
    /// `WindowFit`: fits a pixel rectangle; `view.window_fit(10, 10, 200, 100)?`.
    pub fn window_fit(
        &self,
        min_xp: i32,
        min_yp: i32,
        max_xp: i32,
        max_yp: i32,
    ) -> Result<(), Failure> {
        let was_update_enabled = self.set_immediate_update(false);
        let camera = self.camera();
        if !camera.is_orthographic() {
            self.window_fit_perspective(&camera, [min_xp, min_yp, max_xp, max_yp])?;
        } else {
            let (x1, y1) = self.convert_to_view(min_xp, min_yp)?;
            let (x2, y2) = self.convert_to_view(max_xp, max_yp)?;
            self.fit_all_rect(x1, y1, x2, y2)?;
        }
        self.set_immediate_update(was_update_enabled);
        self.immediate_update()
    }
    fn window_fit_perspective(
        &self,
        camera: &Graphic3dCamera,
        rect: [i32; 4],
    ) -> Result<(), Failure> {
        let Some((width, height)) = self.window().get().map(|w| w.size()) else {
            return Ok(());
        };
        let depth = camera.project(&camera.center()).z();
        let [u_min, v_min, u_max, v_max] = [
            (2.0 / f64::from(width)) * f64::from(rect[0]) - 1.0,
            (2.0 / f64::from(height)) * f64::from(rect[1]) - 1.0,
            (2.0 / f64::from(width)) * f64::from(rect[2]) - 1.0,
            (2.0 / f64::from(height)) * f64::from(rect[3]) - 1.0,
        ];
        let pan_to = camera.convert_proj_to_view(&Point3::new(
            (u_min + u_max) * 0.5,
            (v_min + v_max) * 0.5,
            depth,
        ));
        let pan_from = camera.convert_proj_to_view(&Point3::new(0.0, 0.0, depth));
        let pan = Vector3::between(pan_from, pan_to);
        let bot_left = camera.convert_proj_to_view(&Point3::new(u_min, v_min, depth));
        let top_right = camera.convert_proj_to_view(&Point3::new(u_max, v_max, depth));
        let (u_size, v_size) = (top_right.x() - bot_left.x(), top_right.y() - bot_left.y());
        self.translate_camera(camera, pan.x(), -pan.y())?;
        self.scale_camera(camera, u_size, v_size)
    }
    /// `WindowFitAll`, the same as `WindowFit`; `view.window_fit_all(10, 10, 200, 100)?`.
    pub fn window_fit_all(
        &self,
        xmin: i32,
        ymin: i32,
        xmax: i32,
        ymax: i32,
    ) -> Result<(), Failure> {
        self.window_fit(xmin, ymin, xmax, ymax)
    }
}
