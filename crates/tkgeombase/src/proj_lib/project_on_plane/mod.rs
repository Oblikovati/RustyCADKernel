// Rust translation of ModelingData/TKGeomBase/ProjLib/ProjLib_ProjectOnPlane.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ProjLib_ProjectOnPlane`: the projection of a 3D curve onto a plane along a
//! direction, as a 3D curve adaptor. Lines, conics, Bezier and B-spline curves give
//! an exact result when the projection keeps their kind; other cases are
//! approximated by a B-spline.
mod adaptor;
pub(super) mod approx;
mod conics;
mod eval;
mod load;

use eval::Oblique;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Axis3, Direction3};

/// The fields of one `ProjLib_ProjectOnPlane`.
#[derive(Clone)]
pub(super) struct OnPlaneState {
    pub(super) curve: Handle<dyn Adaptor3dCurve>,
    pub(super) plane: Axis3,
    pub(super) direction: Direction3,
    pub(super) keep_param: bool,
    pub(super) first_par: f64,
    pub(super) last_par: f64,
    pub(super) tolerance: f64,
    pub(super) curve_type: CurveType,
    pub(super) result: Handle<GeomAdaptorCurve>,
    pub(super) is_approx: bool,
}

impl OnPlaneState {
    fn new(plane: Axis3, direction: Direction3) -> Self {
        Self {
            curve: Handle::default(),
            plane,
            direction,
            keep_param: false,
            first_par: 0.,
            last_par: 0.,
            tolerance: 0.,
            curve_type: CurveType::OtherCurve,
            result: Handle::default(),
            is_approx: false,
        }
    }
    fn oblique(&self) -> Oblique {
        Oblique {
            plane: self.plane,
            direction: self.direction,
        }
    }
    /// The loaded curve; natively a null one is dereferenced.
    pub(super) fn curve(&self) -> Result<&dyn Adaptor3dCurve, Failure> {
        self.curve.get().ok_or_else(|| {
            crate::native_failure::failure(
                "Standard_NullObject",
                "ProjLib_ProjectOnPlane: no curve is loaded; expected Load(C, Tol)",
            )
        })
    }
    /// The projected curve adaptor; natively a null one is dereferenced.
    pub(super) fn result(&self) -> Result<&GeomAdaptorCurve, Failure> {
        self.result.get().ok_or_else(|| {
            crate::native_failure::failure(
                "Standard_NullObject",
                "ProjLib_ProjectOnPlane: the result is null; expected a loaded projection",
            )
        })
    }
    /// `ShallowCopy()` on the plain state: the curve and the result are shallow
    /// copies, the other fields are copied.
    pub(super) fn shallow_copy(&self) -> Result<OnPlaneState, Failure> {
        let mut copy: OnPlaneState = OnPlaneState::new(self.plane, self.direction);
        if let Some(curve) = self.curve.get() {
            copy.curve = curve.shallow_copy()?;
        }
        (copy.keep_param, copy.first_par, copy.last_par) =
            (self.keep_param, self.first_par, self.last_par);
        (copy.tolerance, copy.curve_type) = (self.tolerance, self.curve_type);
        if let Some(result) = self.result.get() {
            copy.result = Handle::new(result.shallow_adaptor()?);
        }
        copy.is_approx = self.is_approx;
        Ok(copy)
    }
}

/// `ProjLib_ProjectOnPlane`: an `Adaptor3d_Curve` whose members answer for the
/// projected curve.
///
/// ```ignore
/// let projection = ProjectOnPlane::with_plane(&plane);
/// projection.load(&curve, 1e-7, true)?;
/// let point = projection.value(0.5)?;
/// ```
pub struct ProjectOnPlane {
    parent: StandardTransient,
    state: SharedState<OnPlaneState>,
}
impl_transient!(
    ProjectOnPlane,
    "ProjLib_ProjectOnPlane",
    dyn Adaptor3dCurve,
    parent,
    interfaces: [dyn Adaptor3dCurve]
);

impl Default for ProjectOnPlane {
    fn default() -> Self {
        Self::from_state(OnPlaneState::new(Axis3::default(), Direction3::default()))
    }
}

impl ProjectOnPlane {
    pub(super) fn from_state(state: OnPlaneState) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }
    /// `ProjLib_ProjectOnPlane()`: the default plane and direction;
    /// `ProjectOnPlane::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `ProjLib_ProjectOnPlane(Pl)`: the projection normal to the plane `Pl`;
    /// `ProjectOnPlane::with_plane(&plane)`.
    pub fn with_plane(plane: &Axis3) -> Self {
        Self::from_state(OnPlaneState::new(*plane, plane.direction()))
    }
    /// `ProjLib_ProjectOnPlane(Pl, D)`: the projection along `D`. A direction
    /// parallel to the plane is not refused (the native check is commented out);
    /// `ProjectOnPlane::with_direction(&plane, direction)`.
    pub fn with_direction(plane: &Axis3, direction: Direction3) -> Self {
        Self::from_state(OnPlaneState::new(*plane, direction))
    }
    /// `GetPlane()`; `projection.get_plane()`.
    pub fn get_plane(&self) -> Axis3 {
        self.state.read().plane
    }
    /// `GetDirection()`; `projection.get_direction()`.
    pub fn get_direction(&self) -> Direction3 {
        self.state.read().direction
    }
    /// `GetCurve()`: the loaded curve, null before Load; `projection.get_curve()`.
    pub fn get_curve(&self) -> Handle<dyn Adaptor3dCurve> {
        self.state.read().curve.clone()
    }
    /// `GetResult()`: the projected curve adaptor, null before Load;
    /// `projection.get_result()`.
    pub fn get_result(&self) -> Handle<GeomAdaptorCurve> {
        self.state.read().result.clone()
    }

    /// `Load(C, Tolerance, KeepParametrization)`: sets the curve and computes the
    /// projection. With `keep_parametrization`, `proj(C(u)) = PC(u)` for every u;
    /// otherwise the parametrization may change (a canonical ellipse, for example).
    /// The state is updated even when the computation fails part way, as natively;
    /// `projection.load(&curve, 1e-7, true)?`.
    pub fn load(
        &self,
        curve: &Handle<dyn Adaptor3dCurve>,
        tolerance: f64,
        keep_parametrization: bool,
    ) -> Result<(), Failure> {
        let mut state: OnPlaneState = self.state.read().clone();
        let outcome = state.load(curve, tolerance, keep_parametrization);
        *self.state.write() = state;
        outcome
    }
}
