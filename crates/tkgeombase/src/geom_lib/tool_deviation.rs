// Rust translation of ModelingData/TKGeomBase/GeomLib/GeomLib_Tool.hxx/.cxx
// (FuncSolveDeviation, ComputeDeviation) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomLib_Tool::ComputeDeviation`: the greatest deviation of a 2D curve from the
//! chord joining two of its points.
use crate::failure_capture::FailureCapture;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkmath::gp::{Direction2, Line2, Point2, Vector2, Xy};
use rustycad_tkmath::math::{MultipleVarFunction, Pso, Vector};

/// The deviation of the Newton overload and its optional outputs: the parameter of
/// the greatest deviation, its point on the curve, the vector from that point to the
/// chord (normal to it) and the chord line (`thePrmOnCurve`, `thePtOnCurve`,
/// `theVecCurvLine`, `theLine`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DeviationResult {
    /// The deviation, negative when it cannot be computed.
    pub deviation: f64,
    /// The parameter of the greatest deviation (written on success only).
    pub prm_on_curve: Option<f64>,
    /// The curve point there (written on success only).
    pub pt_on_curve: Option<Point2>,
    /// The vector from the curve point to the chord (written on success only).
    pub vec_curv_line: Option<Vector2>,
    /// The chord line (written as soon as the chord is valid).
    pub line: Option<Line2>,
}

/// `FuncSolveDeviation`: the negated square deviation of the curve from the line
/// through the points `thePf`, `thePl`.
struct FuncSolveDeviation<'c> {
    my_curve: &'c Geom2dAdaptorCurve,
    my_sq_mod: f64,
    my_is_valid: bool,
    my_p_ref: Xy,
    my_dir_ref: Xy,
    my_point_on_curve: Point2,
    my_vec_curv_line: Vector2,
    // A D0 or D2 failure of the curve unwinds natively.
    capture: FailureCapture,
}

impl<'c> FuncSolveDeviation<'c> {
    fn new(curve: &'c Geom2dAdaptorCurve, pf: Xy, pl: Xy) -> Self {
        let dir_ref: Xy = pl - pf;
        let sq_mod: f64 = dir_ref.square_modulus();
        Self {
            my_curve: curve,
            my_sq_mod: sq_mod,
            my_is_valid: sq_mod > precision::square_p_confusion(),
            my_p_ref: pf,
            my_dir_ref: dir_ref,
            my_point_on_curve: Point2::default(),
            my_vec_curv_line: Vector2::default(),
            capture: FailureCapture::default(),
        }
    }

    // The point and the vector to the line at `param`.
    fn update_fields(&mut self, param: f64) -> Result<(), Failure> {
        self.my_point_on_curve = self.my_curve.d0(param)?;
        let vt: Xy = self.my_point_on_curve.xy() - self.my_p_ref;
        let xy: Xy = vt.dot(self.my_dir_ref) * self.my_dir_ref / self.my_sq_mod - vt;
        self.my_vec_curv_line = Vector2::new(xy.x(), xy.y());
        Ok(())
    }

    // The square deviation (cross / |dir|)^2 and its first and second derivatives
    // (of the square cross product).
    fn value_and_derives(&mut self, param: f64) -> Result<(f64, f64, f64), Failure> {
        let (point, d1, d2): (Point2, Vector2, Vector2) = self.my_curve.d2(param)?;
        self.my_point_on_curve = point;
        let vt: Xy = point.xy() - self.my_p_ref;
        let mut val: f64 = vt.crossed(self.my_dir_ref);
        let mut deriv1: f64 = d1.xy().crossed(self.my_dir_ref);
        let deriv2: f64 = 2.0 * (deriv1 * deriv1 + val * d2.xy().crossed(self.my_dir_ref));
        deriv1 *= 2.0 * val;
        val *= val / self.my_sq_mod;
        Ok((val, deriv1, deriv2))
    }

    fn line(&self) -> Result<Line2, Failure> {
        let mut line = Line2::default();
        line.set_direction(Direction2::new(self.my_dir_ref.x(), self.my_dir_ref.y())?);
        line.set_location(Point2::new(self.my_p_ref.x(), self.my_p_ref.y()));
        Ok(line)
    }
}

impl MultipleVarFunction for FuncSolveDeviation<'_> {
    fn variables(&self) -> i32 {
        1
    }

    // The negated square deviation, for the minimization.
    fn value(&mut self, prm: &Vector<'_>, value: &mut f64) -> bool {
        let Some(x) = prm.value(prm.lower()).ok().copied() else {
            return false;
        };
        let derived = self.value_and_derives(x);
        let Some((val, _, _)) = self.capture.keep(derived) else {
            return false;
        };
        *value = -val;
        true
    }
}

/// `GeomLib_Tool::ComputeDeviation(theCurve, theFPar, theLPar, theStartParameter,
/// theNbIters = 100, thePrmOnCurve, thePtOnCurve, theVecCurvLine, theLine)`: the
/// greatest deviation of `curve` from its chord `[FPar, LPar]`, by at most
/// `nb_iters` Newton steps on the square deviation from `start_parameter`; -1 when
/// the start is outside the range or the chord is degenerate;
/// `geom_lib::compute_deviation(&curve, 0.0, 1.0, 0.5, 100)?`.
pub fn compute_deviation(
    curve: &Geom2dAdaptorCurve,
    f_par: f64,
    l_par: f64,
    start_parameter: f64,
    nb_iters: i32,
) -> Result<DeviationResult, Failure> {
    let mut result = DeviationResult {
        deviation: -1.0,
        prm_on_curve: None,
        pt_on_curve: None,
        vec_curv_line: None,
        line: None,
    };
    // Computed maximal deflection.
    if start_parameter < f_par || start_parameter > l_par {
        return Ok(result);
    }
    let pf: Point2 = curve.value(f_par)?;
    let pl: Point2 = curve.value(l_par)?;
    let mut func = FuncSolveDeviation::new(curve, pf.xy(), pl.xy());
    if !func.my_is_valid {
        return Ok(result);
    }
    result.line = Some(func.line()?);
    let (u_max, sq_defl) = newton_maximum(&mut func, [f_par, l_par], start_parameter, nb_iters)?;
    if sq_defl < 0.0 {
        result.deviation = sq_defl;
        return Ok(result);
    }
    result.prm_on_curve = Some(u_max);
    func.update_fields(u_max)?;
    result.pt_on_curve = Some(func.my_point_on_curve);
    result.vec_curv_line = Some(func.my_vec_curv_line);
    result.deviation = sq_defl.sqrt();
    Ok(result)
}

// The Newton iterations on the derivative of the square deviation: the greatest
// square deviation met and its parameter.
fn newton_maximum(
    func: &mut FuncSolveDeviation<'_>,
    [f_par, l_par]: [f64; 2],
    start_parameter: f64,
    nb_iters: i32,
) -> Result<(f64, f64), Failure> {
    let tol_defl: f64 = precision::p_confusion();
    let mut u0: f64 = start_parameter;
    let mut u_max: f64 = start_parameter;
    let (mut sq_defl, mut d1, mut d2) = func.value_and_derives(u0)?;
    for _ in 1..=nb_iters {
        if d2.abs() < precision::p_confusion() {
            break;
        }
        let delta: f64 = d1 / d2;
        let u1: f64 = u0 - delta;
        if u1 < f_par || u1 > l_par {
            break;
        }
        let sq_d: f64;
        (sq_d, d1, d2) = func.value_and_derives(u1)?;
        if sq_d > sq_defl {
            u_max = u1;
            let dd: f64 = if sq_defl > 0.0 {
                (sq_d.sqrt() - sq_defl.sqrt()).abs()
            } else {
                sq_d.sqrt()
            };
            sq_defl = sq_d;
            if dd < tol_defl {
                break;
            }
        }
        if (u0 - u1).abs() < precision::p_confusion() {
            break;
        }
        u0 = u1;
    }
    Ok((u_max, sq_defl))
}

/// `GeomLib_Tool::ComputeDeviation(theCurve, theFPar, theLPar, theNbSubIntervals,
/// theNbIters = 10, thePrmOnCurve)`: the greatest deviation found by math_PSO with
/// `nb_sub_intervals` particles and `nb_iters` iterations (a rough start for the
/// Newton overload), with its parameter; -1 when the chord is degenerate or the
/// search finds nothing; `geom_lib::compute_deviation_pso(&curve, 0.0, 1.0, 10, 10)?`.
pub fn compute_deviation_pso(
    curve: &Geom2dAdaptorCurve,
    f_par: f64,
    l_par: f64,
    nb_sub_intervals: i32,
    nb_iters: i32,
) -> Result<(f64, Option<f64>), Failure> {
    let pf: Point2 = curve.value(f_par)?;
    let pl: Point2 = curve.value(l_par)?;
    let mut func = FuncSolveDeviation::new(curve, pf.xy(), pl.xy());
    if !func.my_is_valid {
        return Ok((-1.0, None));
    }
    let lower: Vector<'static> = Vector::filled(1, 1, f_par)?;
    let upper: Vector<'static> = Vector::filled(1, 1, l_par)?;
    let step: Vector<'static> =
        Vector::filled(1, 1, (l_par - f_par) / (10.0 * f64::from(nb_sub_intervals)))?;
    let mut output_pnt: Vector<'static> = Vector::filled(1, 1, f_par)?;
    let mut sq_defl: f64 = f64::MAX;
    {
        let mut mpso =
            Pso::with_options(&mut func, &lower, &upper, &step, nb_sub_intervals, nb_iters)?;
        mpso.perform_with_iterations(&step, &mut sq_defl, &mut output_pnt, nb_iters)?;
    }
    func.capture.take()?;
    if sq_defl == f64::MAX {
        return Ok((-1.0, None));
    }
    Ok((sq_defl.abs().sqrt(), Some(*output_pnt.value(1)?)))
}
