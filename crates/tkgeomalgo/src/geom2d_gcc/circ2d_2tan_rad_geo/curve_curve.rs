// Rust translation of the curve-curve constructor and PrecRoot of
// Geom2dGcc_Circ2d2TanRadGeo.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The circles of given radius tangent to two curves.
use super::{Circ2d2TanRadGeo, NB_SOL_MAX, cotes, offset_of};
use crate::gcc_ana::check_qualifier;
use crate::geom2d_gcc::domains::std_min;
use crate::geom2d_gcc::{CurveTool, QCurve};
use crate::geom2d_int::GInter;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::{epsilon, is_equal, real_first};
use rustycad_tkg2d::adaptor2d::{Adaptor2dCurve2d, Adaptor2dOffsetCurve};
use rustycad_tkmath::gp::{Point2, Vector2};

const MEMBER: &str = "Geom2dGcc_Circ2d2TanRadGeo";

impl Circ2d2TanRadGeo {
    /// `Geom2dGcc_Circ2d2TanRadGeo(Qualified1, Qualified2, Radius, Tolerance)`: the two
    /// offset curves intersected by Geom2dInt_GInter; near a tangency of the offsets
    /// the centre is refined by PrecRoot. The stored argument parameters are the
    /// unrefined intersection ones, and the second qualifier is never written, as
    /// natively; `Circ2d2TanRadGeo::curve_curve(&q1, &q2, 1.0, 1e-9)?`.
    pub fn curve_curve(
        qualified1: &QCurve,
        qualified2: &QCurve,
        radius: f64,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let qualifiers_ok = check_qualifier(qualified1.qualifier(), MEMBER)
            .and_then(|()| check_qualifier(qualified2.qualifier(), MEMBER));
        let mut circ = Self::start(qualifiers_ok, radius)?;
        let tol: f64 = tolerance.abs();
        let (cote1, cote2) = cotes(qualified1.qualifier(), qualified2.qualifier(), radius);
        let mut intp = GInter::new();
        for &side1 in &cote1 {
            let c1 = offset_of(qualified1, side1)?;
            for &side2 in &cote2 {
                if circ.solutions.nbr_sol >= NB_SOL_MAX {
                    break;
                }
                let c2 = offset_of(qualified2, side2)?;
                intp.perform_between(&c1, &c2, tol, tol)?;
                circ.take_offset_centres(&intp, [qualified1, qualified2], [&c1, &c2], radius)?;
            }
        }
        Ok(circ)
    }

    // The centres of one offset-offset intersection.
    fn take_offset_centres(
        &mut self,
        intp: &GInter,
        [qualified1, qualified2]: [&QCurve; 2],
        [c1, c2]: [&Adaptor2dOffsetCurve; 2],
        radius: f64,
    ) -> Result<(), Failure> {
        if !intp.is_done() {
            return Ok(());
        }
        if !intp.is_empty()? {
            let mut i: i32 = 1;
            while i <= intp.nb_points()? && self.solutions.nbr_sol < NB_SOL_MAX {
                let point = intp.point(i)?;
                let (u, _) = refined_root(c1, c2, point.param_on_first(), point.param_on_second())?;
                let center: Point2 = c1.value(u)?;
                let (pararg1, pararg2) = (point.param_on_first(), point.param_on_second());
                let pnttg1sol: Point2 = CurveTool::value(qualified1.curve(), pararg1)?;
                let pnttg2sol: Point2 = CurveTool::value(qualified2.curve(), pararg2)?;
                // qualifier1 is written twice natively and qualifier2 never: its slot
                // keeps its start value.
                let q2 = self.solutions.qualifier_of(1, self.solutions.nbr_sol + 1)?;
                let tangencies = [(pnttg1sol, pararg1), (pnttg2sol, pararg2)];
                self.push(center, radius, (qualified1.qualifier(), q2), tangencies)?;
                i += 1;
            }
        }
        self.solutions.well_done = true;
        Ok(())
    }
}

// The intersection parameters (u0, v0), refined by PrecRoot when both offsets pass
// within Precision::Approximation() of each other on both sides of the point
// (a tangent zone).
fn refined_root(
    c1: &Adaptor2dOffsetCurve,
    c2: &Adaptor2dOffsetCurve,
    u0: f64,
    v0: f64,
) -> Result<(f64, f64), Failure> {
    let sq_approx_tol: f64 = precision::approximation() * precision::approximation();
    let p_approx: f64 = precision::p_approximation();
    let (u1, v1, u2, v2) = (u0 - p_approx, v0 - p_approx, u0 + p_approx, v0 + p_approx);
    let (p11, p12) = (c1.value(u1)?, c2.value(v1)?);
    let (p21, p22) = (c1.value(u2)?, c2.value(v2)?);
    let dist1112: f64 = p11.square_distance(p12);
    let dist1122: f64 = p11.square_distance(p22);
    let dist1221: f64 = p12.square_distance(p21);
    let dist2122: f64 = p21.square_distance(p22);
    if std_min(dist1112, dist1122) <= sq_approx_tol && std_min(dist1221, dist2122) <= sq_approx_tol
    {
        return prec_root(c1, c2, u0, v0);
    }
    Ok((u0, v0))
}

// The evaluations of both curves at (u, v): (Pu, D1u, D2u, Pv, D1v, D2v).
type PairD2 = (Point2, Vector2, Vector2, Point2, Vector2, Vector2);

fn pair_d2(
    c1: &Adaptor2dOffsetCurve,
    c2: &Adaptor2dOffsetCurve,
    u: f64,
    v: f64,
) -> Result<PairD2, Failure> {
    let (pu, d1u, d2u) = c1.d2(u)?;
    let (pv, d1v, d2v) = c2.d2(v)?;
    Ok((pu, d1u, d2u, pv, d1v, d2v))
}

// `PrecRoot`: when the curves are tangent at (U0, V0), the Newton refinement of the
// extremum system (Vuv . D1u = 0, Vuv . D1v = 0) of Extrema curve-curve, keeping the
// iterate of smallest distance; up to 101 iterations.
fn prec_root(
    c1: &Adaptor2dOffsetCurve,
    c2: &Adaptor2dOffsetCurve,
    u0: f64,
    v0: f64,
) -> Result<(f64, f64), Failure> {
    let (mut u_final, mut v_final) = (u0, v0);
    let (mut u, mut v) = (u0, v0);
    let mut eval: PairD2 = pair_d2(c1, c2, u, v)?;
    let cr_prod: f64 = eval.1.crossed(eval.4).abs();
    if cr_prod * cr_prod > 1.0e-6 * eval.1.square_magnitude() * eval.4.square_magnitude() {
        // Curves are not tangent: the 2D intersection point needs no precision.
        return Ok((u_final, v_final));
    }
    let mut state = NewtonState {
        nb_iter: 0,
        step_u: 0.0,
        step_v: 0.0,
        sq_dist_prev: real_first(),
    };
    loop {
        state.nb_iter += 1;
        let vuv = Vector2::between(eval.3, eval.0);
        let sq_dist: f64 = vuv.square_magnitude();
        if is_equal(sq_dist, 0.0) {
            break;
        }
        if state.nb_iter == 1 || sq_dist < state.sq_dist_prev {
            state.sq_dist_prev = sq_dist;
            (u_final, v_final) = (u, v);
        }
        if !state.newton_step(&eval, vuv, &mut u, &mut v) {
            break;
        }
        eval = pair_d2(c1, c2, u, v)?;
        if state.nb_iter > 100 {
            break;
        }
    }
    Ok((u_final, v_final))
}

// The iteration counters and last steps of PrecRoot.
struct NewtonState {
    nb_iter: i32,
    step_u: f64,
    step_v: f64,
    sq_dist_prev: f64,
}

impl NewtonState {
    // One Newton step on (u, v); false where PrecRoot stops (a null derivative, a
    // singular system without a previous step, or steps below Epsilon).
    fn newton_step(&mut self, eval: &PairD2, vuv: Vector2, u: &mut f64, v: &mut f64) -> bool {
        let (_, d1u, d2u, _, d1v, d2v) = *eval;
        let (g1, g2): (f64, f64) = (d1u.magnitude(), d1v.magnitude());
        if is_equal(g1, 0.0) || is_equal(g2, 0.0) {
            // Here we do not processing singular cases.
            return false;
        }
        let (f1, f2): (f64, f64) = (vuv.dot(d1u), vuv.dot(d1v));
        let (fi_u, fi_v): (f64, f64) = (vuv.dot(d2u), vuv.dot(d2v));
        let (psi_u, psi_v): (f64, f64) = (d1u.dot(d2u), d1v.dot(d2v));
        let theta: f64 = d1u.dot(d1v);
        let (s1, s2): (f64, f64) = (f1 / g1, f2 / g2);
        let ds1u: f64 = (g1 * g1 + fi_u) / g1 - (s1 * psi_u / (g1 * g1));
        let ds1v: f64 = -theta / g1;
        let ds2u: f64 = theta / g2;
        let ds2v: f64 = (fi_v - g2 * g2) / g2 - (s2 * psi_v / (g2 * g2));
        let det: f64 = ds1u * ds2v - ds1v * ds2u;
        if is_equal(det, 0.0) {
            return self.singular_step(s1, ds1u, ds1v, u, v);
        }
        self.step_u = -(s1 * ds2v - s2 * ds1v) / det;
        self.step_v = -(s2 * ds1u - s1 * ds2u) / det;
        if self.step_u.abs() < epsilon(u.abs()) && self.step_v.abs() < epsilon(v.abs()) {
            return false;
        }
        *u += self.step_u;
        *v += self.step_v;
        true
    }

    // The step of a singular system: along the previous V (then U) step.
    fn singular_step(&self, s1: f64, ds1u: f64, ds1v: f64, u: &mut f64, v: &mut f64) -> bool {
        if !is_equal(self.step_v, 0.0) && !is_equal(ds1u, 0.0) {
            *v += self.step_v;
            *u -= (ds1v * self.step_v - s1) / ds1u;
            return true;
        }
        if !is_equal(self.step_u, 0.0) && !is_equal(ds1v, 0.0) {
            *u += self.step_u;
            *v -= (ds1u * self.step_u - s1) / ds1v;
            return true;
        }
        false
    }
}
