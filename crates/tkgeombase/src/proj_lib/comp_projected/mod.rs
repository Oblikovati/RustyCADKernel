// Rust translation of ModelingData/TKGeomBase/ProjLib/ProjLib_CompProjectedCurve.hxx/.cxx
// and ProjLib_HCompProjectedCurve.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ProjLib_CompProjectedCurve`: the orthogonal projection of a 3D curve on any
//! surface as a 2D adaptor. `Init` walks the curve and records the continuous parts
//! of the projection as `(t, u, v)` triples; the curve is evaluated by solving the
//! projection from an interpolation of those triples.
mod accessors;
mod adaptor;
mod advance;
mod bounds;
mod derivatives;
mod eval;
mod init;
mod intervals;
mod parts;
mod perform;
mod post;
mod splits;
mod start;

use crate::native_failure::failure;
use derivatives::Pair;
pub use parts::SequenceOfSequenceOfPnt;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point2, Point3};

/// `#define FuncTol 1.e-10`: the residual tolerance of every PrjResolve call.
const FUNC_TOL: f64 = 1.0e-10;

/// The per-part results of `Perform`.
#[derive(Clone, Default)]
pub(super) struct PerformResults {
    pub(super) is_point: Option<Vec<bool>>,
    pub(super) error_2d_u: Option<Vec<f64>>,
    pub(super) error_2d_v: Option<Vec<f64>>,
    pub(super) error_3d: Option<Vec<f64>>,
    pub(super) point_3d: Option<Vec<Point3>>,
    pub(super) point_2d: Option<Vec<Point2>>,
    pub(super) curve_3d: Option<Vec<Handle<dyn GeomCurve>>>,
    pub(super) curve_2d: Option<Vec<Handle<dyn Geom2dCurve>>>,
}

/// The fields of one `ProjLib_CompProjectedCurve`; a `None` array is the native
/// null handle.
#[derive(Clone)]
pub(super) struct CompState {
    pub(super) surface: Handle<dyn Adaptor3dSurface>,
    pub(super) curve: Handle<dyn Adaptor3dCurve>,
    pub(super) nb_curves: i32,
    pub(super) sequence: Handle<SequenceOfSequenceOfPnt>,
    pub(super) u_iso: Option<Vec<bool>>,
    pub(super) v_iso: Option<Vec<bool>>,
    pub(super) sngl_pnts: Option<Vec<bool>>,
    pub(super) max_distance: Option<Vec<f64>>,
    pub(super) tab_int: Option<Vec<f64>>,
    pub(super) tol3d: f64,
    pub(super) continuity: Shape,
    pub(super) max_degree: i32,
    pub(super) max_seg: i32,
    pub(super) proj2d: bool,
    pub(super) proj3d: bool,
    pub(super) max_dist: f64,
    pub(super) tol_u: f64,
    pub(super) tol_v: f64,
    pub(super) results: PerformResults,
}

impl CompState {
    // The native default constructor leaves the approximation options
    // uninitialized; the port gives them the values of the other constructors.
    fn empty() -> Self {
        Self {
            surface: Handle::default(),
            curve: Handle::default(),
            nb_curves: 0,
            sequence: Handle::default(),
            u_iso: None,
            v_iso: None,
            sngl_pnts: None,
            max_distance: None,
            tab_int: None,
            tol3d: 1.0e-6,
            continuity: Shape::C2,
            max_degree: 14,
            max_seg: 16,
            proj2d: true,
            proj3d: false,
            max_dist: 0.0,
            tol_u: 0.0,
            tol_v: 0.0,
            results: PerformResults::default(),
        }
    }
    fn with_data(
        surface: &Handle<dyn Adaptor3dSurface>,
        curve: &Handle<dyn Adaptor3dCurve>,
        max_dist: f64,
    ) -> Self {
        Self {
            surface: surface.clone(),
            curve: curve.clone(),
            sequence: Handle::new(SequenceOfSequenceOfPnt::default()),
            max_dist,
            ..Self::empty()
        }
    }
    /// The curve and surface; Standard_NullObject when one is missing.
    fn pair(&self) -> Result<Pair<'_>, Failure> {
        let curve = self.curve.get().ok_or_else(|| null("the curve"))?;
        let surface = self.surface.get().ok_or_else(|| null("the surface"))?;
        Ok(Pair { curve, surface })
    }
    /// The sequence of parts; Standard_NullObject for a default-constructed curve.
    fn parts(&self) -> Result<&SequenceOfSequenceOfPnt, Failure> {
        self.sequence
            .get()
            .ok_or_else(|| null("the sequence of projected points"))
    }
}

fn null(what: &str) -> Failure {
    failure(
        "Standard_NullObject",
        &format!("ProjLib_CompProjectedCurve: {what} is null; expected a constructed projection"),
    )
}

/// `ProjLib_CompProjectedCurve` (and its alias `ProjLib_HCompProjectedCurve`).
///
/// ```ignore
/// let projection = CompProjectedCurve::with_tol3d(1e-6, &surface, &curve, -1.0)?;
/// for index in 1..=projection.nb_curves() {
///     let (first, last) = projection.bounds(index)?;
/// }
/// ```
pub struct CompProjectedCurve {
    parent: StandardTransient,
    state: SharedState<CompState>,
}
impl_transient!(
    CompProjectedCurve,
    "ProjLib_CompProjectedCurve",
    dyn Adaptor2dCurve2d,
    parent,
    interfaces: [dyn Adaptor2dCurve2d]
);

/// `ProjLib_HCompProjectedCurve`: a typedef of the class.
pub type HCompProjectedCurve = CompProjectedCurve;

impl Default for CompProjectedCurve {
    fn default() -> Self {
        Self::from_state(CompState::empty())
    }
}

impl CompProjectedCurve {
    pub(super) fn from_state(state: CompState) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }
    pub(super) fn snapshot(&self) -> CompState {
        self.state.read().clone()
    }
    /// `ProjLib_CompProjectedCurve()`: no curve, no surface, no sequence;
    /// `CompProjectedCurve::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `ProjLib_CompProjectedCurve(S, C, TolU, TolV)`: all the solutions;
    /// `CompProjectedCurve::with_tolerances(&surface, &curve, 1e-7, 1e-7)?`.
    pub fn with_tolerances(
        surface: &Handle<dyn Adaptor3dSurface>,
        curve: &Handle<dyn Adaptor3dCurve>,
        tol_u: f64,
        tol_v: f64,
    ) -> Result<Self, Failure> {
        Self::with_max_dist(surface, curve, tol_u, tol_v, -1.0)
    }
    /// `ProjLib_CompProjectedCurve(S, C, TolU, TolV, MaxDist)`: the search assumes the
    /// curve within `max_dist` of the surface (all solutions when negative);
    /// `CompProjectedCurve::with_max_dist(&surface, &curve, 1e-7, 1e-7, 0.1)?`.
    pub fn with_max_dist(
        surface: &Handle<dyn Adaptor3dSurface>,
        curve: &Handle<dyn Adaptor3dCurve>,
        tol_u: f64,
        tol_v: f64,
        max_dist: f64,
    ) -> Result<Self, Failure> {
        let state = CompState {
            tol_u,
            tol_v,
            ..CompState::with_data(surface, curve, max_dist)
        };
        let projection: Self = Self::from_state(state);
        projection.init()?;
        Ok(projection)
    }
    /// `ProjLib_CompProjectedCurve(Tol3d, S, C, MaxDist = -1)`: the parametric
    /// tolerances are the surface resolutions of `tol3d` (at least PConfusion);
    /// `CompProjectedCurve::with_tol3d(1e-6, &surface, &curve, -1.0)?`.
    pub fn with_tol3d(
        tol3d: f64,
        surface: &Handle<dyn Adaptor3dSurface>,
        curve: &Handle<dyn Adaptor3dCurve>,
        max_dist: f64,
    ) -> Result<Self, Failure> {
        let mut state = CompState {
            tol3d,
            ..CompState::with_data(surface, curve, max_dist)
        };
        let s: &dyn Adaptor3dSurface = surface.get().ok_or_else(|| null("the surface"))?;
        state.tol_u = precision::p_confusion().max(s.u_resolution(tol3d)?);
        state.tol_v = precision::p_confusion().max(s.v_resolution(tol3d)?);
        let projection: Self = Self::from_state(state);
        projection.init()?;
        Ok(projection)
    }

    /// `ShallowCopy()` as a concrete adaptor: shallow copies of the curve and surface,
    /// the same sequence (shared), tolerances, iso and point flags and distances; the
    /// approximation options and results are not copied, as natively.
    pub fn shallow_projection(&self) -> Result<CompProjectedCurve, Failure> {
        let source: CompState = self.snapshot();
        let mut copy: CompState = CompState::empty();
        if let Some(surface) = source.surface.get() {
            copy.surface = surface.shallow_copy()?;
        }
        if let Some(curve) = source.curve.get() {
            copy.curve = curve.shallow_copy()?;
        }
        copy.nb_curves = source.nb_curves;
        copy.sequence = source.sequence.clone();
        (copy.tol_u, copy.tol_v, copy.max_dist) = (source.tol_u, source.tol_v, source.max_dist);
        (copy.u_iso, copy.v_iso) = (source.u_iso.clone(), source.v_iso.clone());
        copy.sngl_pnts = source.sngl_pnts.clone();
        copy.max_distance = source.max_distance.clone();
        Ok(Self::from_state(copy))
    }
}
