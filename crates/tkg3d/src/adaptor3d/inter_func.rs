// Rust translation of ModelingData/TKG3d/Adaptor3d/Adaptor3d_InterFunc.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Adaptor3d_InterFunc`: `F(t) = C(t).X - FixVal` (or `.Y`), the function whose
//! roots are the parameters where a 2D curve crosses an iso line of its surface.
use crate::geom::errors::construction;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkmath::gp::{Point2, Vector2};
use rustycad_tkmath::math::{Function, FunctionWithDerivative};

/// The coordinate `fix` (1 for X, 2 for Y) of a 2D curve minus `fix_val`, with its
/// derivative (`Adaptor3d_InterFunc`).
///
/// The math traits cannot carry an exception: a failure of the curve is kept (the
/// first one), the evaluation reports false, and the caller returns the kept failure
/// ([`Adaptor3dInterFunc::take_failure`]) as the native exception would propagate;
/// `Adaptor3dInterFunc::new(&curve, 0.5, 1)?`.
pub struct Adaptor3dInterFunc {
    curve: Handle<dyn Adaptor2dCurve2d>,
    fix_val: f64,
    fix: i32,
    failure: Option<Failure>,
}

impl Adaptor3dInterFunc {
    /// The function of `curve` against `fix_val` on coordinate `fix`
    /// (`Adaptor3d_InterFunc(C, FixVal, Fix)`); Standard_ConstructionError unless
    /// `fix` is 1 or 2; `Adaptor3dInterFunc::new(&curve, 0.5, 2)?`.
    pub fn new(
        curve: &Handle<dyn Adaptor2dCurve2d>,
        fix_val: f64,
        fix: i32,
    ) -> Result<Self, Failure> {
        if fix != 1 && fix != 2 {
            return Err(construction(&format!(
                "Adaptor3d_InterFunc: Fix = {fix}; expected 1 (X) or 2 (Y)"
            )));
        }
        Ok(Self {
            curve: curve.clone(),
            fix_val,
            fix,
            failure: None,
        })
    }
    /// The first curve failure met by an evaluation, if any; `f.take_failure()`.
    pub fn take_failure(&mut self) -> Option<Failure> {
        self.failure.take()
    }
    // The fixed coordinate of a point or vector.
    fn coordinate(&self, (x, y): (f64, f64)) -> f64 {
        if self.fix == 1 { x } else { y }
    }
    fn keep(&mut self, failure: Failure) -> bool {
        self.failure.get_or_insert(failure);
        false
    }
}

impl Function for Adaptor3dInterFunc {
    /// `Value`: `C(x).X - FixVal` (or `.Y`); false after a curve failure.
    fn value(&mut self, x: f64, f: &mut f64) -> bool {
        let point: Point2 = match self.curve.d0(x) {
            Ok(point) => point,
            Err(failure) => return self.keep(failure),
        };
        *f = self.coordinate((point.x(), point.y())) - self.fix_val;
        true
    }
}

impl FunctionWithDerivative for Adaptor3dInterFunc {
    /// `Derivative`: through `Values`.
    fn derivative(&mut self, x: f64, d: &mut f64) -> bool {
        let mut f: f64 = 0.0;
        self.values(x, &mut f, d)
    }
    /// `Values`: the value and the fixed coordinate of `C'(x)`.
    fn values(&mut self, x: f64, f: &mut f64, d: &mut f64) -> bool {
        let (point, tangent): (Point2, Vector2) = match self.curve.d1(x) {
            Ok(result) => result,
            Err(failure) => return self.keep(failure),
        };
        *f = self.coordinate((point.x(), point.y())) - self.fix_val;
        *d = self.coordinate((tangent.x(), tangent.y()));
        true
    }
}
