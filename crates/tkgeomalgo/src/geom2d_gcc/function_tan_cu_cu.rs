// Rust translation of Geom2dGcc_FunctionTanCuCu.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dGcc_FunctionTanCuCu`: the system of a line tangent to two curves.
use super::{CurveTool, EvalFailure, Type3, null_object};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle2, Point2, Vector2};
use rustycad_tkmath::math::{FunctionSet, FunctionSetWithDerivatives, Matrix, Vector};

/// The points and derivatives of both arguments (`InitDerivative` outputs):
/// `(Point1, Point2, Tan1, Tan2, D21, D22)`.
pub type TanCuCuDerivatives = (Point2, Point2, Vector2, Vector2, Vector2, Vector2);

/// `Geom2dGcc_FunctionTanCuCu`: with `P1`, `P2` the points of parameters `u1`, `u2`
/// and `T1`, `T2` the tangents, `F1 = P1P2 ^ T1 / (|T1| |P1P2|^2)` and
/// `F2 = T1 ^ T2 / (|T1| |T2|)`, both zero on a line tangent to both arguments;
/// a `math_FunctionSetWithDerivatives` of 2 variables and 2 equations.
/// ```
/// use rustycad_tkgeomalgo::geom2d_gcc::FunctionTanCuCu;
/// use rustycad_tkg2d::geom2d::{Geom2dCircle, Geom2dCurve};
/// use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::gp::{Axis2d, Direction2, Point2};
/// use rustycad_tkmath::math::{FunctionSet, Vector};
/// let axis = |x: f64| Axis2d::new(Point2::new(x, 0.0), Direction2::new(1.0, 0.0).unwrap());
/// let c1: Handle<dyn Geom2dCurve> = Handle::new(Geom2dCircle::from_axis(axis(0.0), 1.0, true)?).cast();
/// let c2: Handle<dyn Geom2dCurve> = Handle::new(Geom2dCircle::from_axis(axis(4.0), 1.0, true)?).cast();
/// let (a1, a2) = (Geom2dAdaptorCurve::with_curve(&c1)?, Geom2dAdaptorCurve::with_curve(&c2)?);
/// let mut f = FunctionTanCuCu::new(&a1, &a2);
/// let x = Vector::filled(1, 2, std::f64::consts::FRAC_PI_2)?;
/// let mut values = Vector::with_bounds(1, 2)?;
/// assert!(f.value(&x, &mut values) && values[2].abs() < 1e-15);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct FunctionTanCuCu<'a> {
    the_curve1: Option<&'a Geom2dAdaptorCurve>,
    the_curve2: &'a Geom2dAdaptorCurve,
    the_circ1: Circle2,
    the_type: Type3,
    failure: EvalFailure,
}

impl<'a> FunctionTanCuCu<'a> {
    /// `Geom2dGcc_FunctionTanCuCu(Curv1, Curv2)`: type CuCu;
    /// `FunctionTanCuCu::new(&adaptor1, &adaptor2)`.
    pub fn new(curv1: &'a Geom2dAdaptorCurve, curv2: &'a Geom2dAdaptorCurve) -> Self {
        Self {
            the_curve1: Some(curv1),
            the_curve2: curv2,
            the_circ1: Circle2::default(),
            the_type: Type3::CuCu,
            failure: EvalFailure::default(),
        }
    }

    /// `Geom2dGcc_FunctionTanCuCu(Circ1, Curv2)`: type CiCu;
    /// `FunctionTanCuCu::with_circle(circle, &adaptor)`.
    pub fn with_circle(circ1: Circle2, curv2: &'a Geom2dAdaptorCurve) -> Self {
        Self {
            the_curve1: None,
            the_curve2: curv2,
            the_circ1: circ1,
            the_type: Type3::CiCu,
            failure: EvalFailure::default(),
        }
    }

    /// The exception a curve evaluation raised natively, after a member answered
    /// `false`; `f.raised()?`.
    pub fn raised(&self) -> Result<(), Failure> {
        self.failure.check()
    }

    /// `InitDerivative(X, Point1, Point2, Tan1, Tan2, D21, D22)`: the points and
    /// first two derivatives of both arguments at `X(1)`, `X(2)`;
    /// `let (p1, p2, t1, t2, d21, d22) = f.init_derivative(&x)?;`.
    pub fn init_derivative(&self, x: &Vector<'_>) -> Result<TanCuCuDerivatives, Failure> {
        let (point1, tan1, d21) = match self.the_type {
            Type3::CuCu => {
                let curve1 = self
                    .the_curve1
                    .ok_or_else(|| null_object("Geom2dGcc_FunctionTanCuCu::InitDerivative"))?;
                CurveTool::d2(curve1, *x.value(1)?)?
            }
            Type3::CiCu => elclib::d2(*x.value(1)?, &self.the_circ1),
        };
        let (point2, tan2, d22) = CurveTool::d2(self.the_curve2, *x.value(2)?)?;
        Ok((point1, point2, tan1, tan2, d21, d22))
    }

    // Fval(1), Fval(2).
    fn write_values(&self, x: &Vector<'_>, fval: &mut Vector<'_>) -> Result<(), Failure> {
        let (point1, point2, vect11, vect21, _, _) = self.init_derivative(x)?;
        let norme_d11: f64 = vect11.magnitude();
        let norme_d21: f64 = vect21.magnitude();
        let the_direction = Vector2::between(point1, point2);
        let squaredir: f64 = the_direction.dot(the_direction);
        *fval.change_value(1)? = the_direction.crossed(vect11) / (norme_d11 * squaredir);
        *fval.change_value(2)? = vect11.crossed(vect21) / (norme_d11 * norme_d21);
        Ok(())
    }

    // Deriv(1..2, 1..2).
    fn write_derivatives(&self, x: &Vector<'_>, deriv: &mut Matrix<'_>) -> Result<(), Failure> {
        let (point1, point2, v11, v21, v12, v22) = self.init_derivative(x)?;
        let n11: f64 = v11.magnitude();
        let n21: f64 = v21.magnitude();
        let td = Vector2::between(point1, point2);
        let sq: f64 = td.dot(td);
        *deriv.change_value(1, 1)? = td.crossed(v12) / (n11 * sq)
            + (td.crossed(v11) * n11 * n11 * v11.dot(td)) / (n11 * n11 * n11 * sq * sq * sq);
        *deriv.change_value(1, 2)? = v21.crossed(v11) / (n11 * sq)
            - (td.crossed(v11) * n11 * n11 * v21.dot(td)) / (n11 * n11 * n11 * sq * sq * sq);
        *deriv.change_value(2, 1)? = (v12.crossed(v21)) / (n11 * n21)
            - (v11.crossed(v21)) * (v12.dot(v11)) * n21 * n21 / (n11 * n11 * n11 * n21 * n21 * n21);
        *deriv.change_value(2, 2)? = (v11.crossed(v22)) / (n11 * n21)
            - (v11.crossed(v21)) * (v22.dot(v21)) * n11 * n11 / (n11 * n11 * n11 * n21 * n21 * n21);
        Ok(())
    }
}

impl FunctionSet for FunctionTanCuCu<'_> {
    fn variables(&self) -> i32 {
        2
    }
    fn equations(&self) -> i32 {
        2
    }
    fn value(&mut self, x: &Vector<'_>, f: &mut Vector<'_>) -> bool {
        let result = self.write_values(x, f);
        self.failure.keep(result).is_some()
    }
}

impl FunctionSetWithDerivatives for FunctionTanCuCu<'_> {
    fn derivatives(&mut self, x: &Vector<'_>, deriv: &mut Matrix<'_>) -> bool {
        let result = self.write_derivatives(x, deriv);
        self.failure.keep(result).is_some()
    }
    fn values(&mut self, x: &Vector<'_>, f: &mut Vector<'_>, deriv: &mut Matrix<'_>) -> bool {
        let result = self
            .write_values(x, f)
            .and_then(|()| self.write_derivatives(x, deriv));
        self.failure.keep(result).is_some()
    }
}
