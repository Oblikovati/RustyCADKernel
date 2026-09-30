// Rust translation of ModelingData/TKGeomBase/ExtremaPC2d/ExtremaPC2d_Line.hxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ExtremaPC2d_Line`: the projection of a point on a line.
use super::{CurveEvaluator, Domain1D, ExtremaResult, ExtremumResult, SearchMode, Status};
use crate::extrema_pc_core::NeighborStep;
use crate::extrema_pc_core::analytic::{self, AnalyticEvaluator};
use rustycad_tkmath::gp::{Line2, Point2, Vector2};
use std::convert::Infallible;

/// Point-line extrema (`ExtremaPC2d_Line`): the closest point is at `u = (P - O) . D`,
/// the only extremum, a minimum. The domain is fixed at construction;
/// `Line::new(line).perform(point, 1e-9, SearchMode::MinMax).nb_ext() == 1`.
pub struct Line {
    line: Line2,
    domain: Option<Domain1D>,
    result: ExtremaResult,
}

impl Line {
    /// Evaluator on the unbounded line (`ExtremaPC2d_Line(theLine)`); `Line::new(line)`.
    pub fn new(line: Line2) -> Self {
        Self {
            line,
            domain: None,
            result: ExtremaResult::default(),
        }
    }
    /// Evaluator on `domain` (`ExtremaPC2d_Line(theLine, theDomain)`);
    /// `Line::with_domain(line, Domain1D::new(0.0, 1.0))`.
    pub fn with_domain(line: Line2, domain: Domain1D) -> Self {
        Self {
            domain: Some(domain),
            ..Self::new(line)
        }
    }
    /// The point at `u` (`Value`); `extrema.value(0.5)`.
    pub fn value(&self, u: f64) -> Point2 {
        self.line
            .location()
            .translated(Vector2::from(self.line.direction()) * u)
    }
    /// True with a domain (`IsBounded`); `extrema.is_bounded()`.
    pub fn is_bounded(&self) -> bool {
        self.domain.is_some()
    }
    /// The domain, `None` when unbounded (`Domain`, valid only when bounded natively);
    /// `extrema.domain()`.
    pub fn domain(&self) -> Option<&Domain1D> {
        self.domain.as_ref()
    }
    /// The line (`Line`); `extrema.line()`.
    pub fn line(&self) -> &Line2 {
        &self.line
    }
    /// The interior extremum (`Perform`); Max mode finds none. A projection outside
    /// the domain widened by `tol` gives NoSolution, one inside is clamped to it;
    /// `extrema.perform(point, 1e-9, SearchMode::MinMax)`.
    pub fn perform(&mut self, point: Point2, tol: f64, mode: SearchMode) -> &ExtremaResult {
        self.result.clear();
        if super::is_invalid_input(tol, &point, self.domain.as_ref()) {
            self.result.status = Status::InvalidInput;
            return &self.result;
        }
        if mode == SearchMode::Max {
            self.result.status = Status::NoSolution;
            return &self.result;
        }
        let direction: Vector2 = Vector2::from(self.line.direction());
        let origin: Point2 = self.line.location();
        let mut u: f64 = Vector2::between(origin, point).dot(direction);
        if let Some(domain) = self.domain {
            if u < domain.min - tol || u > domain.max + tol {
                // Projection is outside bounds - no interior extremum
                self.result.status = Status::NoSolution;
                return &self.result;
            }
            u = std_clamp(u, domain.min, domain.max);
        }
        let on_line: Point2 = origin.translated(direction * u);
        let extremum = ExtremumResult::new(u, on_line, point.square_distance(on_line), true, false);
        self.result.extrema.push(extremum);
        self.result.status = Status::OK;
        &self.result
    }
    /// The interior extremum and the extremal ends of a bounded line
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
}

/// `std::clamp`: `v < lo ? lo : (hi < v ? hi : v)`.
pub(crate) fn std_clamp(value: f64, low: f64, high: f64) -> f64 {
    if value < low {
        low
    } else if high < value {
        high
    } else {
        value
    }
}

impl CurveEvaluator<Point2> for Line {
    type Error = Infallible;
    fn evaluate(&self, u: f64) -> Result<Point2, Infallible> {
        Ok(self.value(u))
    }
}

impl AnalyticEvaluator<Point2> for Line {
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
