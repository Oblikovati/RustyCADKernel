// Rust translation of ModelingData/TKGeomBase/ExtremaPC/ExtremaPC_Parabola.hxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ExtremaPC_Parabola`: the extrema on a parabola in space.
use super::conic_frame::ConicFrame;
use super::planar::{PlanarQuery, perform_planar};
use super::{CurveEvaluator, Domain1D, ExtremaResult, ExtremumResult, SearchMode, Status};
use crate::extrema_pc_core::analytic::{self, AnalyticEvaluator};
use crate::extrema_pc_core::conic_roots::{PolynomialRoots, add_polynomial_extrema};
use crate::extrema_pc_core::{NeighborStep, ValueFn};
use crate::extrema_pc2d;
use crate::proj_lib as projlib;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{self, Axis3, Parabola3, Plane3, Point3, Vector3};
use rustycad_tkmath::math_poly::{PolyResult, cubic};
use std::convert::Infallible;

/// Point-parabola extrema (`ExtremaPC_Parabola`): solved by `ExtremaPC2d_Parabola` in
/// the parabola plane; when that fails, the cubic of the 2D evaluator for the point
/// projected on the plane. A focal length below gp::Resolution() is the X axis line;
/// `Parabola::new(parabola).perform(point, 1e-9, SearchMode::MinMax)?`.
pub struct Parabola {
    parabola: Parabola3,
    domain: Option<Domain1D>,
    result: ExtremaResult,
    shape: ParabolaShape,
}

/// The cached geometry of `cacheGeometry` and `Value`.
#[derive(Clone, Copy)]
struct ParabolaShape {
    frame: ConicFrame,
    vertex: Point3,
    x_direction: Vector3,
    focal: f64,
    one_over_4f: f64,
    two_f: f64,
}

impl ParabolaShape {
    fn new(parabola: &Parabola3) -> Self {
        let focal: f64 = parabola.focal();
        Self {
            frame: ConicFrame::of(parabola.position()),
            vertex: parabola.location(),
            x_direction: Vector3::from(parabola.x_axis().direction()),
            focal,
            one_over_4f: if focal > gp::resolution() {
                1.0 / (4.0 * focal)
            } else {
                0.0
            },
            two_f: 2.0 * focal,
        }
    }
    // `Value`: the X axis line for a degenerate focal, else `V + (u^2/4F) X + u Y`.
    fn value(&self, u: f64) -> Point3 {
        if self.focal <= gp::resolution() {
            return self.vertex.translated(self.x_direction * u);
        }
        self.frame.point(u * u * self.one_over_4f, u)
    }
}

impl Parabola {
    /// Evaluator on the whole parabola (`ExtremaPC_Parabola(theParabola)`);
    /// `Parabola::new(parabola)`.
    pub fn new(parabola: Parabola3) -> Self {
        Self {
            shape: ParabolaShape::new(&parabola),
            parabola,
            domain: None,
            result: ExtremaResult::default(),
        }
    }
    /// Evaluator on `domain` (`ExtremaPC_Parabola(theParabola, theDomain)`);
    /// `Parabola::with_domain(parabola, Domain1D::new(-1.0, 1.0))`.
    pub fn with_domain(parabola: Parabola3, domain: Domain1D) -> Self {
        Self {
            domain: Some(domain),
            ..Self::new(parabola)
        }
    }
    /// The point at `u` from the cached geometry (`Value`); `extrema.value(0.5)`.
    pub fn value(&self, u: f64) -> Point3 {
        self.shape.value(u)
    }
    /// True with a domain (`IsBounded`); `extrema.is_bounded()`.
    pub fn is_bounded(&self) -> bool {
        self.domain.is_some()
    }
    /// The domain (`Domain`); `extrema.domain()`.
    pub fn domain(&self) -> Option<&Domain1D> {
        self.domain.as_ref()
    }
    /// The parabola (`Parabola`); `extrema.parabola()`.
    pub fn parabola(&self) -> &Parabola3 {
        &self.parabola
    }
    /// The interior extrema (`Perform`); `extrema.perform(point, 1e-9, SearchMode::MinMax)?`.
    pub fn perform(
        &mut self,
        point: Point3,
        tol: f64,
        mode: SearchMode,
    ) -> Result<&ExtremaResult, Failure> {
        let valid: bool = !super::is_invalid_input(tol, &point, self.domain.as_ref());
        if valid && self.perform_in_plane(point, tol, mode)? {
            return Ok(&self.result);
        }
        self.perform_core(point, tol, mode);
        Ok(&self.result)
    }
    /// The interior extrema and the extremal ends of a bounded arc
    /// (`PerformWithEndpoints`); `extrema.perform_with_endpoints(point, 1e-9, SearchMode::MinMax)?`.
    pub fn perform_with_endpoints(
        &mut self,
        point: Point3,
        tol: f64,
        mode: SearchMode,
    ) -> Result<&ExtremaResult, Failure> {
        analytic::perform_with_endpoints(self, point, tol, mode)
    }
    // ExtremaPC2d_Parabola in the parabola plane.
    fn perform_in_plane(
        &mut self,
        point: Point3,
        tol: f64,
        mode: SearchMode,
    ) -> Result<bool, Failure> {
        let plane = Plane3::from_position(Axis3::from_axis2(self.parabola.position()));
        let parabola2d = projlib::project_parabola_on_plane(&plane, &self.parabola)?;
        let query = PlanarQuery {
            point2d: projlib::project_point_on_plane(&plane, point),
            point3d: point,
            plane: &plane,
            tol,
            mode,
            with_endpoints: false,
        };
        let shape = self.shape;
        let evaluator = ValueFn(|u| Ok::<Point3, Infallible>(shape.value(u)));
        let Ok(done) = perform_planar::<extrema_pc2d::Parabola, _>(
            parabola2d,
            self.domain,
            &query,
            &evaluator,
            &mut self.result,
        );
        Ok(done)
    }
    // `performCore`: the degenerate line or the cubic of the projected point.
    fn perform_core(&mut self, point: Point3, tol: f64, mode: SearchMode) {
        self.result.clear();
        if super::is_invalid_input(tol, &point, self.domain.as_ref()) {
            self.result.status = Status::InvalidInput;
            return;
        }
        if self.shape.focal <= gp::resolution() {
            self.perform_degenerate(point, tol, mode);
            return;
        }
        let (x, y) = self.shape.frame.local_coordinates(point);
        let s = &self.shape;
        let roots: PolyResult = cubic(s.one_over_4f, 0.0, s.two_f - x, -s.two_f * y);
        let mut result: ExtremaResult = std::mem::take(&mut self.result);
        let shape = self.shape;
        add_polynomial_extrema(
            &mut result,
            &PolynomialRoots::direct(&roots),
            (&point, self.domain.as_ref(), mode),
            |u| shape.value(u),
        );
        self.result = result;
    }
    // A degenerate parabola: the projection on the X axis line, within the domain
    // widened by `tol`.
    fn perform_degenerate(&mut self, point: Point3, tol: f64, mode: SearchMode) {
        if mode == SearchMode::Max {
            self.result.status = Status::NoSolution;
            return;
        }
        let mut u: f64 = Vector3::between(self.shape.vertex, point).dot(self.shape.x_direction);
        if let Some(domain) = self.domain {
            if !domain.contains(u, tol) {
                self.result.status = Status::NoSolution;
                return;
            }
            u = domain.clamp(u);
        }
        let on_curve: Point3 = self.shape.value(u);
        let extremum =
            ExtremumResult::new(u, on_curve, point.square_distance(&on_curve), true, false);
        self.result.extrema.push(extremum);
        self.result.status = Status::OK;
    }
}

impl CurveEvaluator<Point3> for Parabola {
    type Error = Infallible;
    fn evaluate(&self, u: f64) -> Result<Point3, Infallible> {
        Ok(self.value(u))
    }
}

impl AnalyticEvaluator<Point3> for Parabola {
    const IS_PERIODIC: bool = false;
    const NEIGHBOR_STEP: NeighborStep = super::neighbor_step;
    type PerformError = Failure;
    fn fixed_domain(&self) -> Option<Domain1D> {
        self.domain
    }
    fn result_slot(&mut self) -> &mut ExtremaResult {
        &mut self.result
    }
    fn perform_interior(
        &mut self,
        point: Point3,
        tol: f64,
        mode: SearchMode,
    ) -> Result<(), Failure> {
        self.perform(point, tol, mode).map(|_| ())
    }
}
