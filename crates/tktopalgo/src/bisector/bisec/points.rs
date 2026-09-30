// Rust translation of ModelingAlgorithms/TKTopAlgo/Bisector/Bisector_Bisec.cxx (the Perform overloads with points and IsMaxRC) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The bisector of a curve and a point, and of two points.
use super::curves::opposite_sum;
use super::{Bisec, half_line};
use crate::bisector::{BisecAna, BisecPC, BisectorCurve, is_convex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkg2d::geom2d::{
    Geom2dCircle, Geom2dCurve, Geom2dLine, Geom2dPoint, Geom2dTrimmedCurve,
};
use rustycad_tkmath::gp::{self, Direction2, Point2, Vector2};

/// Which end of the curve the point is at: the end of the curve (curve-point) or
/// its start (point-curve).
#[derive(Clone, Copy, PartialEq)]
enum PointSide {
    AtCurveEnd,
    AtCurveStart,
}

/// The arguments of the mixed Perform overloads.
struct MixedInput<'a> {
    curve: &'a Handle<dyn Geom2dCurve>,
    point: Point2,
    apoint: Point2,
    v1: Vector2,
    v2: Vector2,
    adirection: f64,
    side: PointSide,
}

// True for a circle or a line (or a trimmed one): the analytic bisector applies.
fn is_analytic(curve: &Handle<dyn Geom2dCurve>) -> bool {
    let basis: Handle<dyn Geom2dCurve> = match curve.cast::<Geom2dTrimmedCurve>().get() {
        Some(trimmed) => trimmed.basis_curve(),
        None => curve.clone(),
    };
    basis.cast::<Geom2dCircle>().is_some() || basis.cast::<Geom2dLine>().is_some()
}

impl Bisec {
    /// `Perform(Cu, Pnt, P, V1, V2, Sense, Tolerance, oncurve)`: the bisector of a
    /// curve and a point at its end; `bisec.perform_curve_point(&c, &pnt, p, v1, v2, 1.0, 1e-7, true)?`.
    #[allow(clippy::too_many_arguments)] // The native signature.
    pub fn perform_curve_point(
        &mut self,
        cu: &Handle<dyn Geom2dCurve>,
        pnt: &Handle<dyn Geom2dPoint>,
        p: Point2,
        v1: Vector2,
        v2: Vector2,
        sense: f64,
        tolerance: f64,
        oncurve: bool,
    ) -> Result<(), Failure> {
        let result = if is_analytic(cu) {
            let bis_ana = BisecAna::new();
            bis_ana.perform_curve_point(cu, pnt, p, v1, v2, sense, tolerance, oncurve)?;
            ana_result(bis_ana)
        } else {
            let input = MixedInput {
                curve: cu,
                point: pnt.pnt2d(),
                apoint: p,
                v1,
                v2,
                adirection: sense,
                side: PointSide::AtCurveEnd,
            };
            mixed_bisector(&input, oncurve)?
        };
        self.trim_clamped(result)
    }
    /// `Perform(Pnt, Cu, P, V1, V2, Sense, Tolerance, oncurve)`: the bisector of a
    /// point at the start of a curve and the curve;
    /// `bisec.perform_point_curve(&pnt, &c, p, v1, v2, 1.0, 1e-7, true)?`.
    #[allow(clippy::too_many_arguments)] // The native signature.
    pub fn perform_point_curve(
        &mut self,
        pnt: &Handle<dyn Geom2dPoint>,
        cu: &Handle<dyn Geom2dCurve>,
        p: Point2,
        v1: Vector2,
        v2: Vector2,
        sense: f64,
        tolerance: f64,
        oncurve: bool,
    ) -> Result<(), Failure> {
        let result = if is_analytic(cu) {
            let bis_ana = BisecAna::new();
            bis_ana.perform_point_curve(pnt, cu, p, v1, v2, sense, tolerance, oncurve)?;
            ana_result(bis_ana)
        } else {
            let input = MixedInput {
                curve: cu,
                point: pnt.pnt2d(),
                apoint: p,
                v1,
                v2,
                adirection: sense,
                side: PointSide::AtCurveStart,
            };
            mixed_bisector(&input, oncurve)?
        };
        self.trim_clamped(result)
    }
    /// `Perform(Pnt1, Pnt2, P, V1, V2, Sense, Tolerance, oncurve)`: the analytic
    /// bisector of two points; `bisec.perform_points(&p1, &p2, p, v1, v2, 1.0, 0.0, true)?`.
    #[allow(clippy::too_many_arguments)] // The native signature.
    pub fn perform_points(
        &mut self,
        pnt1: &Handle<dyn Geom2dPoint>,
        pnt2: &Handle<dyn Geom2dPoint>,
        p: Point2,
        v1: Vector2,
        v2: Vector2,
        sense: f64,
        tolerance: f64,
        oncurve: bool,
    ) -> Result<(), Failure> {
        let bis_ana = BisecAna::new();
        bis_ana.perform_points(pnt1, pnt2, p, v1, v2, sense, tolerance, oncurve)?;
        let (u_first, u_last) = (
            bis_ana.parameter_of_start_point(),
            bis_ana.parameter_of_end_point(),
        );
        self.trim(&Handle::new(bis_ana).cast(), u_first, u_last)
    }
    // The final trim of the mixed overloads, clamped to the range of the bisector.
    fn trim_clamped(
        &mut self,
        (bis, u_first, u_last): (Handle<dyn BisectorCurve>, f64, f64),
    ) -> Result<(), Failure> {
        let u_first: f64 = std_max(u_first, bis.first_parameter());
        let u_last: f64 = std_min(u_last, bis.last_parameter());
        self.trim(&bis, u_first, u_last)
    }
}

// The analytic bisector with its start and end parameters.
fn ana_result(bis_ana: BisecAna) -> (Handle<dyn BisectorCurve>, f64, f64) {
    let (u_first, u_last) = (
        bis_ana.parameter_of_start_point(),
        bis_ana.parameter_of_end_point(),
    );
    (Handle::new(bis_ana).cast(), u_first, u_last)
}

// The bisector of a point and a curve that is not a circle or a line: a half line
// where the curve is convex (or its curvature is maximal at the point), else the
// curve-point bisector.
fn mixed_bisector(
    input: &MixedInput<'_>,
    oncurve: bool,
) -> Result<(Handle<dyn BisectorCurve>, f64, f64), Failure> {
    let mut rc: f64 = precision::infinite();
    if oncurve {
        let u_point: f64 = match input.side {
            PointSide::AtCurveEnd => input.curve.last_parameter(),
            PointSide::AtCurveStart => input.curve.first_parameter(),
        };
        if is_convex(input.curve, input.adirection)? || is_max_rc(&**input.curve, u_point, &mut rc)?
        {
            let n = Direction2::new(
                -input.adirection * input.v1.y(),
                input.adirection * input.v1.x(),
            )?;
            return half_line(input.apoint, n, rc);
        }
    }
    let bis_pc = match input.side {
        PointSide::AtCurveEnd => BisecPC::with_curve(
            &input.curve.reversed()?,
            input.point,
            -input.adirection,
            500.,
        )?,
        PointSide::AtCurveStart => {
            BisecPC::with_curve(input.curve, input.point, input.adirection, 500.)?
        }
    };
    // KHROMOV, Feb 2002: an empty bisector becomes a half line.
    if bis_pc.is_empty() {
        let (nx, ny) = opposite_sum(input.v1, input.v2)?;
        let n = Direction2::new(input.adirection * nx, input.adirection * ny)?;
        return half_line(input.apoint, n, rc);
    }
    let u_first: f64 = bis_pc.parameter(input.apoint)?;
    let u_last: f64 = bis_pc.last_parameter();
    if u_first >= u_last {
        // Extrapolate by the line of the chord of the bisector.
        let v = Vector2::between(
            bis_pc.value(bis_pc.first_parameter())?,
            bis_pc.value(u_last)?,
        );
        return half_line(input.apoint, Direction2::from_vector(v)?, rc);
    }
    Ok((Handle::new(bis_pc).cast(), u_first, u_last))
}

// The curvature at `u`: 0 where the square norm of the tangent is below the
// resolution.
fn unsigned_curvature(c: &dyn Geom2dCurve, u: f64) -> Result<f64, Failure> {
    let (_, d1, d2) = c.d2(u)?;
    let norm2: f64 = d1.square_magnitude();
    if norm2 < gp::resolution() {
        return Ok(0.0);
    }
    Ok(d1.crossed(d2).abs() / (norm2 * norm2.sqrt()))
}

// `IsMaxRC(C, U, R)`: true when the curvature at the end `u` is smaller than at the
// other end; `r` is then the radius of curvature there.
fn is_max_rc(c: &dyn Geom2dCurve, u: f64, r: &mut f64) -> Result<bool, Failure> {
    let (us, ul) = (c.first_parameter(), c.last_parameter());
    let kf: f64 = unsigned_curvature(c, us)?;
    let kl: f64 = unsigned_curvature(c, ul)?;
    let (k_here, k_other) = if u == ul { (kl, kf) } else { (kf, kl) };
    if k_here < k_other {
        *r = if k_here == 0.0 {
            precision::infinite()
        } else {
            1. / k_here
        };
        return Ok(true);
    }
    Ok(false)
}
