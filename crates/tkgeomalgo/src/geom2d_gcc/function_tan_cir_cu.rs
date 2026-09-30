// Rust translation of Geom2dGcc_FunctionTanCirCu.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dGcc_FunctionTanCirCu`: the function of a line tangent to a circle and a
//! curve.
use super::{CurveTool, EvalFailure};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkmath::gp::{Circle2, Vector2, Xy};
use rustycad_tkmath::math::{Function, FunctionWithDerivative};

/// `Geom2dGcc_FunctionTanCirCu`: with `P1` the curve point of parameter `u`, `T`
/// its tangent and `C`, `R` the circle centre and radius,
/// `F(u) = (|CP1|^2 - (CP1 . T)^2 / |T|^2 - R^2) / W`, zero where the tangent line
/// at `P1` touches the circle. `W` (skv, 2001) is the larger of `R` and the squared
/// distance from the centre to the sum of 20 curve samples; a
/// `math_FunctionWithDerivative`.
/// ```
/// use rustycad_tkgeomalgo::geom2d_gcc::FunctionTanCirCu;
/// use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
/// use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::gp::{Axis2d, Circle2, Direction2, Point2};
/// use rustycad_tkmath::math::Function;
/// let line: Handle<dyn Geom2dCurve> =
///     Handle::new(Geom2dLine::from_point(Point2::new(0.0, 1.0), Direction2::new(1.0, 0.0)?)).cast();
/// let adaptor = Geom2dAdaptorCurve::with_range(&line, -1.0, 1.0)?;
/// let circle = Circle2::from_x_axis(Axis2d::new(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0)?), 1.0, true)?;
/// let mut f = FunctionTanCirCu::new(circle, &adaptor)?;
/// let mut value = 1.0;
/// assert!(f.value(0.5, &mut value) && value.abs() < 1e-15);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct FunctionTanCirCu<'a> {
    the_circ: Circle2,
    curve: &'a Geom2dAdaptorCurve,
    my_weight: f64,
    failure: EvalFailure,
}

impl<'a> FunctionTanCirCu<'a> {
    /// `Geom2dGcc_FunctionTanCirCu(Circ, Curv)`: the weight samples the curve,
    /// whose exceptions are returned; `FunctionTanCirCu::new(circle, &adaptor)?`.
    pub fn new(circ: Circle2, curv: &'a Geom2dAdaptorCurve) -> Result<Self, Failure> {
        let a_nb_samp: i32 = CurveTool::nb_samples(curv);
        let a_first: f64 = CurveTool::first_parameter(curv)?;
        let a_last: f64 = CurveTool::last_parameter(curv)?;
        let a_step: f64 = (a_last - a_first) / f64::from(a_nb_samp);
        let mut an_x: f64 = a_first + a_step / 2.0;
        let mut a_loc: Xy = Xy::new(0.0, 0.0);
        // A range without progress (first == last) loops forever natively too.
        while an_x <= a_last {
            a_loc += CurveTool::value(curv, an_x)?.xy();
            an_x += a_step;
        }
        let distance: f64 = (a_loc - circ.location().xy()).square_modulus();
        // std::max(a, b) keeps `a` unless a < b.
        let my_weight: f64 = if distance < circ.radius() {
            circ.radius()
        } else {
            distance
        };
        Ok(Self {
            the_circ: circ,
            curve: curv,
            my_weight,
            failure: EvalFailure::default(),
        })
    }

    /// The exception a curve evaluation raised natively, after a member answered
    /// `false`; `f.raised()?`.
    pub fn raised(&self) -> Result<(), Failure> {
        self.failure.check()
    }

    // -2 (CP1 . T / |T|^2) (CP1 . N - (CP1 . T)(T . N) / |T|^2) / W.
    fn derivative_of(&self, the_direction: Vector2, vect1: Vector2, vect2: Vector2) -> f64 {
        let norme_d1: f64 = vect1.square_magnitude();
        let cp1dott: f64 = the_direction.dot(vect1);
        let deriv: f64 = -2.0
            * (cp1dott / norme_d1)
            * (the_direction.dot(vect2) - cp1dott * vect1.dot(vect2) / norme_d1);
        deriv / self.my_weight
    }
}

impl Function for FunctionTanCirCu<'_> {
    fn value(&mut self, x: f64, f: &mut f64) -> bool {
        let Some((point, vect1)) = self.failure.keep(CurveTool::d1(self.curve, x)) else {
            return false;
        };
        let norme_d1: f64 = vect1.magnitude();
        let the_direction = Vector2::between(self.the_circ.location(), point);
        let squaredir: f64 = the_direction.dot(the_direction);
        let r: f64 = self.the_circ.radius();
        let fval: f64 = squaredir
            - r * r
            - (the_direction.dot(vect1)) * (the_direction.dot(vect1)) / (norme_d1 * norme_d1);
        *f = fval / self.my_weight;
        true
    }
}

impl FunctionWithDerivative for FunctionTanCirCu<'_> {
    fn derivative(&mut self, x: f64, deriv: &mut f64) -> bool {
        let Some((point, vect1, vect2)) = self.failure.keep(CurveTool::d2(self.curve, x)) else {
            return false;
        };
        let the_direction = Vector2::between(self.the_circ.location(), point);
        *deriv = self.derivative_of(the_direction, vect1, vect2);
        true
    }

    fn values(&mut self, x: f64, f: &mut f64, deriv: &mut f64) -> bool {
        let Some((point, vect1, vect2)) = self.failure.keep(CurveTool::d2(self.curve, x)) else {
            return false;
        };
        let norme_d1: f64 = vect1.square_magnitude();
        let the_direction = Vector2::between(self.the_circ.location(), point);
        let squaredir: f64 = the_direction.square_magnitude();
        let cp1dott: f64 = the_direction.dot(vect1);
        let r: f64 = self.the_circ.radius();
        let fval: f64 = squaredir - r * r - cp1dott * cp1dott / norme_d1;
        *f = fval / self.my_weight;
        *deriv = self.derivative_of(the_direction, vect1, vect2);
        true
    }
}
