//! `Graphic3d_CullingTool`: the view frustum, distance and size culling tests of boxes,
//! with the frustum plane projections cached per camera state.
use super::{GRAPHIC3D_CAMERA_FRUSTUM_VERTICES_NB, Graphic3dCamera, Graphic3dWorldViewProjState};
use rustycad_tkernel::ncollection::{Mat4, Vec3, Vec4};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::resolution;

/// `Graphic3d_CullingTool::CullingContext`: the distance and squared size limits (-1: none).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Graphic3dCullingContext {
    /// `DistCull`: the culling distance.
    pub dist_cull: f64,
    /// `SizeCull2`: the squared culling size.
    pub size_cull2: f64,
}
impl Default for Graphic3dCullingContext {
    fn default() -> Self {
        Self {
            dist_cull: -1.0,
            size_cull2: -1.0,
        }
    }
}

/// `Graphic3d_CullingTool::Plane`: a frustum plane (origin and normal).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Graphic3dCullingPlane {
    /// `Origin`.
    pub origin: Vec3<f64>,
    /// `Normal`.
    pub normal: Vec3<f64>,
}
impl Default for Graphic3dCullingPlane {
    fn default() -> Self {
        Self {
            origin: Vec3::new(0.0, 0.0, 0.0),
            normal: Vec3::new(0.0, 0.0, 1.0),
        }
    }
}

impl Graphic3dCullingPlane {
    /// A plane through `origin` with `normal`; `Graphic3dCullingPlane::new(o, n)`.
    pub fn new(origin: Vec3<f64>, normal: Vec3<f64>) -> Self {
        Self { origin, normal }
    }
}

/// The number of frustum planes (`PlanesNB`).
const PLANES_NB: usize = 6;

/// The culling tool; `let mut tool = Graphic3dCullingTool::default(); tool.set_view_volume(&camera, &Mat4::identity());`.
#[derive(Clone)]
pub struct Graphic3dCullingTool {
    pub(crate) clip_planes: [Graphic3dCullingPlane; PLANES_NB],
    pub(crate) clip_verts: [Vec3<f64>; GRAPHIC3D_CAMERA_FRUSTUM_VERTICES_NB + 1],
    pub(crate) camera: Handle<Graphic3dCamera>,
    pub(crate) max_clip_projection_pts: [f64; PLANES_NB],
    pub(crate) min_clip_projection_pts: [f64; PLANES_NB],
    pub(crate) max_ortho_projection_pts: [f64; 3],
    pub(crate) min_ortho_projection_pts: [f64; 3],
    pub(crate) is_projection_parallel: bool,
    pub(crate) projection_mat: Mat4<f64>,
    pub(crate) world_view_mat: Mat4<f64>,
    pub(crate) viewport_width: i32,
    pub(crate) viewport_height: i32,
    pub(crate) world_view_proj_state: Graphic3dWorldViewProjState,
    pub(crate) cam_eye: Vec3<f64>,
    pub(crate) cam_dir: Vec3<f64>,
    pub(crate) cam_scale: f64,
    pub(crate) pixel_size: f64,
}

impl Default for Graphic3dCullingTool {
    /// Parallel projection, unit scale and pixel size; the projections (natively
    /// uninitialised until `CacheClipPtsProjections`) start at zero.
    fn default() -> Self {
        let zero = Vec3::new(0.0, 0.0, 0.0);
        Self {
            clip_planes: [Graphic3dCullingPlane::default(); PLANES_NB],
            clip_verts: [zero; GRAPHIC3D_CAMERA_FRUSTUM_VERTICES_NB + 1],
            camera: Handle::default(),
            max_clip_projection_pts: [0.0; PLANES_NB],
            min_clip_projection_pts: [0.0; PLANES_NB],
            max_ortho_projection_pts: [0.0; 3],
            min_ortho_projection_pts: [0.0; 3],
            is_projection_parallel: true,
            projection_mat: Mat4::identity(),
            world_view_mat: Mat4::identity(),
            viewport_width: 0,
            viewport_height: 0,
            world_view_proj_state: Graphic3dWorldViewProjState::default(),
            cam_eye: zero,
            cam_dir: zero,
            cam_scale: 1.0,
            pixel_size: 1.0,
        }
    }
}

/// `std::max(double, double)` / `std::min`.
fn max_f64(a: f64, b: f64) -> f64 {
    if a < b { b } else { a }
}
fn min_f64(a: f64, b: f64) -> f64 {
    if b < a { b } else { a }
}

impl Graphic3dCullingTool {
    /// `Plane_Left`: the index of the left frustum plane.
    pub const PLANE_LEFT: usize = 0;
    /// `Plane_Right`.
    pub const PLANE_RIGHT: usize = 1;
    /// `Plane_Bottom`.
    pub const PLANE_BOTTOM: usize = 2;
    /// `Plane_Top`.
    pub const PLANE_TOP: usize = 3;
    /// `Plane_Near`.
    pub const PLANE_NEAR: usize = 4;
    /// `Plane_Far`.
    pub const PLANE_FAR: usize = 5;
    /// `PlanesNB`: the number of frustum planes.
    pub const PLANES_NB: usize = PLANES_NB;

    /// `SetViewVolume`: the frustum of the camera (in the model space of `model_world`);
    /// nothing changes for the same camera state without a model transformation.
    /// Example: `tool.set_view_volume(&camera, &Mat4::identity())`.
    pub fn set_view_volume(&mut self, camera: &Handle<Graphic3dCamera>, model_world: &Mat4<f64>) {
        let Some(cam) = camera.get() else { return };
        let has_model_trsf = !model_world.is_identity();
        if !self
            .world_view_proj_state
            .is_changed(&cam.world_view_proj_state())
            && !has_model_trsf
        {
            return;
        }
        self.is_projection_parallel = cam.is_orthographic();
        let direction = cam.direction();
        self.camera = camera.clone();
        self.projection_mat = cam.projection_matrix();
        self.world_view_mat = cam.orientation_matrix();
        self.world_view_proj_state = cam.world_view_proj_state();
        let eye = cam.eye();
        self.cam_eye = Vec3::new(eye.x(), eye.y(), eye.z());
        self.cam_dir = Vec3::new(direction.x(), direction.y(), direction.z());
        if has_model_trsf {
            self.apply_model_inverse(model_world);
        }
        self.cam_scale = if cam.is_orthographic() {
            cam.scale()
        } else {
            2.0 * (cam.fov_y() * std::f64::consts::PI / 360.0).tan()
        };
        let points = cam.frustum_points(model_world);
        self.clip_verts[..GRAPHIC3D_CAMERA_FRUSTUM_VERTICES_NB].copy_from_slice(&points);
        self.update_clip_planes();
    }
    /// The eye and direction in the model space (`Inverted` leaves the identity when singular).
    fn apply_model_inverse(&mut self, model_world: &Mat4<f64>) {
        let mut inverse = Mat4::identity();
        let _ = model_world.inverted_to(&mut inverse);
        let (e, d) = (self.cam_eye, self.cam_dir);
        let eye = inverse * Vec4::new(e.x, e.y, e.z, 1.0);
        let dir = inverse * Vec4::new(d.x, d.y, d.z, 0.0);
        self.cam_eye = Vec3::new(eye.x, eye.y, eye.z);
        self.cam_dir = Vec3::new(dir.x, dir.y, dir.z);
    }
    /// The six frustum planes from the frustum corners (left/right, bottom/top, near/far).
    fn update_clip_planes(&mut self) {
        const LOOKUP1: [usize; 3] = [0, 1, 0];
        const LOOKUP2: [usize; 3] = [0, 0, 1];
        let mut shifts = [0usize; 3];
        for face in 0..3 {
            for i in 0..2 {
                let mut points = [Vec3::new(0.0, 0.0, 0.0); 3];
                for (k, point) in points.iter_mut().enumerate() {
                    shifts[face] = i;
                    shifts[(face + 1) % 3] = LOOKUP1[k];
                    shifts[(face + 2) % 3] = LOOKUP2[k];
                    *point = self.clip_verts[shifts[0] * 4 + shifts[1] * 2 + shifts[2]];
                }
                let normal =
                    Vec3::cross(&(points[1] - points[0]), &(points[2] - points[0])).normalized();
                let sign = if i == 0 { -1.0 } else { 1.0 };
                self.clip_planes[face * 2 + i] = Graphic3dCullingPlane {
                    origin: points[0],
                    normal: normal * sign,
                };
            }
        }
    }
    /// `SetViewportSize`: the viewport (at least 1x1) and the pixel size.
    /// Example: `tool.set_viewport_size(800, 600, 1.0)`.
    pub fn set_viewport_size(&mut self, width: i32, height: i32, resolution_ratio: f64) {
        self.viewport_height = if height > 0 { height } else { 1 };
        self.viewport_width = if width > 0 { width } else { 1 };
        self.pixel_size = max_f64(
            resolution_ratio / f64::from(self.viewport_height),
            resolution_ratio / f64::from(self.viewport_width),
        );
    }
    /// `SetCullingDistance`: a finite positive distance for perspective views only.
    /// Example: `tool.set_culling_distance(&mut ctx, 100.0)`.
    pub fn set_culling_distance(&self, ctx: &mut Graphic3dCullingContext, distance: f64) {
        ctx.dist_cull = -1.0;
        if !self.is_projection_parallel {
            let finite = distance > 0.0 && !precision::is_infinite(distance);
            ctx.dist_cull = if finite { distance } else { -1.0 };
        }
    }
    /// `SetCullingSize`: the squared size in world units at unit distance.
    /// Example: `tool.set_culling_size(&mut ctx, 2.0)`.
    pub fn set_culling_size(&self, ctx: &mut Graphic3dCullingContext, size: f64) {
        ctx.size_cull2 = -1.0;
        if size > 0.0 && !precision::is_infinite(size) {
            ctx.size_cull2 = self.pixel_size * size;
            ctx.size_cull2 *= self.cam_scale;
            ctx.size_cull2 *= ctx.size_cull2;
        }
    }
    /// `CacheClipPtsProjections`: the frustum corner projections onto the plane normals and
    /// the axes. Example: `tool.cache_clip_pts_projections()`.
    pub fn cache_clip_pts_projections(&mut self) {
        let step = if self.is_projection_parallel { 2 } else { 1 };
        for plane in (0..PLANES_NB - 1).step_by(step) {
            let normal = self.clip_planes[plane].normal;
            let (min, max) = self.corner_projection_range(&normal);
            (
                self.min_clip_projection_pts[plane],
                self.max_clip_projection_pts[plane],
            ) = (min, max);
        }
        let axes = [
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
        ];
        for (dim, axis) in axes.iter().enumerate() {
            let (min, max) = self.corner_projection_range(axis);
            (
                self.min_ortho_projection_pts[dim],
                self.max_ortho_projection_pts[dim],
            ) = (min, max);
        }
    }
    fn corner_projection_range(&self, axis: &Vec3<f64>) -> (f64, f64) {
        let (mut min, mut max) = (f64::MAX, -f64::MAX);
        for corner in &self.clip_verts[..GRAPHIC3D_CAMERA_FRUSTUM_VERTICES_NB] {
            let projection = corner.dot(axis);
            max = max_f64(projection, max);
            min = min_f64(projection, min);
        }
        (min, max)
    }
    /// `SignedPlanePointDistance`: 0 for a null normal; `tool.signed_plane_point_distance(&n, &p)`.
    pub fn signed_plane_point_distance(&self, normal: &Vec4<f64>, point: &Vec4<f64>) -> f64 {
        let length = (normal.x * normal.x + normal.y * normal.y + normal.z * normal.z).sqrt();
        if length < resolution() {
            return 0.0;
        }
        let inv = 1.0 / length;
        let (d, a, b, c) = (
            normal.w * inv,
            normal.x * inv,
            normal.y * inv,
            normal.z * inv,
        );
        d + (a * point.x + b * point.y + c * point.z)
    }
}

impl Graphic3dCullingTool {
    /// `IsCulled`: out of the frustum, too distant or too small; `inside` (when given and
    /// true) is cleared when the box is not fully inside. Example: `tool.is_culled(&ctx, &min, &max, None)`.
    pub fn is_culled(
        &self,
        ctx: &Graphic3dCullingContext,
        min: &Vec3<f64>,
        max: &Vec3<f64>,
        mut inside: Option<&mut bool>,
    ) -> bool {
        self.is_out_frustum(min, max, inside.as_deref_mut())
            || self.is_too_distant(ctx, min, max, inside)
            || self.is_too_small(ctx, min, max)
    }
    /// `IsOutFrustum`: the separating-axis test against the frustum. Quirk kept: the "fully
    /// inside" Z test compares the Y coordinates of the box. Example: `tool.is_out_frustum(&min, &max, None)`.
    pub fn is_out_frustum(
        &self,
        min: &Vec3<f64>,
        max: &Vec3<f64>,
        mut inside: Option<&mut bool>,
    ) -> bool {
        let (lo, hi) = (
            &self.min_ortho_projection_pts,
            &self.max_ortho_projection_pts,
        );
        if min.x > hi[0]
            || max.x < lo[0]
            || min.y > hi[1]
            || max.y < lo[1]
            || min.z > hi[2]
            || max.z < lo[2]
        {
            return true;
        }
        if let Some(flag) = inside.as_deref_mut().filter(|f| **f) {
            *flag = min.x >= lo[0]
                && max.x <= hi[0]
                && min.y >= lo[1]
                && max.y <= hi[1]
                && min.y >= lo[2]
                && max.y <= hi[2];
        }
        let step = if self.is_projection_parallel { 2 } else { 1 };
        (0..PLANES_NB - 1)
            .step_by(step)
            .any(|plane| self.is_out_of_plane(plane, min, max, inside.as_deref_mut()))
    }
    /// The test of one frustum plane of `IsOutFrustum`.
    fn is_out_of_plane(
        &self,
        plane: usize,
        min: &Vec3<f64>,
        max: &Vec3<f64>,
        inside: Option<&mut bool>,
    ) -> bool {
        let axis = self.clip_planes[plane].normal;
        let pick = |positive: bool, a: f64, b: f64| if positive { a } else { b };
        let p_vertex = Vec3::new(
            pick(axis.x > 0.0, max.x, min.x),
            pick(axis.y > 0.0, max.y, min.y),
            pick(axis.z > 0.0, max.z, min.z),
        );
        let p0 = p_vertex.dot(&axis);
        let (plane_min, plane_max) = (
            self.min_clip_projection_pts[plane],
            self.max_clip_projection_pts[plane],
        );
        if inside.is_none() && p0 >= plane_min && p0 <= plane_max {
            return false;
        }
        let n_vertex = Vec3::new(
            pick(axis.x > 0.0, min.x, max.x),
            pick(axis.y > 0.0, min.y, max.y),
            pick(axis.z > 0.0, min.z, max.z),
        );
        let p1 = n_vertex.dot(&axis);
        let (box_min, box_max) = (if p0 < p1 { p0 } else { p1 }, if p0 > p1 { p0 } else { p1 });
        if box_min > plane_max || box_max < plane_min {
            return true;
        }
        if let Some(flag) = inside.filter(|f| **f) {
            *flag = box_min >= plane_min && box_max <= plane_max;
        }
        false
    }
    /// `IsTooDistant`: the bounding sphere beyond the culling distance from the eye.
    /// Example: `tool.is_too_distant(&ctx, &min, &max, None)`.
    pub fn is_too_distant(
        &self,
        ctx: &Graphic3dCullingContext,
        min: &Vec3<f64>,
        max: &Vec3<f64>,
        inside: Option<&mut bool>,
    ) -> bool {
        if ctx.dist_cull <= 0.0 {
            return false;
        }
        let center = (*min + *max) * 0.5;
        let radius = (*max - *min).max_comp() * 0.5;
        let distance = (center - self.cam_eye).modulus();
        if distance - radius > ctx.dist_cull {
            return true;
        }
        if let Some(flag) = inside.filter(|f| **f) {
            *flag = distance + radius <= ctx.dist_cull;
        }
        false
    }
    /// `IsTooSmall`: the box diagonal below the culling size (scaled by the distance along
    /// the view direction in perspective). Example: `tool.is_too_small(&ctx, &min, &max)`.
    pub fn is_too_small(
        &self,
        ctx: &Graphic3dCullingContext,
        min: &Vec3<f64>,
        max: &Vec3<f64>,
    ) -> bool {
        if ctx.size_cull2 <= 0.0 {
            return false;
        }
        let diag2 = (*max - *min).square_modulus();
        if self.is_projection_parallel {
            return diag2 < ctx.size_cull2;
        }
        let center = (*min + *max) * 0.5;
        let distance = (center - self.cam_eye).dot(&self.cam_dir);
        diag2 < ctx.size_cull2 * distance * distance
    }
}

impl Graphic3dCullingTool {
    /// The camera of the view volume; `tool.camera()`.
    pub fn camera(&self) -> &Handle<Graphic3dCamera> {
        &self.camera
    }
    /// The projection matrix; `tool.projection_matrix()`.
    pub fn projection_matrix(&self) -> &Mat4<f64> {
        &self.projection_mat
    }
    /// The world-view matrix; `tool.world_view_matrix()`.
    pub fn world_view_matrix(&self) -> &Mat4<f64> {
        &self.world_view_mat
    }
    /// The viewport width; `tool.viewport_width()`.
    pub fn viewport_width(&self) -> i32 {
        self.viewport_width
    }
    /// The viewport height; `tool.viewport_height()`.
    pub fn viewport_height(&self) -> i32 {
        self.viewport_height
    }
    /// The camera state of the view volume; `tool.world_view_proj_state()`.
    pub fn world_view_proj_state(&self) -> &Graphic3dWorldViewProjState {
        &self.world_view_proj_state
    }
    /// The eye (in the model space); `tool.camera_eye()`.
    pub fn camera_eye(&self) -> &Vec3<f64> {
        &self.cam_eye
    }
    /// The view direction (in the model space); `tool.camera_direction()`.
    pub fn camera_direction(&self) -> &Vec3<f64> {
        &self.cam_dir
    }
}
