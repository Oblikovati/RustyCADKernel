// Rust translation of ModelingAlgorithms/TKTopAlgo/Bisector/Bisector_BisecCC.cxx (ValueByInt and the static PointByInt) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The point of the bisector of two curves computed by root finding on the whole
//! second curve (`ValueByInt`), and by intersection with a curve-point bisector
//! (`PointByInt`).
use super::state::{CcState, OnCurves, curvature, discret_par};
use crate::bisector::{BisecPC, BisectorFunctionH, is_convex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkgeomalgo::geom2d_int::GInter;
use rustycad_tkmath::gp::{Direction2, Point2, Vector2};
use rustycad_tkmath::math::FunctionRoots;

// The tangent of `c2` at its end `u` when `p1` is on that end; the native code tests
// the first end, then the last, and evaluates each only when it gets there.
fn end_contact(c2: &dyn Geom2dCurve, p1: Point2, u: f64) -> Result<Option<Vector2>, Failure> {
    if p1.distance(c2.value(u)?) < precision::confusion() {
        return Ok(Some(c2.d1(u)?.1));
    }
    Ok(None)
}

// The candidate point on the normal of C1 at `p1` for the root `u_sol` on C2, when
// the root is on the side `sign1`.
fn candidate(
    c2: &dyn Geom2dCurve,
    p1: Point2,
    n1: Vector2,
    u_sol: f64,
    sign1: f64,
) -> Result<Option<Point2>, Failure> {
    let p2_curve2: Point2 = c2.value(u_sol)?;
    let p2p1 = Vector2::new(p1.x() - p2_curve2.x(), p1.y() - p2_curve2.y());
    let square_p2p1: f64 = p2p1.square_magnitude();
    let n1p2p1: f64 = n1.dot(p2p1);
    if n1p2p1 * sign1 > 0. {
        return Ok(Some(p1.translated(-(0.5 * square_p2p1 / n1p2p1) * n1)));
    }
    Ok(None)
}

impl CcState {
    /// `ValueByInt(U, U1, U2, Distance)`: the point of the bisector on the normal of
    /// C1 at the parameter `u`, from the roots of H on C2 (the part after the polygon);
    /// an infinite distance and the curve point when there is no valid solution.
    pub(crate) fn value_by_int(&self, u: f64, out: &mut OnCurves) -> Result<Point2, Failure> {
        out.u1 = self.link_bis_curve(u);
        let (p1, tan1) = self.curve1.d1(out.u1)?;
        let n1 = Vector2::new(tan1.y(), -tan1.x());
        for u2 in [self.curve2.first_parameter(), self.curve2.last_parameter()] {
            let Some(tan2) = end_contact(&*self.curve2, p1, u2)? else {
                continue;
            };
            out.u2 = u2;
            if (self.is_convex1 && self.is_convex2)
                || !tan1.is_parallel(tan2, precision::angular())?
            {
                out.dist = 0.;
                return Ok(p1);
            }
        }
        let Some((u_first, u_last)) = self.c2_range() else {
            out.dist = precision::infinite();
            return Ok(p1);
        };
        let solution: Option<(Point2, f64, f64)> =
            self.closest_root(p1, n1, tan1, u_first, u_last)?;
        let Some((psol, u2, d_min)) = solution else {
            out.dist = precision::infinite();
            return Ok(p1);
        };
        out.u2 = u2;
        out.dist = d_min;
        if self.accept(p1, psol, out)? {
            return Ok(psol);
        }
        out.dist = precision::infinite();
        Ok(p1)
    }

    // The range of C2 after the polygon; `None` when it is too short.
    fn c2_range(&self) -> Option<(f64, f64)> {
        let mut u_first: f64 = self.curve2.first_parameter();
        let mut u_last: f64 = self.curve2.last_parameter();
        if !self.my_polygon.is_empty() {
            if self.sign1 == self.sign2 {
                u_last = self.my_polygon.last().param_on_c2();
            } else {
                u_first = self.my_polygon.last().param_on_c2();
            }
        }
        if (u_last - u_first).abs() < precision::p_confusion() / 100. {
            return None;
        }
        Some((u_first, u_last))
    }

    // The root of H on `[u_first, u_last]` whose point is the closest to `p1`: the
    // point, its parameter on C2 and its square distance.
    fn closest_root(
        &self,
        p1: Point2,
        n1: Vector2,
        tan1: Vector2,
        u_first: f64,
        u_last: f64,
    ) -> Result<Option<(Point2, f64, f64)>, Failure> {
        let eps_max: f64 = 1.0E-6;
        let eps_h: f64 = 1.0E-8;
        let (_, eps_x, nb_samples) = discret_par((u_last - u_first).abs(), eps_h, eps_max, 2, 20);
        let mut h = BisectorFunctionH::new(&self.curve2, p1, self.sign1 * self.sign2 * tan1)?;
        let roots = FunctionRoots::with_options(
            &mut h,
            u_first,
            u_last,
            nb_samples,
            [eps_x, eps_h, eps_h],
            0.,
        );
        h.raised()?;
        if !roots.is_done() {
            return Ok(None);
        }
        let mut best: Option<(Point2, f64, f64)> = None;
        let mut d_min: f64 = precision::infinite();
        for j in 1..=roots.number_of_solutions()? {
            let u_sol: f64 = roots.value(j)?;
            let Some(p) = candidate(&*self.curve2, p1, n1, u_sol, self.sign1)? else {
                continue;
            };
            let dist_pp1: f64 = p1.square_distance(p);
            if dist_pp1 < d_min {
                d_min = dist_pp1;
                best = Some((p, u_sol, d_min));
            }
        }
        Ok(best)
    }

    // The distance and angular tests of a found point: the two curve points must
    // not be seen in the same direction, and the square distance must not pass the
    // square radius of curvature of a concave curve.
    fn accept(&self, p1: Point2, psol: Point2, out: &OnCurves) -> Result<bool, Failure> {
        let dist: f64 = out.dist;
        let p2: Point2 = self.curve2.value(out.u2)?;
        let pp1 = Vector2::new(p1.x() - psol.x(), p1.y() - psol.y());
        let pp2 = Vector2::new(p2.x() - psol.x(), p2.y() - psol.y());
        // Dist = product of the norms = the square distance.
        if pp1.dot(pp2) > (1. - precision::angular()) * dist {
            return Ok(false);
        }
        if !self.is_convex1 && too_far_square(&*self.curve1, out.u1, dist)? {
            return Ok(false);
        }
        Ok(self.is_convex2 || !too_far_square(&*self.curve2, out.u2, dist)?)
    }
}

// True when the square distance passes the square radius of curvature at `u`.
fn too_far_square(c: &dyn Geom2dCurve, u: f64, dist: f64) -> Result<bool, Failure> {
    let k: f64 = curvature(c, u, precision::confusion())?;
    Ok(k != 0. && dist > 1. / (k * k))
}

// True when the distance passes the radius of curvature at `u`.
fn too_far(c: &dyn Geom2dCurve, u: f64, dist: f64) -> Result<bool, Failure> {
    let k: f64 = curvature(c, u, precision::confusion())?;
    Ok(k != 0. && dist > (1. / k).abs())
}

/// The outputs of `PointByInt`: the parameter on the second curve and the square
/// distance (in-out, as the native references).
pub(crate) struct PointByIntResult {
    pub found: bool,
    pub u_on_b: f64,
    pub dist: f64,
}

/// `PointByInt(CA, CB, SignA, SignB, UOnA, UOnB, Dist)`: the point of the bisector of
/// CA and CB on the normal of CA at `u_on_a`, by intersection of that normal with the
/// bisector of CB and the point of CA.
pub(crate) fn point_by_int(
    ca: &Handle<dyn Geom2dCurve>,
    cb: &Handle<dyn Geom2dCurve>,
    signs: (f64, f64),
    u_on_a: f64,
    mut result: PointByIntResult,
) -> Result<PointByIntResult, Failure> {
    let (sign_a, sign_b) = signs;
    let convex_a: bool = is_convex(ca, sign_a)?;
    let convex_b: bool = is_convex(cb, sign_b)?;
    let (p1, tan1) = ca.d1(u_on_a)?;
    let n1 = Vector2::new(tan1.y(), -tan1.x());
    for u2 in [cb.first_parameter(), cb.last_parameter()] {
        let Some(tan2) = end_contact(&**cb, p1, u2)? else {
            continue;
        };
        result.u_on_b = u2;
        if convex_a && convex_b {
            result.dist = 0.;
            result.found = true;
            return Ok(result);
        }
        if !tan1.is_parallel(tan2, precision::angular())? {
            result.dist = 0.;
            result.found = false;
            return Ok(result);
        }
    }
    let bis_pc: Handle<BisecPC> = Handle::new(BisecPC::with_curve(cb, p1, sign_b, 500.)?);
    if bis_pc.is_empty() {
        result.dist = precision::infinite();
        result.found = false;
        return Ok(result);
    }
    let Some(psol) = intersect_normal(&bis_pc, p1, n1, sign_a, &mut result)? else {
        result.found = false;
        return Ok(result);
    };
    result.found = accept_by_int(
        (ca, cb),
        (convex_a, convex_b),
        (p1, psol, u_on_a),
        &mut result,
    )?;
    Ok(result)
}

// The intersection of the curve-point bisector with the normal at `p1`: the point
// closest to `p1` on the side `sign_a`; it writes the parameter and the square
// distance of each better point.
#[allow(clippy::neg_cmp_op_on_partial_ord)] // The negated native test keeps its NaN answer.
fn intersect_normal(
    bis_pc: &Handle<BisecPC>,
    p1: Point2,
    n1: Vector2,
    sign_a: f64,
    result: &mut PointByIntResult,
) -> Result<Option<Point2>, Failure> {
    let nor_li: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dLine::from_point(p1, Direction2::from_vector(n1)?)).cast();
    let a_bis_pc = Geom2dAdaptorCurve::with_curve(&bis_pc.cast::<dyn Geom2dCurve>())?;
    let a_nor_li = Geom2dAdaptorCurve::with_curve(&nor_li)?;
    let tol: f64 = precision::confusion();
    let intersect = GInter::between(&a_bis_pc, &a_nor_li, tol, tol)?;
    if !intersect.is_done() || intersect.is_empty()? {
        return Ok(None);
    }
    let mut d_min: f64 = precision::infinite();
    let mut psol: Option<Point2> = None;
    for i in 1..=intersect.nb_points()? {
        let point = intersect.point(i)?;
        if !(point.param_on_second() * sign_a < precision::p_confusion()) {
            continue;
        }
        let p: Point2 = point.value();
        if p.square_distance(p1) < d_min {
            d_min = p.square_distance(p1);
            psol = Some(p);
            result.u_on_b = bis_pc.link_bis_curve(point.param_on_first());
            result.dist = d_min;
        }
    }
    Ok(psol)
}

// The tests of a point found by intersection: not on a curve point, the two curve
// points not seen in the same direction, the distance (its square root, written
// back) within the radius of curvature of the concave curves.
fn accept_by_int(
    curves: (&Handle<dyn Geom2dCurve>, &Handle<dyn Geom2dCurve>),
    convex: (bool, bool),
    points: (Point2, Point2, f64),
    result: &mut PointByIntResult,
) -> Result<bool, Failure> {
    let (ca, cb) = curves;
    let (p1, psol, u_on_a) = points;
    let p2: Point2 = cb.value(result.u_on_b)?;
    if p1.square_distance(psol) < 1.0e-32 || p2.square_distance(psol) < 1.0e-32 {
        return Ok(false);
    }
    let pp1_unit = Direction2::new(p1.x() - psol.x(), p1.y() - psol.y())?;
    let pp2_unit = Direction2::new(p2.x() - psol.x(), p2.y() - psol.y())?;
    if pp1_unit.dot(pp2_unit) > 1. - precision::angular() {
        return Ok(false);
    }
    result.dist = result.dist.sqrt();
    if !convex.0 && too_far(&**ca, u_on_a, result.dist)? {
        return Ok(false);
    }
    Ok(convex.1 || !too_far(&**cb, result.u_on_b, result.dist)?)
}
