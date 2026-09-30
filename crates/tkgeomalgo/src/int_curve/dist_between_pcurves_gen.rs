// Rust translation of IntCurve_DistBetweenPCurvesGen.gxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntCurve_DistBetweenPCurvesGen`: the vector from a point of the second curve
//! to a point of the first one, as a function of both parameters.
use super::PCurveTool;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point2, Vector2};
use rustycad_tkmath::math::{FunctionSet, FunctionSetWithDerivatives, Matrix, Vector};

/// `IntCurve_DistBetweenPCurvesGen` over a curve tool
/// (`Geom2dInt_TheDistBetweenPCurvesOfTheIntPCurvePCurveOfGInter`): the two
/// equations `C1(X1) - C2(X2) = 0` with their Jacobian. The math traits carry no
/// exception, so a curve failure is kept ([`DistBetweenPCurvesGen::take_failure`])
/// and reported as `false`.
///
/// ```
/// use rustycad_tkgeomalgo::geom2d_int::TheDistBetweenPCurvesOfTheIntPCurvePCurveOfGInter;
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
/// use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
/// use rustycad_tkmath::gp::{Direction2, Point2};
/// use rustycad_tkmath::math::{FunctionSet, Vector};
/// let line: Handle<dyn Geom2dCurve> =
///     Handle::new(Geom2dLine::from_point(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0).unwrap())).cast();
/// let a = Geom2dAdaptorCurve::with_curve(&line).unwrap();
/// let mut f = TheDistBetweenPCurvesOfTheIntPCurvePCurveOfGInter::new(&a, &a);
/// let x = Vector::from_xy(rustycad_tkmath::gp::Xy::new(2.0, 0.5));
/// let mut r = Vector::with_bounds(1, 2).unwrap();
/// assert!(f.value(&x, &mut r));
/// assert_eq!(r[1], 1.5);
/// ```
pub struct DistBetweenPCurvesGen<'a, T: PCurveTool> {
    thecurve1: &'a T::Curve,
    thecurve2: &'a T::Curve,
    failure: Option<Failure>,
}

impl<'a, T: PCurveTool> DistBetweenPCurvesGen<'a, T> {
    /// `IntCurve_DistBetweenPCurvesGen(C1, C2)`.
    /// `TheDistBetweenPCurvesOfTheIntPCurvePCurveOfGInter::new(&c1, &c2)`.
    pub fn new(c1: &'a T::Curve, c2: &'a T::Curve) -> Self {
        Self {
            thecurve1: c1,
            thecurve2: c2,
            failure: None,
        }
    }
    /// `NbVariables()`: 2. `function.nb_variables()`.
    pub fn nb_variables(&self) -> i32 {
        2
    }
    /// `NbEquations()`: 2. `function.nb_equations()`.
    pub fn nb_equations(&self) -> i32 {
        2
    }
    /// The curve failure raised by the last evaluation, if any.
    /// `function.take_failure()`.
    pub fn take_failure(&mut self) -> Option<Failure> {
        self.failure.take()
    }

    fn residual(&self, x: &Vector<'_>, f: &mut Vector<'_>) -> Result<(), Failure> {
        let p1: Point2 = T::d0(self.thecurve1, x[1])?;
        let p2: Point2 = T::d0(self.thecurve2, x[2])?;
        f[1] = p1.x() - p2.x();
        f[2] = p1.y() - p2.y();
        Ok(())
    }
    // The Jacobian; returns the points of both curves.
    fn jacobian(&self, x: &Vector<'_>, d: &mut Matrix<'_>) -> Result<(Point2, Point2), Failure> {
        let (mut p1, mut p2, mut t) = (Point2::default(), Point2::default(), Vector2::default());
        T::d1(self.thecurve1, x[1], &mut p1, &mut t)?;
        d[(1, 1)] = t.x();
        d[(2, 1)] = t.y();
        T::d1(self.thecurve2, x[2], &mut p2, &mut t)?;
        d[(1, 2)] = -(t.x());
        d[(2, 2)] = -(t.y());
        Ok((p1, p2))
    }
    fn keep(&mut self, result: Result<(), Failure>) -> bool {
        match result {
            Ok(()) => true,
            Err(failure) => {
                self.failure = Some(failure);
                false
            }
        }
    }
}

impl<T: PCurveTool> FunctionSet for DistBetweenPCurvesGen<'_, T> {
    fn variables(&self) -> i32 {
        2
    }
    fn equations(&self) -> i32 {
        2
    }
    /// `Value(X, F)`: `F = C1(X1) - C2(X2)`.
    fn value(&mut self, x: &Vector<'_>, f: &mut Vector<'_>) -> bool {
        let result: Result<(), Failure> = self.residual(x, f);
        self.keep(result)
    }
}

impl<T: PCurveTool> FunctionSetWithDerivatives for DistBetweenPCurvesGen<'_, T> {
    /// `Derivatives(X, D)`: the tangents of C1 and of C2 negated, as columns.
    fn derivatives(&mut self, x: &Vector<'_>, d: &mut Matrix<'_>) -> bool {
        let result: Result<(), Failure> = self.jacobian(x, d).map(|_| ());
        self.keep(result)
    }
    /// `Values(X, F, D)`: both from one D1 per curve.
    fn values(&mut self, x: &Vector<'_>, f: &mut Vector<'_>, d: &mut Matrix<'_>) -> bool {
        let result: Result<(), Failure> = self.jacobian(x, d).map(|(p1, p2)| {
            f[1] = p1.x() - p2.x();
            f[2] = p1.y() - p2.y();
        });
        self.keep(result)
    }
}
