//! The conversion and query members of `V3d_View`: pixel, view and world coordinates, the
//! projection of points, the camera values and the gravity point.
//!
//! The overloads of `Convert` get one name each: `convert_pixel_to_view` (`Convert(int)`),
//! `convert_to_view` (`Convert(Xp, Yp, Xv, Yv)`), `convert_view_to_pixel` (`Convert(double)`),
//! `convert_view_to_pixels` (`Convert(Xv, Yv, Xp, Yp)`), `convert_to_world`
//! (`Convert(Xp, Yp, X, Y, Z)`) and `convert_world_to_pixels` (`Convert(X, Y, Z, Xp, Yp)`).
use super::view_camera::screen_axis;
use super::view_mapping::{std_max, std_min};
use super::{V3dTypeOfView, V3dView, v3d_un_mapped};
use rustycad_tkernel::ncollection::Vec3;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::real_to_int;
use rustycad_tkmath::gp::{Point3, Vector3, Xyz, dx, dy, dz};
use rustycad_tkservice::graphic3d::{
    Graphic3dCStructureMembers as _, Graphic3dCViewStructures as _, Graphic3dStructureMembers as _,
    Graphic3dStructureSource,
};
use std::f64::consts::PI;

impl V3dView {
    /// The window size; `V3d_UnMapped` when the view has no window.
    fn mapped_size(&self) -> Result<(i32, i32), Failure> {
        if !self.view().is_defined() {
            return Err(v3d_un_mapped("view has no window"));
        }
        Ok(self.window().get().map_or((0, 0), |w| w.size()))
    }
    /// `Convert(Vp)`: a pixel length in view units; `view.convert_pixel_to_view(10)?`.
    pub fn convert_pixel_to_view(&self, vp: i32) -> Result<f64, Failure> {
        let (width, _) = self.mapped_size()?;
        let dims = self.camera().view_dimensions();
        Ok(dims.x() * f64::from(vp) / f64::from(width))
    }
    /// `Convert(Xp, Yp, Xv, Yv)`: a pixel in view coordinates; `view.convert_to_view(10, 20)?`.
    pub fn convert_to_view(&self, xp: i32, yp: i32) -> Result<(f64, f64), Failure> {
        let (width, height) = self.mapped_size()?;
        let point = Point3::new(
            f64::from(xp) * 2.0 / f64::from(width) - 1.0,
            f64::from(height - yp) * 2.0 / f64::from(height) - 1.0,
            0.0,
        );
        let point = self.camera().convert_proj_to_view(&point);
        Ok((point.x(), point.y()))
    }
    /// `Convert(Vv)`: a view length in pixels; `view.convert_view_to_pixel(1.5)?`.
    pub fn convert_view_to_pixel(&self, vv: f64) -> Result<i32, Failure> {
        let (width, _) = self.mapped_size()?;
        let dims = self.camera().view_dimensions();
        Ok(real_to_int(f64::from(width) * vv / dims.x()))
    }
    /// `Convert(Xv, Yv, Xp, Yp)`: view coordinates in pixels; `view.convert_view_to_pixels(0.0, 0.0)?`.
    pub fn convert_view_to_pixels(&self, xv: f64, yv: f64) -> Result<(i32, i32), Failure> {
        let (width, height) = self.mapped_size()?;
        let point = self
            .camera()
            .convert_view_to_proj(&Point3::new(xv, yv, 0.0));
        let (w, h) = (f64::from(width), f64::from(height));
        Ok((
            real_to_int((point.x() + 1.0) * w / 2.0),
            real_to_int(h - (point.y() + 1.0) * h / 2.0),
        ))
    }
    /// `Convert(Xp, Yp, X, Y, Z)`: the world point of a pixel on the near plane.
    /// Example: `view.convert_to_world(10, 20)?`.
    pub fn convert_to_world(&self, xp: i32, yp: i32) -> Result<(f64, f64, f64), Failure> {
        let (width, height) = self.mapped_size()?;
        let camera = self.camera();
        let depth = if camera.is_zero_to_one_depth() {
            0.0
        } else {
            -1.0
        };
        let point = Point3::new(
            2.0 * f64::from(xp) / f64::from(width) - 1.0,
            2.0 * f64::from(height - 1 - yp) / f64::from(height) - 1.0,
            depth,
        );
        let result = camera.un_project(&point);
        Ok((result.x(), result.y(), result.z()))
    }
    /// `ConvertWithProj`: the world point of a pixel and the normalized projection direction
    /// `[X, Y, Z, Dx, Dy, Dz]`. Example: `view.convert_with_proj(10, 20)?`.
    pub fn convert_with_proj(&self, xp: i32, yp: i32) -> Result<[f64; 6], Failure> {
        let (width, height) = self.mapped_size()?;
        let x = 2.0 * f64::from(xp) / f64::from(width) - 1.0;
        let y = 2.0 * f64::from(height - 1 - yp) / f64::from(height) - 1.0;
        let z = 2.0 * 0.0 - 1.0;
        let camera = self.camera();
        let result1 = camera.un_project(&Point3::new(x, y, z));
        let result2 = camera.un_project(&Point3::new(x, y, z - 10.0));
        let mut direction = Vec3::new(
            result1.x() - result2.x(),
            result1.y() - result2.y(),
            result1.z() - result2.z(),
        );
        direction.normalize();
        Ok([
            result1.x(),
            result1.y(),
            result1.z(),
            direction.x,
            direction.y,
            direction.z,
        ])
    }
    /// `Convert(X, Y, Z, Xp, Yp)`: the pixel of a world point; `view.convert_world_to_pixels(0.0, 0.0, 0.0)?`.
    pub fn convert_world_to_pixels(&self, x: f64, y: f64, z: f64) -> Result<(i32, i32), Failure> {
        let (width, height) = self.mapped_size()?;
        let point = self.camera().project(&Point3::new(x, y, z));
        let (w, h) = (f64::from(width), f64::from(height));
        Ok((
            real_to_int((point.x() + 1.0) * 0.5 * w),
            real_to_int(h - 1.0 - (point.y() + 1.0) * 0.5 * h),
        ))
    }
    /// `Project(X, Y, Z, Xp, Yp)`: view-plane coordinates of a point; `view.project(1.0, 2.0, 3.0)`.
    pub fn project(&self, x: f64, y: f64, z: f64) -> (f64, f64) {
        let (xp, yp, _) = self.project3(x, y, z);
        (xp, yp)
    }
    /// `Project(X, Y, Z, Xp, Yp, Zp)`: view coordinates of a point scaled by the view size.
    /// Example: `view.project3(1.0, 2.0, 3.0)`.
    pub fn project3(&self, x: f64, y: f64, z: f64) -> (f64, f64, f64) {
        let camera = self.camera();
        let dims = camera.view_dimensions();
        let (x_size, y_size, z_size) = (dims.x(), dims.y(), dims.z());
        let point = camera.project(&Point3::new(x, y, z));
        let zp = if camera.is_zero_to_one_depth() {
            point.z() * z_size
        } else {
            point.z() * z_size * 0.5
        };
        (point.x() * x_size * 0.5, point.y() * y_size * 0.5, zp)
    }
    /// `Scale`: the zoom relative to the default camera; `view.scale()`.
    pub fn scale(&self) -> f64 {
        self.default_camera().scale() / self.camera().scale()
    }
    /// `AxialScale(Sx, Sy, Sz)`; `view.axial_scale()`.
    pub fn axial_scale(&self) -> (f64, f64, f64) {
        let scale = self.camera().axial_scale();
        (scale.x(), scale.y(), scale.z())
    }
    /// `Size(Width, Height)`: the view dimensions; `view.size()`.
    pub fn size(&self) -> (f64, f64) {
        let dims = self.camera().view_dimensions();
        (dims.x(), dims.y())
    }
    /// `ZSize`: the view depth; `view.z_size()`.
    pub fn z_size(&self) -> f64 {
        self.camera().view_dimensions().z()
    }
    /// `Eye`; `view.eye()`.
    pub fn eye(&self) -> (f64, f64, f64) {
        let eye = self.camera().eye();
        (eye.x(), eye.y(), eye.z())
    }
    /// `FocalReferencePoint`, the same as `Eye`; `view.focal_reference_point()`.
    pub fn focal_reference_point(&self) -> (f64, f64, f64) {
        self.eye()
    }
    /// `ProjReferenceAxe`: the world point of a pixel and the projection direction there
    /// (towards the eye in perspective). Example: `view.proj_reference_axe(10, 20)?`.
    pub fn proj_reference_axe(&self, xpix: i32, ypix: i32) -> Result<[f64; 6], Failure> {
        let (xp, yp, zp) = self.convert_to_world(xpix, ypix)?;
        let (vx, vy, vz) = if self.view_type() == V3dTypeOfView::Perspective {
            let (xo, yo, zo) = self.focal_reference_point();
            (xo - xp, yo - yp, zo - zp)
        } else {
            self.proj()
        };
        Ok([xp, yp, zp, vx, vy, vz])
    }
    /// `Depth`: the camera distance; `view.depth()`.
    pub fn depth(&self) -> f64 {
        self.camera().distance()
    }
    /// `Proj`: the projection vector (from center to eye); `view.proj()`.
    pub fn proj(&self) -> (f64, f64, f64) {
        let direction = self.camera().direction().reversed();
        (direction.x(), direction.y(), direction.z())
    }
    /// `At`: the center; `view.at()`.
    pub fn at(&self) -> (f64, f64, f64) {
        let center = self.camera().center();
        (center.x(), center.y(), center.z())
    }
    /// `Up`; `view.up()`.
    pub fn up(&self) -> (f64, f64, f64) {
        let up = self.camera().up();
        (up.x(), up.y(), up.z())
    }
    /// `Twist`: the angle of the up direction around the view axis from the screen Y axis of
    /// Z, Y or X. Example: `view.twist()`.
    pub fn twist(&self) -> f64 {
        let camera = self.camera();
        let reference = camera.direction().reversed();
        let mut axes = screen_axis(&reference, &dz());
        if !axes.0 {
            axes = screen_axis(&reference, &dy());
        }
        if !axes.0 {
            axes = screen_axis(&reference, &dx());
        }
        let y_axis = axes.2;
        let up = camera.up();
        let p: Xyz = y_axis.xyz().crossed(up.xyz());
        let mut angle = std_max(std_min(p.modulus(), 1.0), -1.0).asin();
        if y_axis.dot(Vector3::from(up.xyz())) < 0.0 {
            angle = PI - angle;
        }
        if angle > 0.0 && angle < PI && p.dot(reference.xyz()) < 0.0 {
            angle = 2.0 * PI - angle;
        }
        angle
    }
    /// `Type`: orthographic or perspective; `view.view_type()`.
    pub fn view_type(&self) -> V3dTypeOfView {
        if self.camera().is_orthographic() {
            V3dTypeOfView::Orthographic
        } else {
            V3dTypeOfView::Perspective
        }
    }
    /// `IfWindow`: whether the driver view has a window; `view.if_window()`.
    pub fn if_window(&self) -> bool {
        self.view().is_defined()
    }
    /// `MinMax(Umin, Vmin, Umax, Vmax)` (private): the projected bounds and the structure count.
    /// Example: `view.min_max_projected()?`.
    pub fn min_max_projected(&self) -> Result<(i32, [f64; 4]), Failure> {
        let count = self.view().number_of_displayed_structures();
        if count == 0 {
            return Ok((count, [0.0; 4]));
        }
        let [xmin, ymin, zmin, xmax, ymax, zmax] = self.view().min_max_values(false)?.get()?;
        let (u0, v0, w0) = self.project3(xmin, ymin, zmin);
        let (u1, v1, w1) = self.project3(xmax, ymax, zmax);
        // The native member starts from the two corners without ordering them.
        let mut range = [u0, u1, v0, v1, w0, w1];
        let corners = [
            (xmin, ymin, zmax),
            (xmax, ymin, zmax),
            (xmax, ymin, zmin),
            (xmax, ymax, zmin),
            (xmin, ymax, zmax),
            (xmin, ymax, zmin),
        ];
        for (x, y, z) in corners {
            let (u, v, w) = self.project3(x, y, z);
            range = super::view_mapping::extend_range(range, u, v, w);
        }
        Ok((count, [range[0], range[2], range[1], range[3]]))
    }
    /// `MinMax(Xmin, ..., Zmax)` (private): the world bounds and the structure count.
    /// Example: `view.min_max_world()?`.
    pub fn min_max_world(&self) -> Result<(i32, [f64; 6]), Failure> {
        let count = self.view().number_of_displayed_structures();
        if count == 0 {
            return Ok((count, [0.0; 6]));
        }
        Ok((count, self.view().min_max_values(false)?.get()?))
    }
    /// `GravityPoint`: the mean of the visible box corners of the (highlighted) structures in
    /// the view, else of the whole bounds. Example: `view.gravity_point()?`.
    pub fn gravity_point(&self) -> Result<Point3, Failure> {
        let structures = self.view().displayed_structures();
        let has_selection = structures
            .iter()
            .filter_map(Handle::get)
            .any(|s| s.is_highlighted() && s.is_visible());
        let camera = self.camera();
        let (mut result, mut nb_points) = (Xyz::new(0.0, 0.0, 0.0), 0);
        for structure in structures.iter().filter_map(Handle::get) {
            let Some(corners) = structure_corners(structure, has_selection) else {
                continue;
            };
            for corner in corners {
                let projected = camera.project(&corner);
                if projected.x().abs() <= 1.0 && projected.y().abs() <= 1.0 {
                    result += corner.xyz();
                    nb_points += 1;
                }
            }
        }
        if nb_points == 0 {
            let bounds = self.view().min_max_values(false)?;
            if !bounds.is_void() {
                for corner in box_corners(bounds.get()?) {
                    result += corner.xyz();
                    nb_points += 1;
                }
            }
        }
        if nb_points > 0 {
            result /= f64::from(nb_points);
        }
        Ok(Point3::from(result))
    }
}

use rustycad_tkernel::standard::Handle;

/// The eight corners of `[Xmin, Ymin, Zmin, Xmax, Ymax, Zmax]` in the native order.
fn box_corners(b: [f64; 6]) -> [Point3; 8] {
    let [x0, y0, z0, x1, y1, z1] = b;
    [
        Point3::new(x0, y0, z0),
        Point3::new(x0, y0, z1),
        Point3::new(x0, y1, z0),
        Point3::new(x0, y1, z1),
        Point3::new(x1, y0, z0),
        Point3::new(x1, y0, z1),
        Point3::new(x1, y1, z0),
        Point3::new(x1, y1, z1),
    ]
}

/// The box corners of a visible, finite structure without transform persistence (and
/// highlighted when there is a selection).
fn structure_corners(
    structure: &dyn Graphic3dStructureSource,
    has_selection: bool,
) -> Option<[Point3; 8]> {
    if !structure.is_visible()
        || structure.is_infinite()
        || (has_selection && !structure.is_highlighted())
    {
        return None;
    }
    let bounds = structure.cstructure().get()?.bounding_box();
    if !bounds.is_valid() || !structure.transform_persistence().is_null() {
        return None;
    }
    let (min, max) = (bounds.corner_min(), bounds.corner_max());
    Some(box_corners([min.x, min.y, min.z, max.x, max.y, max.z]))
}
