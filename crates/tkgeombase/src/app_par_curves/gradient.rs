// Rust translation of ModelingData/TKGeomBase/AppParCurves/AppParCurves_Gradient.gxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AppParCurves_Gradient` (AppParCurves_Gradient.gxx): the parameters of a
//! constrained Bezier fit improved by one projection step then BFGS.
use super::Constraints;
use super::arrays::scratch;
use super::function::Function;
use super::gradient_bfgs::GradientBfgs;
use super::line_tool::LineTool;
use super::multi_curve::{MultiCurve, MultiCurveEval};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::real_epsilon;
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};
use rustycad_tkmath::math::Vector;

/// The largest parameter change of the projection step.
const MAX_STEP: f64 = 5.0e-02;
/// The BFGS tolerance of the native gradients.
const BFGS_EPS: f64 = 1.0e-07;

/// `AppParCurves_Gradient`: generic over the MultiLine (the native NbIterations
/// default is 200). Error() reads the constrained branch of the function, so it is 0
/// for a line without inner constraints, as natively.
///
/// ```
/// use rustycad_tkgeombase::app_def::MultiLine;
/// use rustycad_tkgeombase::app_par_curves::{Gradient, MultiCurveEval};
/// # use rustycad_tkgeombase::app_par_curves::{Constraint, ConstraintCouple};
/// # use rustycad_tkernel::ncollection::{Array1, HArray1};
/// # use rustycad_tkernel::standard::Handle;
/// # use rustycad_tkmath::gp::Point3;
/// # use rustycad_tkmath::math::Vector;
/// // Six points of the parabola y = x^2, AppDef_MultiLine read by AppDef_MyLineTool.
/// let points: Vec<Point3> = (0..6).map(|i| {
///     let t = f64::from(i) / 5.0;
///     Point3::new(t, t * t, 0.0)
/// }).collect();
/// let line = MultiLine::from_points(&Array1::from_shared(&points, 1));
/// # let ends = [ConstraintCouple::new(1, Constraint::PassPoint), ConstraintCouple::new(6, Constraint::PassPoint)];
/// # let constraints = Handle::new(HArray1::from_array(&Array1::from_shared(&ends, 1)));
/// # let mut u = Vector::with_bounds(1, 6)?;
/// # (1..=6).for_each(|i| u[i] = f64::from(i - 1) / 5.0);
/// let gradient = Gradient::new(&line, 1, 6, &constraints, &mut u, 3, 1e-3, 1e-6, 200)?;
/// assert!(gradient.is_done());
/// assert_eq!(gradient.value().nb_poles(), 4);
/// assert!(gradient.max_error_3d() < 1e-3);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct Gradient {
    scu: MultiCurve,
    par_error: Vector<'static>,
    av_error: f64,
    m_error3d: f64,
    m_error2d: f64,
    done: bool,
}

/// The canonical coefficients of every 3D and 2D curve of `curve`, curve after curve.
pub(crate) struct CurveCoefficients {
    pub(crate) coef3d: Vec<Array1<'static, Point3>>,
    pub(crate) coef2d: Vec<Array1<'static, Point2>>,
}

fn coefficients(
    curve: &MultiCurve,
    [nb_p3d, nb_p2d]: [i32; 2],
) -> Result<CurveCoefficients, Failure> {
    let npoles: i32 = curve.nb_poles();
    let mut result = CurveCoefficients {
        coef3d: Vec::new(),
        coef2d: Vec::new(),
    };
    for k in 1..=nb_p3d {
        let mut poles: Array1<'static, Point3> = scratch(npoles)?;
        curve.curve_3d(k, &mut poles)?;
        let mut coef: Array1<'static, Point3> = scratch(npoles)?;
        bsplclib::poles_coefficients(&poles, None, &mut coef, None)?;
        result.coef3d.push(coef);
    }
    for k in 1..=nb_p2d {
        let mut poles: Array1<'static, Point2> = scratch(npoles)?;
        curve.curve_2d(nb_p3d + k, &mut poles)?;
        let mut coef: Array1<'static, Point2> = scratch(npoles)?;
        bsplclib::poles_coefficients(&poles, None, &mut coef, None)?;
        result.coef2d.push(coef);
    }
    Ok(result)
}

/// `(FU, DFU)`: the projection numerator `sum (P - C) . C'` and denominator
/// `sum |C'|^2` of the line points at `u` on the Bezier coefficients.
fn projection_terms(
    ssp: &impl LineTool,
    j: i32,
    u: f64,
    coef: &CurveCoefficients,
) -> Result<(f64, f64), Failure> {
    let (mut points, mut points2d) = (
        scratch::<Point3>(ssp.nb_p3d()?)?,
        scratch::<Point2>(ssp.nb_p2d()?)?,
    );
    // The native test reads `nbP != 0 && nbP2d != 0` with nbP the total count: any
    // 2D point selects the 3D and 2D overload.
    if ssp.nb_p2d()? != 0 {
        ssp.value(j, &mut points, &mut points2d)?;
    } else {
        ssp.value_3d(j, &mut points)?;
    }
    let (mut fu, mut dfu): (f64, f64) = (0., 0.);
    for (k, c) in (1..).zip(&coef.coef3d) {
        let (pt, v1): (Point3, Vector3) = bsplclib::coefs_d1(u, c, None)?;
        fu += Vector3::between(pt, points[k]).dot(v1);
        dfu += v1.square_magnitude();
    }
    for (k, c) in (1..).zip(&coef.coef2d) {
        let (pt, v1): (Point2, Vector2) = bsplclib::coefs_d1(u, c, None)?;
        fu += Vector2::between(pt, points2d[k]).dot(v1);
        dfu += v1.square_magnitude();
    }
    Ok((fu, dfu))
}

/// The projection step (Rogers & Fog 89): `u += sign(FU/DFU) min(0.05, |FU/DFU|)`
/// when `DFU >= RealEpsilon()`.
pub(crate) fn projected(u: f64, (fu, dfu): (f64, f64)) -> f64 {
    if dfu < real_epsilon() {
        return u;
    }
    let du: f64 = fu / dfu;
    u + MAX_STEP.min(du.abs()).copysign(du)
}

impl Gradient {
    /// The native constructor: fits, projects the inner parameters once, fits again
    /// and, when the errors exceed the tolerances and `nb_iterations != 0`, runs BFGS;
    /// `parameters` receives the final parameters.
    #[allow(clippy::too_many_arguments)] // Mirrors the native nine-argument constructor.
    pub fn new<L: LineTool + Clone>(
        ssp: &L,
        first_point: i32,
        last_point: i32,
        constraints: &Constraints,
        parameters: &mut Vector<'_>,
        deg: i32,
        tol3d: f64,
        tol2d: f64,
        nb_iterations: i32,
    ) -> Result<Self, Failure> {
        let mut gradient: Self = Self {
            scu: MultiCurve::new(),
            par_error: Vector::filled(first_point, last_point, 0.)?,
            av_error: 0.,
            m_error3d: 0.,
            m_error2d: 0.,
            done: false,
        };
        let mut f: Function<L> =
            Function::new(ssp, first_point, last_point, constraints, parameters, deg)?;
        let mut fval: f64 = 0.;
        if !f.evaluate(parameters, &mut fval)? {
            return Ok(gradient);
        }
        gradient.scu = f.curve_value()?.clone();
        let coef: CurveCoefficients = coefficients(&gradient.scu, [ssp.nb_p3d()?, ssp.nb_p2d()?])?;
        for j in first_point + 1..=last_point - 1 {
            let u: f64 = *parameters.value(j)?;
            *parameters.change_value(j)? = projected(u, projection_terms(ssp, j, u, &coef)?);
        }
        if !f.evaluate(parameters, &mut fval)? {
            gradient.scu = MultiCurve::new();
            return Ok(gradient);
        }
        gradient.optimize(&mut f, parameters, [tol3d, tol2d], nb_iterations)?;
        gradient.errors(
            &f,
            [first_point, last_point],
            ssp.nb_p3d()? + ssp.nb_p2d()?,
            [tol3d, tol2d],
        )?;
        Ok(gradient)
    }

    fn optimize<L: LineTool + Clone>(
        &mut self,
        f: &mut Function<L>,
        parameters: &mut Vector<'_>,
        [tol3d, tol2d]: [f64; 2],
        nb_iterations: i32,
    ) -> Result<(), Failure> {
        (self.m_error3d, self.m_error2d) = (f.max_error_3d(), f.max_error_2d());
        if self.m_error3d <= tol3d && self.m_error2d <= tol2d {
            self.done = true;
            self.scu = f.curve_value()?.clone();
            return Ok(());
        }
        if nb_iterations == 0 {
            return Ok(());
        }
        GradientBfgs::new(f, parameters, tol3d, tol2d, BFGS_EPS, nb_iterations)?;
        if let Some(failure) = f.take_failure() {
            return Err(failure);
        }
        parameters.initialized(f.new_parameters())?;
        self.scu = f.curve_value()?.clone();
        Ok(())
    }

    // ParError (the largest curve error at each point), AvError and the final errors.
    fn errors<L: LineTool + Clone>(
        &mut self,
        f: &Function<L>,
        [first_point, last_point]: [i32; 2],
        nb_p: i32,
        [tol3d, tol2d]: [f64; 2],
    ) -> Result<(), Failure> {
        self.av_error = 0.;
        for j in first_point..=last_point {
            for k in 1..=nb_p {
                let error: f64 = f.error(j, k)?;
                // std::max(a, b): b only when a < b.
                if self.par_error[j] < error {
                    self.par_error[j] = error;
                }
            }
            self.av_error += self.par_error[j];
        }
        self.av_error /= f64::from(last_point - first_point + 1);
        (self.m_error3d, self.m_error2d) = (f.max_error_3d(), f.max_error_2d());
        if self.m_error3d <= tol3d && self.m_error2d <= tol2d {
            self.done = true;
        }
        Ok(())
    }

    /// `Value()`: the curves; `gradient.value()`.
    pub fn value(&self) -> MultiCurve {
        self.scu.clone()
    }

    /// `IsDone()`: the tolerances are reached; `gradient.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }

    /// `Error(Index)`: the largest error at point `index`; `gradient.error(3)?`.
    pub fn error(&self, index: i32) -> Result<f64, Failure> {
        self.par_error.value(index).copied()
    }

    /// `AverageError()`; `gradient.average_error()`.
    pub fn average_error(&self) -> f64 {
        self.av_error
    }

    /// `MaxError3d()`; `gradient.max_error_3d()`.
    pub fn max_error_3d(&self) -> f64 {
        self.m_error3d
    }

    /// `MaxError2d()`; `gradient.max_error_2d()`.
    pub fn max_error_2d(&self) -> f64 {
        self.m_error2d
    }
}
