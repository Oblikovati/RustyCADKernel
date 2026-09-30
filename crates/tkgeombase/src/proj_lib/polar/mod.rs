// Rust translation of ModelingData/TKGeomBase/ProjLib/ProjLib_ComputeApproxOnPolarSurface.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ProjLib_ComputeApproxOnPolarSurface`: the 2D curve of a 3D curve projected on a
//! surface with poles or seams (Bezier, B-spline and offset surfaces, and the
//! periodic elementary ones). A rough initial 2D curve, given or built from
//! extrema, starts each point; the refined points are fitted by Bezier pieces.
mod function;
mod initial;
mod intervals;
mod poles;
mod project;
mod start_point;
mod walk;

use crate::app_par_curves::Constraint;
use crate::native_failure::failure;
use crate::proj_lib::compute_approx::function::{live_curve, live_surface};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dBoundedCurve, Geom2dCurve, Geom2dGeometry};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkmath::gp::Point2;

/// `ProjLib_ComputeApproxOnPolarSurface`: the projected B-spline, the second pcurve
/// of a seam, and the reached tolerance.
///
/// ```ignore
/// let mut polar = ComputeApproxOnPolarSurface::new();
/// polar.set_tolerance(1e-6);
/// let init = polar.build_initial_curve2d(&curve, &surface)?;
/// let pcurve = polar.project_using_initial_curve2d(&curve, &surface, &init)?;
/// ```
#[derive(Clone)]
pub struct ComputeApproxOnPolarSurface {
    proj_is_done: bool,
    tolerance: f64,
    bspline: Handle<Geom2dBSplineCurve>,
    second_curve: Handle<dyn Geom2dCurve>,
    tol_reached: f64,
    deg_min: i32,
    deg_max: i32,
    max_segments: i32,
    max_dist: f64,
    bnd_pnt: Constraint,
    dist: f64,
}

impl Default for ComputeApproxOnPolarSurface {
    fn default() -> Self {
        Self::with_tolerance(precision::approximation())
    }
}

impl ComputeApproxOnPolarSurface {
    fn with_tolerance(tolerance: f64) -> Self {
        Self {
            proj_is_done: false,
            tolerance,
            bspline: Handle::default(),
            second_curve: Handle::default(),
            tol_reached: -1.0,
            deg_min: -1,
            deg_max: -1,
            max_segments: -1,
            max_dist: -1.,
            bnd_pnt: Constraint::TangencyPoint,
            dist: 0.,
        }
    }
    /// `ProjLib_ComputeApproxOnPolarSurface()`: tolerance Approximation;
    /// `ComputeApproxOnPolarSurface::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `ProjLib_ComputeApproxOnPolarSurface(C, S, Tol = 1.0e-4)`: performs;
    /// `ComputeApproxOnPolarSurface::with_curve(&curve, &surface, 1.0e-4)?`.
    pub fn with_curve(
        curve: &Handle<dyn Adaptor3dCurve>,
        surface: &Handle<dyn Adaptor3dSurface>,
        tol: f64,
    ) -> Result<Self, Failure> {
        let mut polar: Self = Self::with_tolerance(tol);
        polar.bspline = polar.perform_with_initial(&Handle::default(), curve, surface)?;
        Ok(polar)
    }
    /// `ProjLib_ComputeApproxOnPolarSurface(InitCurve2d, C, S, Tol)`: performs from a
    /// rough initial 2D curve;
    /// `ComputeApproxOnPolarSurface::with_initial_curve(&init, &curve, &surface, 1e-6)?`.
    pub fn with_initial_curve(
        init_curve2d: &Handle<dyn Adaptor2dCurve2d>,
        curve: &Handle<dyn Adaptor3dCurve>,
        surface: &Handle<dyn Adaptor3dSurface>,
        tol: f64,
    ) -> Result<Self, Failure> {
        let mut polar: Self = Self::with_tolerance(tol);
        polar.bspline = polar.perform_with_initial(init_curve2d, curve, surface)?;
        Ok(polar)
    }

    /// `ProjLib_ComputeApproxOnPolarSurface(InitCurve2d, InitCurve2dBis, C, S, Tol)`:
    /// the two pcurves of a seam edge. The projection of the first one is kept as
    /// the B-spline when it starts nearer to it, and translated onto the other as
    /// the second curve (and the other way round);
    /// `ComputeApproxOnPolarSurface::with_seam_curves(&init, &init_bis, &curve, &surface, 1e-6)?`.
    pub fn with_seam_curves(
        init_curve2d: &Handle<dyn Adaptor2dCurve2d>,
        init_curve2d_bis: &Handle<dyn Adaptor2dCurve2d>,
        curve: &Handle<dyn Adaptor3dCurve>,
        surface: &Handle<dyn Adaptor3dSurface>,
        tol: f64,
    ) -> Result<Self, Failure> {
        let mut polar: Self = Self::with_tolerance(tol);
        let bsc: Handle<Geom2dBSplineCurve> =
            polar.perform_with_initial(init_curve2d, curve, surface)?;
        if polar.proj_is_done {
            polar.split_seam(&bsc, init_curve2d, init_curve2d_bis)?;
        }
        Ok(polar)
    }

    fn split_seam(
        &mut self,
        bsc: &Handle<Geom2dBSplineCurve>,
        init: &Handle<dyn Adaptor2dCurve2d>,
        init_bis: &Handle<dyn Adaptor2dCurve2d>,
    ) -> Result<(), Failure> {
        let spline: &Geom2dBSplineCurve = bsc.get().ok_or_else(|| {
            failure(
                "Standard_NullObject",
                "ProjLib_ComputeApproxOnPolarSurface: the projection is null",
            )
        })?;
        let p2dproj: Point2 = spline.start_point()?;
        let p2d: Point2 = start_of(init)?;
        let p2d_bis: Point2 = start_of(init_bis)?;
        if p2dproj.distance(p2d) < p2dproj.distance(p2d_bis) {
            self.bspline = bsc.clone();
            self.second_curve = spline.translated_between(p2d, p2d_bis)?.cast();
        } else {
            self.second_curve = bsc.clone().cast();
            self.bspline = spline.translated_between(p2d_bis, p2d)?.cast();
        }
        Ok(())
    }
}

// The start point of an initial 2D curve.
fn start_of(curve: &Handle<dyn Adaptor2dCurve2d>) -> Result<Point2, Failure> {
    let adaptor: &dyn Adaptor2dCurve2d = curve.get().ok_or_else(|| {
        failure(
            "Standard_NullObject",
            "ProjLib_ComputeApproxOnPolarSurface: null initial 2D curve",
        )
    })?;
    adaptor.value(adaptor.first_parameter()?)
}

impl ComputeApproxOnPolarSurface {
    /// `SetDegree(theDegMin, theDegMax)`: a negative degree keeps 2 (min) or 8 (max);
    /// `polar.set_degree(2, 8)`.
    pub fn set_degree(&mut self, deg_min: i32, deg_max: i32) {
        (self.deg_min, self.deg_max) = (deg_min, deg_max);
    }
    /// `SetMaxSegments(theMaxSegments)`: a negative count keeps 1000;
    /// `polar.set_max_segments(100)`.
    pub fn set_max_segments(&mut self, max_segments: i32) {
        self.max_segments = max_segments;
    }
    /// `SetBndPnt(theBndPnt)`; `polar.set_bnd_pnt(Constraint::PassPoint)`.
    pub fn set_bnd_pnt(&mut self, bnd_pnt: Constraint) {
        self.bnd_pnt = bnd_pnt;
    }
    /// `SetMaxDist(theMaxDist)`: a negative distance keeps 100 * Tolerance;
    /// `polar.set_max_dist(0.1)`.
    pub fn set_max_dist(&mut self, max_dist: f64) {
        self.max_dist = max_dist;
    }
    /// `SetTolerance(theTolerance)`; `polar.set_tolerance(1e-6)`.
    pub fn set_tolerance(&mut self, tolerance: f64) {
        self.tolerance = tolerance;
    }
    /// `BSpline()`: the result (after `BuildInitialCurve2d` alone, the initial
    /// curve); `polar.bspline()`.
    pub fn bspline(&self) -> Handle<Geom2dBSplineCurve> {
        self.bspline.clone()
    }
    /// `Curve2d()`: the second pcurve of a seam; Standard_NoSuchObject unless done;
    /// `polar.curve2d()?`.
    pub fn curve2d(&self) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        if !self.proj_is_done {
            return Err(failure(
                "Standard_NoSuchObject",
                "ProjLib_ComputeApproxOnPolarSurface:2ndCurve2d",
            ));
        }
        Ok(self.second_curve.clone())
    }
    /// `IsDone()`; `polar.is_done()`.
    pub fn is_done(&self) -> bool {
        self.proj_is_done
    }
    /// `Tolerance()`: the reached tolerance, -1 before a fit; `polar.tolerance()`.
    // Native Tolerance() returns myTolReached, not the requested myTolerance.
    #[allow(clippy::misnamed_getters)]
    pub fn tolerance(&self) -> f64 {
        self.tol_reached
    }

    /// `Perform(C, S)`: the projection without an initial curve, stored as the
    /// B-spline; `polar.perform(&curve, &surface)?`.
    pub fn perform(
        &mut self,
        curve: &Handle<dyn Adaptor3dCurve>,
        surface: &Handle<dyn Adaptor3dSurface>,
    ) -> Result<(), Failure> {
        self.bspline = self.perform_with_initial(&Handle::default(), curve, surface)?;
        Ok(())
    }

    /// `Perform(InitCurve2d, C, S)`: the projected B-spline, from `init_curve2d` or
    /// from a built initial curve when it is null. A B-spline curve with several C1
    /// spans is projected span by span and the pieces concatenated; a null handle
    /// when a step fails; `polar.perform_with_initial(&init, &curve, &surface)?`.
    pub fn perform_with_initial(
        &mut self,
        init_curve2d: &Handle<dyn Adaptor2dCurve2d>,
        curve: &Handle<dyn Adaptor3dCurve>,
        surface: &Handle<dyn Adaptor3dSurface>,
    ) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
        if let Some(result) = self.perform_by_spans(init_curve2d, curve, surface)? {
            return Ok(result);
        }
        let mut ahc2d: Handle<dyn Adaptor2dCurve2d> = init_curve2d.clone();
        if init_curve2d.is_null() {
            ahc2d = self.build_initial_curve2d(curve, surface)?;
            if !self.proj_is_done {
                return Ok(Handle::default());
            }
        }
        self.project_using_initial_curve2d(curve, surface, &ahc2d)
    }
}
