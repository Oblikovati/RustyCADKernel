//! The cached matrices of `Graphic3d_Camera` and the point conversions between world, view
//! and projection space.
use super::camera::{Graphic3dCameraMatrices, Graphic3dCameraState};
use super::camera_projection::{
    Graphic3dCameraScalar, ProjectionParams, compute_projection, look_orientation,
};
use super::{
    Graphic3dCamera, Graphic3dCameraFocusType, Graphic3dCameraIodType, Graphic3dWorldViewProjState,
};
use rustycad_tkernel::ncollection::{ElementCast, Mat4, Vec3, Vec4};
use rustycad_tkmath::gp::Point3;

/// The projection parameters of a camera state (IOD and focus resolved in world units).
fn projection_params(s: &Graphic3dCameraState) -> ProjectionParams {
    let iod = if s.iod_type == Graphic3dCameraIodType::Relative {
        s.iod * s.distance
    } else {
        s.iod
    };
    let z_focus = if s.z_focus_type == Graphic3dCameraFocusType::Relative {
        s.z_focus * s.distance
    } else {
        s.z_focus
    };
    ProjectionParams {
        kind: s.proj_type,
        is_zero_to_one_depth: s.is_zero_to_one_depth,
        scale: s.scale,
        z_near: s.z_near,
        z_far: s.z_far,
        aspect: s.aspect,
        fov_y_tan: s.fov_y_tan,
        iod,
        z_focus,
        tile: s.tile,
    }
}

/// `UpdateOrientation`: computes the orientation when it is not valid.
fn update_orientation<T: Graphic3dCameraScalar>(
    s: &Graphic3dCameraState,
    matrices: &mut Graphic3dCameraMatrices<T>,
) {
    if matrices.is_orientation_valid {
        return;
    }
    let vec = |x: f64, y: f64, z: f64| Vec3::new(T::from_f64(x), T::from_f64(y), T::from_f64(z));
    let (e, d, u, a) = (s.eye, s.direction, s.up, s.axial_scale);
    matrices.orientation = look_orientation(
        vec(e.x(), e.y(), e.z()),
        vec(d.x(), d.y(), d.z()),
        vec(u.x(), u.y(), u.z()),
        vec(a.x(), a.y(), a.z()),
    );
    matrices.is_orientation_valid = true;
}

/// `UpdateProjection`: computes the projections when they are not valid.
fn update_projection<T: Graphic3dCameraScalar>(
    s: &Graphic3dCameraState,
    matrices: &mut Graphic3dCameraMatrices<T>,
) where
    f64: ElementCast<T>,
{
    if matrices.is_projection_valid {
        return;
    }
    [
        matrices.m_projection,
        matrices.l_projection,
        matrices.r_projection,
    ] = compute_projection(&projection_params(s), &s.custom, true);
    matrices.is_projection_valid = true;
}

/// The double matrices after `update` (an orientation or projection update).
fn matrices_d(
    camera: &Graphic3dCamera,
    update: fn(&Graphic3dCameraState, &mut Graphic3dCameraMatrices<f64>),
) -> Graphic3dCameraMatrices<f64> {
    camera.write(|s| {
        let mut matrices = s.matrices_d;
        update(s, &mut matrices);
        s.matrices_d = matrices;
        matrices
    })
}

/// The float matrices after `update`.
fn matrices_f(
    camera: &Graphic3dCamera,
    update: fn(&Graphic3dCameraState, &mut Graphic3dCameraMatrices<f32>),
) -> Graphic3dCameraMatrices<f32> {
    camera.write(|s| {
        let mut matrices = s.matrices_f;
        update(s, &mut matrices);
        s.matrices_f = matrices;
        matrices
    })
}

/// The matrices and their state.
impl Graphic3dCamera {
    /// The world-to-view matrix; `camera.orientation_matrix()`.
    pub fn orientation_matrix(&self) -> Mat4<f64> {
        matrices_d(self, update_orientation).orientation
    }
    /// The world-to-view matrix in float; `camera.orientation_matrix_f()`.
    pub fn orientation_matrix_f(&self) -> Mat4<f32> {
        matrices_f(self, update_orientation).orientation
    }
    /// The (mono) projection matrix; `camera.projection_matrix()`.
    pub fn projection_matrix(&self) -> Mat4<f64> {
        matrices_d(self, update_projection).m_projection
    }
    /// The projection matrix in float; `camera.projection_matrix_f()`.
    pub fn projection_matrix_f(&self) -> Mat4<f32> {
        matrices_f(self, update_projection).m_projection
    }
    /// The left eye projection; `camera.projection_stereo_left()`.
    pub fn projection_stereo_left(&self) -> Mat4<f64> {
        matrices_d(self, update_projection).l_projection
    }
    /// The left eye projection in float; `camera.projection_stereo_left_f()`.
    pub fn projection_stereo_left_f(&self) -> Mat4<f32> {
        matrices_f(self, update_projection).l_projection
    }
    /// The right eye projection; `camera.projection_stereo_right()`.
    pub fn projection_stereo_right(&self) -> Mat4<f64> {
        matrices_d(self, update_projection).r_projection
    }
    /// The right eye projection in float; `camera.projection_stereo_right_f()`.
    pub fn projection_stereo_right_f(&self) -> Mat4<f32> {
        matrices_f(self, update_projection).r_projection
    }
    /// Drops the cached projections; `camera.invalidate_projection()`.
    pub fn invalidate_projection(&self) {
        self.write(Graphic3dCameraState::invalidate_projection);
    }
    /// Drops the cached orientations; `camera.invalidate_orientation()`.
    pub fn invalidate_orientation(&self) {
        self.write(Graphic3dCameraState::invalidate_orientation);
    }
    /// The change counters; `camera.world_view_proj_state()`.
    pub fn world_view_proj_state(&self) -> Graphic3dWorldViewProjState {
        self.read(|s| s.world_view_proj_state)
    }
    /// The projection counter; `camera.projection_state()`.
    pub fn projection_state(&self) -> usize {
        self.world_view_proj_state().projection_state()
    }
    /// The world-view counter; `camera.world_view_state()`.
    pub fn world_view_state(&self) -> usize {
        self.world_view_proj_state().world_view_state()
    }
    /// `StereoProjection`: left projection, left head-to-eye, right projection, right
    /// head-to-eye, without the head-to-eye in the projections. Native quirk: with a zero
    /// IOD the head-to-eye outputs are left untouched; here they are the inputs passed in.
    /// Example: `camera.stereo_projection(Mat4::identity(), Mat4::identity())`.
    pub fn stereo_projection(
        &self,
        head_to_eye_l: Mat4<f64>,
        head_to_eye_r: Mat4<f64>,
    ) -> [Mat4<f64>; 4] {
        self.read(|s| stereo_projection_of(s, head_to_eye_l, head_to_eye_r))
    }
    /// `StereoProjectionF`; `camera.stereo_projection_f(Mat4::identity(), Mat4::identity())`.
    pub fn stereo_projection_f(
        &self,
        head_to_eye_l: Mat4<f32>,
        head_to_eye_r: Mat4<f32>,
    ) -> [Mat4<f32>; 4] {
        self.read(|s| stereo_projection_of(s, head_to_eye_l, head_to_eye_r))
    }
}

/// `stereoProjection<Elem_t>`.
fn stereo_projection_of<T: Graphic3dCameraScalar>(
    s: &Graphic3dCameraState,
    mut head_l: Mat4<T>,
    mut head_r: Mat4<T>,
) -> [Mat4<T>; 4]
where
    f64: ElementCast<T>,
{
    if let Some([pl, hl, pr, hr]) = s.custom.stereo {
        return [
            Mat4::from_other(&pl),
            Mat4::from_other(&hl),
            Mat4::from_other(&pr),
            Mat4::from_other(&hr),
        ];
    }
    let [_, left, right] = compute_projection::<T>(&projection_params(s), &s.custom, false);
    let iod = if s.iod_type == Graphic3dCameraIodType::Relative {
        s.iod * s.distance
    } else {
        s.iod
    };
    if iod != 0.0 {
        let zero = T::from_f64(0.0);
        head_l = Mat4::identity();
        head_l.set_column3(3, Vec3::new(T::from_f64(0.5 * iod), zero, zero));
        head_r = Mat4::identity();
        head_r.set_column3(3, Vec3::new(T::from_f64(-0.5 * iod), zero, zero));
    }
    [left, head_l, right, head_r]
}

/// `safePointCast`: coordinates beyond `1e15f` clamp to `±1e15f * 0.1f`, with `w = 1`.
fn safe_point_cast(point: &Point3) -> Vec4<f64> {
    let limit = f64::from(1e15f32);
    let big = limit * f64::from(0.1f32);
    let clamp = |value: f64| {
        if value.abs() > limit {
            if value >= 0.0 { big } else { -big }
        } else {
            value
        }
    };
    Vec4::new(clamp(point.x()), clamp(point.y()), clamp(point.z()), 1.0)
}

/// Applies the matrices in order and divides by `w`.
fn transformed(point: &Point3, matrices: &[Mat4<f64>]) -> Point3 {
    let mut value = safe_point_cast(point);
    for matrix in matrices {
        value = *matrix * value;
    }
    let inv_w = 1.0 / value.w;
    Point3::new(value.x * inv_w, value.y * inv_w, value.z * inv_w)
}

/// The inverse of a matrix, `None` when singular (`Inverted` returning false).
fn inverse(matrix: &Mat4<f64>) -> Option<Mat4<f64>> {
    let mut output = Mat4::identity();
    matrix.inverted_to(&mut output).then_some(output)
}

/// Point conversions.
impl Graphic3dCamera {
    /// `Project`: world to normalized device coordinates; `camera.project(&p)`.
    pub fn project(&self, point: &Point3) -> Point3 {
        transformed(
            point,
            &[self.orientation_matrix(), self.projection_matrix()],
        )
    }
    /// `UnProject`: device coordinates to world (the origin for singular matrices);
    /// `camera.un_project(&p)`.
    pub fn un_project(&self, point: &Point3) -> Point3 {
        let (view, projection) = (self.orientation_matrix(), self.projection_matrix());
        match (inverse(&view), inverse(&projection)) {
            (Some(inv_view), Some(inv_projection)) => {
                transformed(point, &[inv_projection, inv_view])
            }
            _ => Point3::new(0.0, 0.0, 0.0),
        }
    }
    /// `ConvertView2Proj`; `camera.convert_view_to_proj(&p)`.
    pub fn convert_view_to_proj(&self, point: &Point3) -> Point3 {
        transformed(point, &[self.projection_matrix()])
    }
    /// `ConvertProj2View` (the origin for a singular projection); `camera.convert_proj_to_view(&p)`.
    pub fn convert_proj_to_view(&self, point: &Point3) -> Point3 {
        inverse(&self.projection_matrix())
            .map_or(Point3::new(0.0, 0.0, 0.0), |inv| transformed(point, &[inv]))
    }
    /// `ConvertWorld2View`; `camera.convert_world_to_view(&p)`.
    pub fn convert_world_to_view(&self, point: &Point3) -> Point3 {
        transformed(point, &[self.orientation_matrix()])
    }
    /// `ConvertView2World` (the origin for a singular orientation); `camera.convert_view_to_world(&p)`.
    pub fn convert_view_to_world(&self, point: &Point3) -> Point3 {
        inverse(&self.orientation_matrix())
            .map_or(Point3::new(0.0, 0.0, 0.0), |inv| transformed(point, &[inv]))
    }
}
