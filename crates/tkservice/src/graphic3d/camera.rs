//! `Graphic3d_Camera`: the view orientation (eye, direction, up, distance, axial scale) and
//! the projection (orthographic, perspective or stereo; field of view, Z range, aspect,
//! stereo focus and IOD, tile, custom matrices) with cached `double` and `float` matrices.
//!
//! The camera is shared by handles and its matrices are computed lazily from `const`
//! members natively, so the whole state sits behind one lock. The orientation members are
//! here; the projection members in camera_mapping.rs, the matrices and conversions in
//! camera_matrices.rs, and fitting and interpolation in camera_fit.rs.
use super::camera_projection::CustomProjection;
use super::{Graphic3dCameraTile, Graphic3dWorldViewProjState};
use rustycad_tkernel::ncollection::Mat4;
use rustycad_tkernel::standard::{Failure, SharedState, StandardTransient};
use rustycad_tkmath::gp::{Direction3, Point3, Transform3, TransformForm, Xyz};
use std::sync::atomic::{AtomicUsize, Ordering};

/// `DTR_HALF`: half a degree in radians, as the native constant.
pub(crate) const DTR_HALF: f64 = 0.5 * 0.0174532925;
/// `DEFAULT_ZNEAR`.
pub(crate) const DEFAULT_ZNEAR: f64 = 0.001;
/// `DEFAULT_ZFAR`.
pub(crate) const DEFAULT_ZFAR: f64 = 3000.0;

/// `THE_STATE_COUNTER`: the source of the projection and world-view counters.
static STATE_COUNTER: AtomicUsize = AtomicUsize::new(0);
/// The source of the camera identities (the native `this` pointer of the state).
static CAMERA_IDENTITY: AtomicUsize = AtomicUsize::new(0);

/// `++THE_STATE_COUNTER`.
pub(crate) fn next_state() -> usize {
    STATE_COUNTER.fetch_add(1, Ordering::SeqCst).wrapping_add(1)
}

/// The projection type (`Graphic3d_Camera::Projection`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Graphic3dCameraProjection {
    /// `Projection_Orthographic`.
    Orthographic = 0,
    /// `Projection_Perspective`.
    Perspective = 1,
    /// `Projection_Stereo`.
    Stereo = 2,
    /// `Projection_MonoLeftEye`.
    MonoLeftEye = 3,
    /// `Projection_MonoRightEye`.
    MonoRightEye = 4,
}

/// How the stereo focus is given (`Graphic3d_Camera::FocusType`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Graphic3dCameraFocusType {
    /// `FocusType_Absolute`: in world units.
    Absolute = 0,
    /// `FocusType_Relative`: a factor of the distance.
    Relative = 1,
}

/// How the intraocular distance is given (`Graphic3d_Camera::IODType`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Graphic3dCameraIodType {
    /// `IODType_Absolute`: in world units.
    Absolute = 0,
    /// `IODType_Relative`: a factor of the distance.
    Relative = 1,
}

/// The cached matrices of one element type (`TransformMatrices<Elem_t>`).
#[derive(Clone, Copy, Debug)]
pub struct Graphic3dCameraMatrices<T> {
    /// `Orientation`.
    pub orientation: Mat4<T>,
    /// `MProjection`.
    pub m_projection: Mat4<T>,
    /// `LProjection`.
    pub l_projection: Mat4<T>,
    /// `RProjection`.
    pub r_projection: Mat4<T>,
    /// `myIsOrientationValid`.
    pub is_orientation_valid: bool,
    /// `myIsProjectionValid`.
    pub is_projection_valid: bool,
}

impl<T: Copy + From<u8>> Default for Graphic3dCameraMatrices<T> {
    /// Identity matrices, neither orientation nor projection valid.
    fn default() -> Self {
        let identity = Mat4::default();
        Self {
            orientation: identity,
            m_projection: identity,
            l_projection: identity,
            r_projection: identity,
            is_orientation_valid: false,
            is_projection_valid: false,
        }
    }
}

/// The fields of `Graphic3d_Camera`.
#[derive(Clone, Debug)]
pub struct Graphic3dCameraState {
    pub(crate) up: Direction3,
    pub(crate) direction: Direction3,
    pub(crate) eye: Point3,
    pub(crate) distance: f64,
    pub(crate) axial_scale: Xyz,
    pub(crate) proj_type: Graphic3dCameraProjection,
    pub(crate) fov_y: f64,
    pub(crate) fov_x: f64,
    pub(crate) fov_2d: f64,
    pub(crate) fov_y_tan: f64,
    pub(crate) z_near: f64,
    pub(crate) z_far: f64,
    pub(crate) aspect: f64,
    pub(crate) is_zero_to_one_depth: bool,
    pub(crate) scale: f64,
    pub(crate) z_focus: f64,
    pub(crate) z_focus_type: Graphic3dCameraFocusType,
    pub(crate) iod: f64,
    pub(crate) iod_type: Graphic3dCameraIodType,
    pub(crate) tile: Graphic3dCameraTile,
    pub(crate) custom: CustomProjection,
    pub(crate) matrices_d: Graphic3dCameraMatrices<f64>,
    pub(crate) matrices_f: Graphic3dCameraMatrices<f32>,
    pub(crate) world_view_proj_state: Graphic3dWorldViewProjState,
    pub(crate) identity: usize,
}

impl Graphic3dCameraState {
    /// The member initialisers of both native constructors.
    fn initial() -> Self {
        Self {
            up: rustycad_tkmath::gp::dy(),
            direction: rustycad_tkmath::gp::dz(),
            eye: Point3::new(0.0, 0.0, -1500.0),
            distance: 1500.0,
            axial_scale: Xyz::new(1.0, 1.0, 1.0),
            proj_type: Graphic3dCameraProjection::Orthographic,
            fov_y: 45.0,
            fov_x: 45.0,
            fov_2d: 180.0,
            fov_y_tan: (DTR_HALF * 45.0).tan(),
            z_near: DEFAULT_ZNEAR,
            z_far: DEFAULT_ZFAR,
            aspect: 1.0,
            is_zero_to_one_depth: false,
            scale: 1000.0,
            z_focus: 1.0,
            z_focus_type: Graphic3dCameraFocusType::Relative,
            iod: 0.05,
            iod_type: Graphic3dCameraIodType::Relative,
            tile: Graphic3dCameraTile::default(),
            custom: CustomProjection::default(),
            matrices_d: Graphic3dCameraMatrices::default(),
            matrices_f: Graphic3dCameraMatrices::default(),
            world_view_proj_state: Graphic3dWorldViewProjState::default(),
            identity: CAMERA_IDENTITY
                .fetch_add(1, Ordering::SeqCst)
                .wrapping_add(1),
        }
    }
    /// `InvalidateProjection`: drops the projections and bumps the projection counter.
    pub(crate) fn invalidate_projection(&mut self) {
        self.matrices_d.is_projection_valid = false;
        self.matrices_f.is_projection_valid = false;
        self.world_view_proj_state
            .set_projection_state(next_state());
    }
    /// `InvalidateOrientation`: drops the orientations and bumps the world-view counter.
    pub(crate) fn invalidate_orientation(&mut self) {
        self.matrices_d.is_orientation_valid = false;
        self.matrices_f.is_orientation_valid = false;
        self.world_view_proj_state
            .set_world_view_state(next_state());
    }
    /// `Center`: eye + direction * distance.
    pub(crate) fn center(&self) -> Point3 {
        Point3::from(self.eye.xyz() + self.direction.xyz() * self.distance)
    }
}

/// A camera; `Graphic3dCamera::default().eye()` is `(0, 0, -1500)`.
pub struct Graphic3dCamera {
    transient: StandardTransient,
    pub(crate) state: SharedState<Graphic3dCameraState>,
}
rustycad_tkernel::impl_transient!(
    Graphic3dCamera,
    "Graphic3d_Camera",
    StandardTransient,
    transient
);

impl Default for Graphic3dCamera {
    /// An orthographic camera at `(0, 0, -1500)` looking along +Z with +Y up, scale 1000;
    /// the state counters start from two new counter values.
    fn default() -> Self {
        let mut state = Graphic3dCameraState::initial();
        let projection = next_state();
        let world_view = next_state();
        state
            .world_view_proj_state
            .initialize(projection, world_view, state.identity);
        Self {
            transient: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }
}

impl Graphic3dCamera {
    /// `Graphic3d_Camera(theOther)`: zero counters, then a `Copy` of the other camera.
    /// Example: `Graphic3dCamera::from_camera(&other)`.
    pub fn from_camera(other: &Graphic3dCamera) -> Self {
        let mut state = Graphic3dCameraState::initial();
        state
            .world_view_proj_state
            .initialize_camera(state.identity);
        let camera = Self {
            transient: StandardTransient::default(),
            state: SharedState::new(state),
        };
        camera.copy(other);
        camera
    }
    /// The implicit copy constructor (`new Graphic3d_Camera(*camera)`): every member as it is,
    /// including the cached matrices and the state counters. It differs from
    /// [`Self::from_camera`], whose `Copy` goes through the setters (a perspective camera's
    /// scale is then recomputed from its distance). Example: `camera.copy_members()`.
    pub fn copy_members(&self) -> Self {
        Self {
            transient: StandardTransient::default(),
            state: SharedState::new(self.read(Clone::clone)),
        }
    }
    /// Reads the state.
    pub(crate) fn read<R>(&self, read: impl FnOnce(&Graphic3dCameraState) -> R) -> R {
        read(&self.state.read())
    }
    /// Changes the state.
    pub(crate) fn write<R>(&self, write: impl FnOnce(&mut Graphic3dCameraState) -> R) -> R {
        write(&mut self.state.write())
    }
    /// `Copy`: the mapping data, then the orientation data; `camera.copy(&other)`.
    pub fn copy(&self, other: &Graphic3dCamera) {
        self.copy_mapping_data(other);
        self.copy_orientation_data(other);
    }
    /// `CopyOrientationData`: eye, up, direction and distance (when any differs) and the
    /// axial scale; `camera.copy_orientation_data(&other)`.
    pub fn copy_orientation_data(&self, other: &Graphic3dCamera) {
        let theirs = other.read(Clone::clone);
        self.write(|s| {
            let same = s.eye.is_equal(&theirs.eye, 0.0)
                && s.up.is_equal(&theirs.up, 0.0)
                && s.direction.is_equal(&theirs.direction, 0.0)
                && s.distance == theirs.distance;
            if !same {
                (s.eye, s.up, s.direction, s.distance) =
                    (theirs.eye, theirs.up, theirs.direction, theirs.distance);
                s.invalidate_orientation();
            }
        });
        // The axial scale of a camera is always positive, so the check cannot raise.
        let _ = self.set_axial_scale(theirs.axial_scale);
    }
}

/// Orientation members.
impl Graphic3dCamera {
    /// The view direction (from the eye); `camera.direction()`.
    pub fn direction(&self) -> Direction3 {
        self.read(|s| s.direction)
    }
    /// Sets the direction keeping the eye; `camera.set_direction_from_eye(&dir)`.
    pub fn set_direction_from_eye(&self, direction: &Direction3) {
        self.write(|s| {
            if !s.direction.is_equal(direction, 0.0) {
                s.direction = *direction;
                s.invalidate_orientation();
            }
        });
    }
    /// Sets the direction keeping the center; `camera.set_direction(&dir)`.
    pub fn set_direction(&self, direction: &Direction3) {
        self.write(|s| {
            if s.direction.is_equal(direction, 0.0) {
                return;
            }
            let center = s.center();
            s.direction = *direction;
            s.eye = Point3::from(center.xyz() - direction.xyz() * s.distance);
            s.invalidate_orientation();
        });
    }
    /// The up direction; `camera.up()`.
    pub fn up(&self) -> Direction3 {
        self.read(|s| s.up)
    }
    /// Sets the up direction; `camera.set_up(&up)`.
    pub fn set_up(&self, up: &Direction3) {
        self.write(|s| {
            if !s.up.is_equal(up, 0.0) {
                s.up = *up;
                s.invalidate_orientation();
            }
        });
    }
    /// The up direction made orthogonal to the view direction; an up parallel to the
    /// direction raises the `gp_Dir` construction error, as natively. `camera.orthogonalized_up()?`.
    pub fn orthogonalized_up(&self) -> Result<Direction3, Failure> {
        let (direction, up) = self.read(|s| (s.direction, s.up));
        direction.crossed(up)?.crossed(direction)
    }
    /// Sets the orthogonalized up; `camera.orthogonalize_up()?`.
    pub fn orthogonalize_up(&self) -> Result<(), Failure> {
        self.set_up(&self.orthogonalized_up()?);
        Ok(())
    }
    /// The right side, `-(direction ^ up)`; `camera.side_right()?`.
    pub fn side_right(&self) -> Result<Direction3, Failure> {
        Ok(self
            .direction()
            .crossed(self.orthogonalized_up()?)?
            .reversed())
    }
    /// The eye; `camera.eye()`.
    pub fn eye(&self) -> Point3 {
        self.read(|s| s.eye)
    }
    /// Moves the eye keeping the direction (the center moves); `camera.move_eye_to(&p)`.
    pub fn move_eye_to(&self, eye: &Point3) {
        self.write(|s| {
            if !s.eye.is_equal(eye, 0.0) {
                s.eye = *eye;
                s.invalidate_orientation();
            }
        });
    }
    /// The center; `camera.center()`.
    pub fn center(&self) -> Point3 {
        self.read(Graphic3dCameraState::center)
    }
    /// The eye-to-center distance; `camera.distance()`.
    pub fn distance(&self) -> f64 {
        self.read(|s| s.distance)
    }
    /// The axial scale; `camera.axial_scale()`.
    pub fn axial_scale(&self) -> Xyz {
        self.read(|s| s.axial_scale)
    }
    /// `Transform`: moves up, direction and eye (identity transformations are ignored);
    /// `camera.transform(&trsf)`.
    pub fn transform(&self, transform: &Transform3) {
        if transform.form() == TransformForm::Identity {
            return;
        }
        self.write(|s| {
            s.up.transform(*transform);
            s.direction.transform(*transform);
            s.eye.transform(*transform);
            s.invalidate_orientation();
        });
    }
}
