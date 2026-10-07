//! The frustum, fitting and interpolation members of `Graphic3d_Camera`.
use super::camera::DTR_HALF;
use super::{Graphic3dCamera, Graphic3dCameraProjection};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::exceptions::program_error;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{Axis1, Direction3, Plane3, Point3, Transform3, Vector3, Xyz, origin};

/// `FrustumVert_LeftBottomNear`: an index of `FrustumPoints`.
pub const GRAPHIC3D_CAMERA_FRUSTUM_VERT_LEFT_BOTTOM_NEAR: usize = 0;
/// `FrustumVert_LeftBottomFar`.
pub const GRAPHIC3D_CAMERA_FRUSTUM_VERT_LEFT_BOTTOM_FAR: usize = 1;
/// `FrustumVert_LeftTopNear`.
pub const GRAPHIC3D_CAMERA_FRUSTUM_VERT_LEFT_TOP_NEAR: usize = 2;
/// `FrustumVert_LeftTopFar`.
pub const GRAPHIC3D_CAMERA_FRUSTUM_VERT_LEFT_TOP_FAR: usize = 3;
/// `FrustumVert_RightBottomNear`.
pub const GRAPHIC3D_CAMERA_FRUSTUM_VERT_RIGHT_BOTTOM_NEAR: usize = 4;
/// `FrustumVert_RightBottomFar`.
pub const GRAPHIC3D_CAMERA_FRUSTUM_VERT_RIGHT_BOTTOM_FAR: usize = 5;
/// `FrustumVert_RightTopNear`.
pub const GRAPHIC3D_CAMERA_FRUSTUM_VERT_RIGHT_TOP_NEAR: usize = 6;
/// `FrustumVert_RightTopFar`.
pub const GRAPHIC3D_CAMERA_FRUSTUM_VERT_RIGHT_TOP_FAR: usize = 7;
/// `FrustumVerticesNB`: the number of `FrustumPoints`.
pub const GRAPHIC3D_CAMERA_FRUSTUM_VERTICES_NB: usize = 8;

/// `std::min(a, b)`: `b < a ? b : a` (NaN-propagating unlike `f64::min`).
fn std_min(a: f64, b: f64) -> f64 {
    if b < a { b } else { a }
}
/// `std::max(a, b)`: `a < b ? b : a`.
fn std_max(a: f64, b: f64) -> f64 {
    if a < b { b } else { a }
}

/// The direction of a vector (the implicit `gp_Dir(gp_Vec)` conversion).
fn dir(vector: Vector3) -> Result<Direction3, Failure> {
    Direction3::from_vector(vector)
}
fn vec(direction: Direction3) -> Vector3 {
    Vector3::from(direction.xyz())
}

impl Graphic3dCamera {
    /// `ViewDimensions(theZValue)`: the view size at a depth (the scale for orthographic),
    /// widened by the aspect, and the Z range; `camera.view_dimensions_at(100.0)`.
    pub fn view_dimensions_at(&self, z_value: f64) -> Xyz {
        self.read(|s| {
            let size = if s.proj_type == Graphic3dCameraProjection::Orthographic {
                s.scale
            } else {
                2.0 * z_value * s.fov_y_tan
            };
            let (x, y) = if s.aspect > 1.0 {
                (size * s.aspect, size)
            } else {
                (size, size / s.aspect)
            };
            Xyz::new(x, y, s.z_far - s.z_near)
        })
    }
    /// `ViewDimensions()`: at the camera distance; `camera.view_dimensions()`.
    pub fn view_dimensions(&self) -> Xyz {
        self.view_dimensions_at(self.distance())
    }
    /// The 2D offset in X of normalized device coordinates; `camera.ndc_2d_offset_x()`.
    pub fn ndc_2d_offset_x(&self) -> f64 {
        self.read(|s| {
            if s.fov_2d >= s.fov_x {
                0.5
            } else {
                0.5 * s.fov_2d / s.fov_x
            }
        })
    }
    /// The 2D offset in Y of normalized device coordinates; `camera.ndc_2d_offset_y()`.
    pub fn ndc_2d_offset_y(&self) -> f64 {
        self.read(|s| {
            if s.fov_2d >= s.fov_y {
                0.5
            } else {
                0.5 * s.fov_2d / s.fov_y
            }
        })
    }
    /// `Frustum`: the left, right, bottom, top, near and far planes (normals inside); a view
    /// direction parallel to up raises `Standard_ProgramError`. Example: `camera.frustum()?`.
    pub fn frustum(&self) -> Result<[Plane3; 6], Failure> {
        let projection = vec(self.direction());
        let up = vec(self.orthogonalized_up()?);
        let side = projection.crossed(up);
        if projection.is_parallel(up, precision::angular())? {
            return Err(program_error(
                "Can not derive SIDE = PROJ x UP - directions are parallel; got the view direction parallel to up",
            ));
        }
        let eye = self.eye();
        let near = Plane3::new(eye.translated(projection * self.z_near()), dir(projection)?)?;
        let far = Plane3::new(eye.translated(projection * self.z_far()), dir(-projection)?)?;
        let [left, right, bottom, top] = self.side_planes(side, up)?;
        Ok([left, right, bottom, top, near, far])
    }
    /// The four side planes of `Frustum`, rotated by the half fields of view in perspective.
    fn side_planes(&self, side: Vector3, up: Vector3) -> Result<[Plane3; 4], Failure> {
        let (scale, aspect, center) = (self.scale(), self.aspect(), self.center());
        let (h, v) = if aspect >= 1.0 {
            (scale * 0.5 * aspect, scale * 0.5)
        } else {
            (scale * 0.5, scale * 0.5 / aspect)
        };
        let points = [
            center.translated(-side * h),
            center.translated(side * h),
            center.translated(-up * v),
            center.translated(up * v),
        ];
        let mut dirs = [side, -side, up, -up];
        if !self.is_orthographic() {
            let fov_y = self.fov_y();
            let (hor, ver) = (((DTR_HALF * fov_y).tan() * aspect).atan(), DTR_HALF * fov_y);
            let (up_axis, side_axis) = (
                Axis1::new(origin(), dir(up)?),
                Axis1::new(origin(), dir(side)?),
            );
            dirs[0].rotate(up_axis, hor)?;
            dirs[1].rotate(up_axis, -hor)?;
            dirs[2].rotate(side_axis, -ver)?;
            dirs[3].rotate(side_axis, ver)?;
        }
        let plane = |i: usize| -> Result<Plane3, Failure> { Plane3::new(points[i], dir(dirs[i])?) };
        Ok([plane(0)?, plane(1)?, plane(2)?, plane(3)?])
    }
}

/// The eight corners of a box in the native order.
fn box_corners(min: &Point3, max: &Point3) -> [Point3; 8] {
    let (a, b) = (min, max);
    [
        Point3::new(a.x(), a.y(), a.z()),
        Point3::new(a.x(), a.y(), b.z()),
        Point3::new(a.x(), b.y(), a.z()),
        Point3::new(a.x(), b.y(), b.z()),
        Point3::new(b.x(), a.y(), a.z()),
        Point3::new(b.x(), a.y(), b.z()),
        Point3::new(b.x(), b.y(), a.z()),
        Point3::new(b.x(), b.y(), b.z()),
    ]
}

/// The distances of `FitMinMax` from the box center to each frustum plane, scaled by the
/// plane tilt, and the asymmetries in X and Y.
fn fit_distances(
    planes: &[Plane3; 6],
    axes: [Direction3; 3],
    center: &Point3,
    corners: &[Point3; 8],
) -> ([f64; 6], f64, f64) {
    let [cam_side, cam_up, cam_dir] = axes;
    let normal = |i: usize| planes[i].axis().direction();
    let tan_angle = |reference: Direction3, i: usize| reference.angle(&normal(i)).tan();
    let reference = [cam_side, -cam_side, cam_up, -cam_up, cam_dir, -cam_dir];
    let mut distances = [0.0f64; 6];
    for (i, distance) in distances.iter_mut().enumerate() {
        let n = vec(normal(i));
        for corner in corners {
            *distance = std_max(*distance, Vector3::between(*center, *corner).dot(n));
        }
    }
    for (i, distance) in distances.iter_mut().enumerate() {
        *distance *= (1.0 + tan_angle(reference[i], i).powf(2.0)).sqrt();
    }
    let assym_x = tan_angle(cam_side, 0) - tan_angle(-cam_side, 1);
    let assym_y = tan_angle(cam_up, 2) - tan_angle(-cam_up, 3);
    (distances, assym_x, assym_y)
}

impl Graphic3dCamera {
    /// `FitMinMax`: centers the camera on a box (with the axial scale) and sets the distance
    /// and scale that show it; false for a void box, a point, or a line unless
    /// `to_enlarge_if_line`. Example: `camera.fit_min_max(&bounds, 1e-7, false)?`.
    pub fn fit_min_max(
        &self,
        bounds: &BoundingBox3,
        resolution: f64,
        to_enlarge_if_line: bool,
    ) -> Result<bool, Failure> {
        if bounds.is_void() {
            return Ok(false);
        }
        let scale = self.axial_scale();
        let min = Point3::from(bounds.corner_min()?.xyz().multiplied_components(scale));
        let max = Point3::from(bounds.corner_max()?.xyz().multiplied_components(scale));
        if max.is_equal(&min, rustycad_tkernel::standard::real::real_epsilon()) {
            return Ok(false);
        }
        let planes = self.frustum()?;
        let (cam_up, cam_dir) = (self.orthogonalized_up()?, self.direction());
        let cam_side = cam_dir.crossed(cam_up)?;
        let center = Point3::from((min.xyz() + max.xyz()) / 2.0);
        let (d, assym_x, assym_y) = fit_distances(
            &planes,
            [cam_side, cam_up, cam_dir],
            &center,
            &box_corners(&min, &max),
        );
        let (size_x, size_y, size_z) = (d[0] + d[1], d[2] + d[3], d[4] + d[5]);
        let offset_x = (d[1] - d[0]) * 0.5 + assym_x * size_x * 0.5;
        let offset_y = (d[3] - d[2]) * 0.5 + assym_y * size_y * 0.5;
        let new_center = center
            .translated(vec(cam_side) * offset_x)
            .translated(vec(cam_up) * offset_y);
        self.transform(&Transform3::translation_between(self.center(), new_center));
        self.set_distance(d[5] + d[4]);
        self.fit_scale(size_x, size_y, size_z, resolution, to_enlarge_if_line)
    }
    /// The scale step of `FitMinMax`.
    fn fit_scale(
        &self,
        size_x: f64,
        size_y: f64,
        size_z: f64,
        resolution: f64,
        to_enlarge_if_line: bool,
    ) -> Result<bool, Failure> {
        let (mut size_x, mut size_y) = (size_x, size_y);
        if size_x < resolution && size_y < resolution {
            if size_x < resolution || !to_enlarge_if_line {
                return Ok(false);
            }
            (size_x, size_y) = (size_z, size_z);
        }
        let aspect = self.aspect();
        self.set_scale(if aspect > 1.0 {
            std_max(size_x / aspect, size_y)
        } else {
            std_max(size_x, size_y * aspect)
        });
        Ok(true)
    }
}

/// `zEpsilon()`: `FLT_EPSILON`.
fn z_epsilon() -> f64 {
    f64::from(f32::EPSILON)
}

/// `zEpsilon(theValue)`: `FLT_EPSILON` scaled to the binary exponent of the value
/// (`FLT_MIN` at or below it).
fn z_epsilon_of(value: f64) -> f64 {
    let abs = value.abs();
    if abs <= f64::from(f32::MIN_POSITIVE) {
        return f64::from(f32::MIN_POSITIVE);
    }
    let log_radix = abs.log10() / 2.0f64.log10();
    f64::from(f32::EPSILON) * 2.0f64.powf(log_radix.floor())
}

/// The corner points of a box (`Get` then the eight combinations).
fn box_points(bounds: &BoundingBox3) -> Result<[Point3; 8], Failure> {
    let [x0, y0, z0, x1, y1, z1] = bounds.get()?;
    Ok(box_corners(
        &Point3::new(x0, y0, z0),
        &Point3::new(x1, y1, z1),
    ))
}

/// The signed distances of points from the eye plane (negative behind the eye), as
/// `(min, max)` over the graphic points and over the model points.
fn depth_ranges(
    camera: &Graphic3dCamera,
    graphic: &[Point3],
    model: &[Point3],
) -> Result<([f64; 2], [f64; 2]), Failure> {
    let (eye, direction, scale) = (camera.eye(), camera.direction(), camera.axial_scale());
    let plane = Plane3::new(eye, direction)?;
    let mut ranges = [[
        rustycad_tkernel::standard::real::real_last(),
        rustycad_tkernel::standard::real::real_first(),
    ]; 2];
    for (index, point) in graphic.iter().chain(model).enumerate() {
        let measured = Point3::new(
            point.x() * scale.x(),
            point.y() * scale.y(),
            point.z() * scale.z(),
        );
        let mut distance = plane.distance(measured);
        let to_point = Vector3::between(eye, measured);
        if to_point.magnitude() > rustycad_tkmath::gp::resolution()
            && direction.is_opposite(dir(to_point)?, std::f64::consts::PI * 0.5)
        {
            distance *= -1.0;
        }
        let range = &mut ranges[usize::from(index >= 8)];
        (range[0], range[1]) = (std_min(distance, range[0]), std_max(distance, range[1]));
    }
    Ok((ranges[0], ranges[1]))
}

/// `ZFitAll` result: whether it fitted, and the near and far distances.
pub type Graphic3dZFit = (bool, f64, f64);

impl Graphic3dCamera {
    /// `ZFitAll(scale, minMax, graphicBB, zNear, zFar)`: the Z range that shows the graphic
    /// box (and the finite model box); the defaults and false for a void box or a box
    /// behind a perspective camera. A non-positive scale factor or an inverted result raises
    /// `Standard_ProgramError`. Example: `camera.z_fit_all_range(1.0, &model, &graphic)?`.
    pub fn z_fit_all_range(
        &self,
        scale_factor: f64,
        min_max: &BoundingBox3,
        graphic: &BoundingBox3,
    ) -> Result<Graphic3dZFit, Failure> {
        if scale_factor <= 0.0 || scale_factor.is_nan() {
            return Err(program_error(&format!(
                "Zero or negative scale factor is not allowed.; got {scale_factor}"
            )));
        }
        if graphic.is_void() {
            return Ok((
                false,
                super::camera::DEFAULT_ZNEAR,
                super::camera::DEFAULT_ZFAR,
            ));
        }
        let is_finite_min_max = !min_max.is_void() && !min_max.is_whole();
        let model = if is_finite_min_max {
            box_points(min_max)?.to_vec()
        } else {
            Vec::new()
        };
        let (graph_range, model_range) = depth_ranges(self, &box_points(graphic)?, &model)?;
        let mid = (graph_range[0] + graph_range[1]) * 0.5;
        let half = (graph_range[1] - graph_range[0]) * 0.5;
        let (near, far) = (mid - half * scale_factor, mid + half * scale_factor);
        if !self.is_orthographic() && far < z_epsilon() {
            return Ok((
                false,
                super::camera::DEFAULT_ZNEAR,
                super::camera::DEFAULT_ZFAR,
            ));
        }
        let eye_conf = 3.0 * z_epsilon_of(self.eye().xyz().modulus());
        let model_conf = 6.0 * z_epsilon_of(graphic.corner_min()?.xyz().modulus())
            + 6.0 * z_epsilon_of(graphic.corner_max()?.xyz().modulus());
        let near = near - (z_epsilon_of(near) + eye_conf + model_conf);
        let far = far + (z_epsilon_of(far) + eye_conf + model_conf);
        let range = if is_finite_min_max {
            model_range
        } else {
            graph_range
        };
        let (near, far) = if self.is_orthographic() {
            (near, far)
        } else {
            perspective_range(near, far, range)?
        };
        if far <= near {
            return Err(program_error(&format!(
                "ZFar should be greater than ZNear; got near {near}, far {far}"
            )));
        }
        Ok((true, near, far))
    }
    /// `ZFitAll(scale, minMax, graphicBB)`: fits and applies the Z range;
    /// `camera.z_fit_all(1.0, &model, &graphic)?`.
    pub fn z_fit_all(
        &self,
        scale_factor: f64,
        min_max: &BoundingBox3,
        graphic: &BoundingBox3,
    ) -> Result<(), Failure> {
        let (_, near, far) = self.z_fit_all_range(scale_factor, min_max, graphic)?;
        self.set_z_range(near, far)
    }
}

/// The perspective correction of `ZFitAll`: a minimal near distance relative to the depth.
fn perspective_range(near: f64, far: f64, [min, max]: [f64; 2]) -> Result<(f64, f64), Failure> {
    let (mut near, mut far) = (near, far);
    let z_range = max - min;
    let z = if min < 0.0 {
        z_range / 2.0
    } else {
        z_range / 2.0 + min
    };
    let near_min = z * 5.97E-4;
    if near < near_min {
        near = near_min;
        if far < near {
            far = near;
        }
    } else {
        near -= far * z_epsilon_of(near) / (far - z_epsilon_of(near));
    }
    far += z_epsilon_of(far);
    if near < z_epsilon() {
        near = z_epsilon();
    }
    if far <= near {
        return Err(program_error(&format!(
            "ZFar should be greater than ZNear; got near {near}, far {far}"
        )));
    }
    Ok((near, far))
}
