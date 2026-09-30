// Rust translation of ModelingAlgorithms/TKTopAlgo/Bisector/Bisector_FunctionInter.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Bisector_FunctionInter`: the difference of the distances from a curve point to
//! two bisectors that share the curve's parameter.
use super::BisectorCurve;
use super::eval_failure::EvalFailure;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, errors};
use rustycad_tkmath::gp::{self, Point2, Vector2};
use rustycad_tkmath::math::{Function, FunctionWithDerivative};

/// `Bisector_FunctionInter`: `F(x) = |C(x) B1(x)| - |C(x) B2(x)|`, zero where the
/// two bisectors meet; a `math_FunctionWithDerivative`. A member answers `false`
/// when an evaluation fails (or a handle is null, a native crash), and
/// [`BisectorFunctionInter::raised`] gives the failure.
/// ```
/// use rustycad_tktopalgo::bisector::BisectorFunctionInter;
/// use rustycad_tkmath::math::Function;
/// let mut f = BisectorFunctionInter::new();
/// let mut value = 0.0;
/// assert!(!f.value(0.0, &mut value) && f.raised().is_err());
/// ```
#[derive(Default)]
pub struct BisectorFunctionInter {
    curve: Handle<dyn Geom2dCurve>,
    bisector1: Handle<dyn BisectorCurve>,
    bisector2: Handle<dyn BisectorCurve>,
    failure: EvalFailure,
}

type Samples = (Point2, Point2, Point2);
type SamplesD1 = ((Point2, Vector2), (Point2, Vector2), (Point2, Vector2));

impl BisectorFunctionInter {
    /// `Bisector_FunctionInter()`: null curves; `BisectorFunctionInter::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Bisector_FunctionInter(C, Bis1, Bis2)`;
    /// `BisectorFunctionInter::with_curves(&guide, &bis1, &bis2)`.
    pub fn with_curves(
        c: &Handle<dyn Geom2dCurve>,
        bis1: &Handle<dyn BisectorCurve>,
        bis2: &Handle<dyn BisectorCurve>,
    ) -> Self {
        let mut function: Self = Self::new();
        function.perform(c, bis1, bis2);
        function
    }
    /// `Perform(C, Bis1, Bis2)`: replaces the three curves; `f.perform(&c, &b1, &b2)`.
    pub fn perform(
        &mut self,
        c: &Handle<dyn Geom2dCurve>,
        bis1: &Handle<dyn BisectorCurve>,
        bis2: &Handle<dyn BisectorCurve>,
    ) {
        self.curve = c.clone();
        self.bisector1 = bis1.clone();
        self.bisector2 = bis2.clone();
    }
    /// The exception an evaluation raised, after a member answered `false`;
    /// `f.raised()?`.
    pub fn raised(&self) -> Result<(), Failure> {
        self.failure.check()
    }
    // The three points at `x`.
    fn points(&self, x: f64) -> Result<Samples, Failure> {
        let (c, b1, b2) = self.curves()?;
        Ok((c.value(x)?, b1.value(x)?, b2.value(x)?))
    }
    // The three points and tangents at `x`.
    fn points_d1(&self, x: f64) -> Result<SamplesD1, Failure> {
        let (c, b1, b2) = self.curves()?;
        Ok((c.d1(x)?, b1.d1(x)?, b2.d1(x)?))
    }
    // The three curves; a null one is a native null dereference.
    fn curves(
        &self,
    ) -> Result<(&dyn Geom2dCurve, &dyn BisectorCurve, &dyn BisectorCurve), Failure> {
        let null =
            || errors::null_object("Bisector_FunctionInter: a curve is null; expected Perform");
        let c = self.curve.get().ok_or_else(null)?;
        let b1 = self.bisector1.get().ok_or_else(null)?;
        let b2 = self.bisector2.get().ok_or_else(null)?;
        Ok((c, b1, b2))
    }
}

// `DF` of one bisector: the derivative of the distance, infinite at a zero distance.
fn distance_derivative(pc: Point2, tc: Vector2, pb: Point2, tb: Vector2, f: f64) -> f64 {
    if f.abs() < gp::resolution() {
        return precision::infinite();
    }
    ((pc.x() - pb.x()) * (tc.x() - tb.x()) + (pc.y() - pb.y()) * (tc.y() - tb.y())) / f
}

impl Function for BisectorFunctionInter {
    fn value(&mut self, x: f64, f: &mut f64) -> bool {
        let Some((pc, pb1, pb2)) = self.failure.keep(self.points(x)) else {
            return false;
        };
        *f = pc.distance(pb1) - pc.distance(pb2);
        true
    }
}

impl FunctionWithDerivative for BisectorFunctionInter {
    fn derivative(&mut self, x: f64, d: &mut f64) -> bool {
        let mut f: f64 = 0.0;
        self.values(x, &mut f, d)
    }
    fn values(&mut self, x: f64, f: &mut f64, d: &mut f64) -> bool {
        let Some(((pc, tc), (pb1, tb1), (pb2, tb2))) = self.failure.keep(self.points_d1(x)) else {
            return false;
        };
        let f1: f64 = pc.distance(pb1);
        let f2: f64 = pc.distance(pb2);
        *f = f1 - f2;
        let df1: f64 = distance_derivative(pc, tc, pb1, tb1, f1);
        let df2: f64 = distance_derivative(pc, tc, pb2, tb2, f2);
        *d = df1 - df2;
        true
    }
}
