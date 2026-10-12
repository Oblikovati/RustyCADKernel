//! `SelectMgr_FrustumBuilder`: the camera, window size and viewport that turn pixels into
//! world points for the selecting frustums.
use rustycad_tkernel::ncollection::{Vec3, Vec4};
use rustycad_tkernel::precision::confusion;
use rustycad_tkernel::standard::{
    Handle, HandleReference, SharedState, StandardTransient, WeakHandleFieldRef,
};
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::graphic3d::Graphic3dCamera;
use std::sync::Arc;

/// The mutable fields of a builder.
#[derive(Clone)]
struct BuilderState {
    camera: Handle<Graphic3dCamera>,
    width: i32,
    height: i32,
    viewport: Vec4<f64>,
    is_viewport_set: bool,
}

/// The pixel-to-world projection of the selecting frustums.
///
/// Example: `let builder = SelectMgrFrustumBuilder::new(); builder.set_window_size(400, 300);`.
pub struct SelectMgrFrustumBuilder {
    transient: StandardTransient,
    state: Arc<SharedState<BuilderState>>,
}
rustycad_tkernel::impl_transient!(
    SelectMgrFrustumBuilder,
    "SelectMgr_FrustumBuilder",
    StandardTransient,
    transient
);

impl Default for SelectMgrFrustumBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl SelectMgrFrustumBuilder {
    /// A builder without camera, of an `INT_MAX` window and no viewport.
    /// Example: `SelectMgrFrustumBuilder::new()`.
    pub fn new() -> Self {
        let state = BuilderState {
            camera: Handle::default(),
            width: i32::MAX,
            height: i32::MAX,
            viewport: Vec4::new(0.0, 0.0, 0.0, 0.0),
            is_viewport_set: false,
        };
        Self {
            transient: StandardTransient::default(),
            state: Arc::new(SharedState::new(state)),
        }
    }
    /// `Camera`; `builder.camera()`.
    pub fn camera(&self) -> Handle<Graphic3dCamera> {
        self.state.read().camera.clone()
    }
    /// Capture the builder camera field without owning the builder or current camera.
    /// Example: `let alias=builder.camera_ref(); builder.set_camera(&camera); assert!(!alias.is_null());`.
    pub fn camera_ref(&self) -> impl HandleReference<Graphic3dCamera> + use<> {
        WeakHandleFieldRef::new(&self.state, builder_camera_field)
    }
    /// `SetCamera`; `builder.set_camera(&camera)`.
    pub fn set_camera(
        &self,
        camera: &dyn rustycad_tkernel::standard::HandleReference<Graphic3dCamera>,
    ) {
        self.state.write().camera = camera.clone_handle();
    }
    /// `SetWindowSize`; `builder.set_window_size(400, 300)`.
    pub fn set_window_size(&self, width: i32, height: i32) {
        let mut state = self.state.write();
        (state.width, state.height) = (width, height);
    }
    /// `SetViewport`; `builder.set_viewport(0.0, 0.0, 1.0, 1.0)`.
    pub fn set_viewport(&self, x: f64, y: f64, width: f64, height: f64) {
        let mut state = self.state.write();
        (state.viewport, state.is_viewport_set) = (Vec4::new(x, y, width, height), true);
    }
    /// `InvalidateViewport`; `builder.invalidate_viewport()`.
    pub fn invalidate_viewport(&self) {
        self.state.write().is_viewport_set = false;
    }
    /// `WindowSize`; `builder.window_size()`.
    pub fn window_size(&self) -> (i32, i32) {
        let state = self.state.read();
        (state.width, state.height)
    }
    /// `SignedPlanePntDist`: the signed distance of a point to the plane through the origin
    /// of a normal (an unnormalized normal shorter than `Precision::Confusion` gives 0).
    /// Example: `builder.signed_plane_pnt_dist(&normal, &point)`.
    pub fn signed_plane_pnt_dist(&self, eq: &Vec3<f64>, pnt: &Vec3<f64>) -> f64 {
        let norm_length = (eq.x * eq.x + eq.y * eq.y + eq.z * eq.z).sqrt();
        let inv_norm_length = if norm_length < confusion() {
            0.0
        } else {
            1.0 / norm_length
        };
        let (a, b, c) = (
            eq.x * inv_norm_length,
            eq.y * inv_norm_length,
            eq.z * inv_norm_length,
        );
        a * pnt.x + b * pnt.y + c * pnt.z
    }
    /// `ProjectPntOnViewPlane`: the world point of a pixel and a normalized depth, through the
    /// viewport when set; the origin without camera. Example: `builder.project_pnt_on_view_plane(10.0, 20.0, 0.0)`.
    pub fn project_pnt_on_view_plane(&self, x: f64, y: f64, z: f64) -> Point3 {
        let state = self.state.read().clone();
        let Some(camera) = state.camera.get() else {
            return Point3::default();
        };
        let (w, h) = (f64::from(state.width), f64::from(state.height));
        let xyz = if !state.is_viewport_set {
            let depth = if camera.is_zero_to_one_depth() {
                z
            } else {
                2.0 * z - 1.0
            };
            Point3::new(2.0 * x / w - 1.0, (h - 1.0 - y) / h * 2.0 - 1.0, depth)
        } else {
            let v = state.viewport;
            Point3::new(
                2.0 * (x - w * v.x) / (w * (v.z - v.x)) - 1.0,
                2.0 * (y - h * v.y) / (h * (v.w - v.y)) - 1.0,
                z,
            )
        };
        camera.un_project(&xyz)
    }
}

fn builder_camera_field(state: &BuilderState) -> &Handle<Graphic3dCamera> {
    // Capture this field once; later reads must never repeat an object getter.
    &state.camera
}
