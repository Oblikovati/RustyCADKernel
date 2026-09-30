// Rust translation of ModelingData/TKGeomBase/ExtremaPC2d/ExtremaPC2d_Parabola.hxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ExtremaPC2d_Parabola`: the extrema on a parabola from a cubic.
use super::{CurveEvaluator, Domain1D, ExtremaResult, ExtremumResult, SearchMode, Status};
use crate::extrema_pc_core::NeighborStep;
use crate::extrema_pc_core::analytic::{self, AnalyticEvaluator};
use crate::extrema_pc_core::conic_roots::{PolynomialRoots, add_polynomial_extrema};
use rustycad_tkmath::gp::{self, Parabola2, Point2, Vector2};
use rustycad_tkmath::math_poly::{PolyResult, cubic};
use std::convert::Infallible;

/// Point-parabola extrema (`ExtremaPC2d_Parabola`): for `C(u) = (u^2 / 4F, u)` the
/// roots of `u^3 / (4F) + (2F - X) u - 2F Y = 0` (up to three extrema). A parabola of
/// focal length below gp::Resolution() is its mirror axis, a line;
/// `Parabola::new(parabola).perform(point, 1e-9, SearchMode::MinMax)`.
pub struct Parabola {
    parabola: Parabola2,
    domain: Option<Domain1D>,
    result: ExtremaResult,
    cache: ParabolaCache,
}

/// The cached geometry of `cacheGeometry`.
#[derive(Clone, Copy)]
struct ParabolaCache {
    vertex: (f64, f64),
    x_dir: (f64, f64),
    y_dir: (f64, f64),
    focal: f64,
    one_over_4f: f64,
    two_f: f64,
}

impl ParabolaCache {
    fn new(parabola: &Parabola2) -> Self {
        let vertex: Point2 = parabola.location();
        let axis = parabola.axis();
        let (x_dir, y_dir) = (axis.x_direction(), axis.y_direction());
        let focal: f64 = parabola.focal();
        Self {
            vertex: (vertex.x(), vertex.y()),
            x_dir: (x_dir.x(), x_dir.y()),
            y_dir: (y_dir.x(), y_dir.y()),
            focal,
            one_over_4f: if focal > gp::resolution() {
                1.0 / (4.0 * focal)
            } else {
                0.0
            },
            two_f: 2.0 * focal,
        }
    }
}

impl Parabola {
    /// Evaluator on the whole parabola (`ExtremaPC2d_Parabola(theParabola)`);
    /// `Parabola::new(parabola)`.
    pub fn new(parabola: Parabola2) -> Self {
        Self {
            cache: ParabolaCache::new(&parabola),
            parabola,
            domain: None,
            result: ExtremaResult::default(),
        }
    }
    /// Evaluator on `domain` (`ExtremaPC2d_Parabola(theParabola, theDomain)`);
    /// `Parabola::with_domain(parabola, Domain1D::new(-1.0, 1.0))`.
    pub fn with_domain(parabola: Parabola2, domain: Domain1D) -> Self {
        Self {
            domain: Some(domain),
            ..Self::new(parabola)
        }
    }
    /// The point `V + (u^2 / 4F) X + u Y` from the cached geometry, a point of the mirror
    /// axis for a degenerate focal (`Value`); `extrema.value(0.5)`.
    pub fn value(&self, u: f64) -> Point2 {
        let c = &self.cache;
        if c.focal <= gp::resolution() {
            let direction = Vector2::from(self.parabola.axis_of_symmetry().direction());
            return self.parabola.location().translated(direction * u);
        }
        let x: f64 = u * u * c.one_over_4f;
        Point2::new(
            c.vertex.0 + x * c.x_dir.0 + u * c.y_dir.0,
            c.vertex.1 + x * c.x_dir.1 + u * c.y_dir.1,
        )
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
    pub fn parabola(&self) -> &Parabola2 {
        &self.parabola
    }
    /// The interior extrema (`Perform`); `extrema.perform(point, 1e-9, SearchMode::MinMax)`.
    pub fn perform(&mut self, point: Point2, tol: f64, mode: SearchMode) -> &ExtremaResult {
        self.result.clear();
        if super::is_invalid_input(tol, &point, self.domain.as_ref()) {
            self.result.status = Status::InvalidInput;
            return &self.result;
        }
        if self.cache.focal <= gp::resolution() {
            self.perform_degenerate(point, tol, mode);
            return &self.result;
        }
        let roots: PolyResult = self.solve_cubic(point);
        let mut result: ExtremaResult = std::mem::take(&mut self.result);
        add_polynomial_extrema(
            &mut result,
            &PolynomialRoots::direct(&roots),
            (&point, self.domain.as_ref(), mode),
            |u| self.value(u),
        );
        self.result = result;
        &self.result
    }
    /// The interior extrema and the extremal ends of a bounded arc
    /// (`PerformWithEndpoints`); `extrema.perform_with_endpoints(point, 1e-9, SearchMode::MinMax)`.
    pub fn perform_with_endpoints(
        &mut self,
        point: Point2,
        tol: f64,
        mode: SearchMode,
    ) -> &ExtremaResult {
        let Ok(result) = analytic::perform_with_endpoints(self, point, tol, mode);
        result
    }
    // A degenerate parabola: the projection on the mirror axis, within the domain
    // widened by `tol`.
    fn perform_degenerate(&mut self, point: Point2, tol: f64, mode: SearchMode) {
        if mode == SearchMode::Max {
            self.result.status = Status::NoSolution;
            return;
        }
        let vertex: Point2 = self.parabola.location();
        let direction = Vector2::from(self.parabola.axis_of_symmetry().direction());
        let mut u: f64 = Vector2::between(vertex, point).dot(direction);
        if let Some(domain) = self.domain {
            if !domain.contains(u, tol) {
                self.result.status = Status::NoSolution;
                return;
            }
            u = domain.clamp(u);
        }
        let on_curve: Point2 = self.value(u);
        let extremum =
            ExtremumResult::new(u, on_curve, point.square_distance(on_curve), true, false);
        self.result.extrema.push(extremum);
        self.result.status = Status::OK;
    }
    // `solveCubic`: `(1/(4F)) u^3 + (2F - X) u - 2F Y` in the local query coordinates.
    fn solve_cubic(&self, point: Point2) -> PolyResult {
        let c = &self.cache;
        let (dx, dy) = (point.x() - c.vertex.0, point.y() - c.vertex.1);
        let x: f64 = dx * c.x_dir.0 + dy * c.x_dir.1;
        let y: f64 = dx * c.y_dir.0 + dy * c.y_dir.1;
        cubic(c.one_over_4f, 0.0, c.two_f - x, -c.two_f * y)
    }
}

impl CurveEvaluator<Point2> for Parabola {
    type Error = Infallible;
    fn evaluate(&self, u: f64) -> Result<Point2, Infallible> {
        Ok(self.value(u))
    }
}

impl AnalyticEvaluator<Point2> for Parabola {
    const IS_PERIODIC: bool = false;
    const NEIGHBOR_STEP: NeighborStep = super::neighbor_step;
    type PerformError = Infallible;
    fn fixed_domain(&self) -> Option<Domain1D> {
        self.domain
    }
    fn result_slot(&mut self) -> &mut ExtremaResult {
        &mut self.result
    }
    fn perform_interior(
        &mut self,
        point: Point2,
        tol: f64,
        mode: SearchMode,
    ) -> Result<(), Infallible> {
        let _ = self.perform(point, tol, mode);
        Ok(())
    }
}
