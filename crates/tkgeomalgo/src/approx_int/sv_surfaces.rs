//! `ApproxInt_SvSurfaces`: the deferred class that the multi-line calls when the
//! approximation needs more points on an intersection line (docs/geomalgo-design.md
//! section 20).
use crate::int_surf::IntSurfPntOn2S;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point3, Vector2, Vector3};
use std::cell::RefCell;
use std::rc::Rc;

/// The data of the base class `ApproxInt_SvSurfaces`: the `myUseSolver` flag, false
/// after the constructor.
///
/// ```
/// use rustycad_tkgeomalgo::approx_int::ApproxIntSvSurfacesBase;
/// assert!(!ApproxIntSvSurfacesBase::default().use_solver);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ApproxIntSvSurfacesBase {
    /// `myUseSolver`: whether the SvSurfaces refine a point with a solver.
    pub use_solver: bool,
}

/// The output references of `Compute(u1, v1, u2, v2, Pt, Tg, Tguv1, Tguv2)`: the point
/// of the line, its 3d tangent and its tangents in the parameter planes. The caller
/// keeps the record, so a failing `Compute` leaves what it wrote, as natively.
///
/// ```
/// use rustycad_tkgeomalgo::approx_int::ApproxIntSvComputed;
/// let out = ApproxIntSvComputed::default();
/// assert_eq!(out.tangent.x(), 0.0);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ApproxIntSvComputed {
    /// `Pt`.
    pub point: Point3,
    /// `Tg`.
    pub tangent: Vector3,
    /// `Tguv1`: the tangent on the first surface.
    pub tangent_uv1: Vector2,
    /// `Tguv2`: the tangent on the second surface.
    pub tangent_uv2: Vector2,
}

/// `ApproxInt_SvSurfaces`: computes points and tangents of the intersection line of two
/// surfaces near given parameters. `SetUseSolver` (non-virtual) and `GetUseSolver`
/// (virtual) are defaulted over the embedded [`ApproxIntSvSurfacesBase`].
///
/// ```ignore
/// let mut out = ApproxIntSvComputed::default();
/// let mut uv = [0.5, 0.5, 0.5, 0.5];
/// if sv.compute(&mut uv, &mut out)? { println!("{:?}", out.tangent); }
/// ```
pub trait ApproxIntSvSurfaces {
    /// The embedded `ApproxInt_SvSurfaces` data.
    fn sv_base(&self) -> &ApproxIntSvSurfacesBase;
    /// The embedded `ApproxInt_SvSurfaces` data, to change.
    fn sv_base_mut(&mut self) -> &mut ApproxIntSvSurfacesBase;
    /// `Compute(u1, v1, u2, v2, Pt, Tg, Tguv1, Tguv2)`: true when the tangents can be
    /// computed; `uv` holds `u1 v1 u2 v2` and gets the parameters of the point found.
    fn compute(
        &mut self,
        uv: &mut [f64; 4],
        out: &mut ApproxIntSvComputed,
    ) -> Result<bool, Failure>;
    /// `Pnt(u1, v1, u2, v2, P)`: the point of the last `Compute` at these parameters.
    fn pnt(&mut self, uv: [f64; 4]) -> Result<Point3, Failure>;
    /// `SeekPoint(u1, v1, u2, v2, Point)`: the point of the line and its parameters on
    /// the surfaces, None when it cannot be computed (the output is left unchanged).
    fn seek_point(&mut self, uv: [f64; 4]) -> Result<Option<IntSurfPntOn2S>, Failure>;
    /// `Tangency(u1, v1, u2, v2, Tg)`: the flag of `Compute` and the stored 3d tangent.
    fn tangency(&mut self, uv: [f64; 4]) -> Result<(bool, Vector3), Failure>;
    /// `TangencyOnSurf1(u1, v1, u2, v2, Tg)`: the flag and the tangent on S1.
    fn tangency_on_surf1(&mut self, uv: [f64; 4]) -> Result<(bool, Vector2), Failure>;
    /// `TangencyOnSurf2(u1, v1, u2, v2, Tg)`: the flag and the tangent on S2.
    fn tangency_on_surf2(&mut self, uv: [f64; 4]) -> Result<(bool, Vector2), Failure>;
    /// `SetUseSolver(theUseSol)`.
    fn set_use_solver(&mut self, use_solver: bool) {
        self.sv_base_mut().use_solver = use_solver;
    }
    /// `GetUseSolver()`.
    fn get_use_solver(&self) -> bool {
        self.sv_base().use_solver
    }
}

/// The native `void*` to an `ApproxInt_SvSurfaces` that a multi-line and its sub-lines
/// share: they change its `UseSolver` flag while they insert points.
pub type SvSurfacesRef = Rc<RefCell<dyn ApproxIntSvSurfaces>>;
