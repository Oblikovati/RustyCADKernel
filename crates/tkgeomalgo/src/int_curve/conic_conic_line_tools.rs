// Rust translation of IntCurve_IntConicConic_1.cxx (the line-line helpers:
// LineLineGeometricIntersection, DomainIntersection, FindPositionLL, getDomainParametrs,
// computeIntPoint, CheckLLCoincidence, SegmentToPoint) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The helpers of the line-line intersection.
use crate::int_res2d::{Domain, IntersectionPoint, Position, Situation, Transition, TypeTrans};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Line2, Point2};

/// `TOLERANCE_ANGULAIRE` of IntCurve_IntConicConic_1.cxx (modified by NIZHNY-MKK,
/// Tue Feb 15 2000, from 1e-8 to agree with the position computation).
pub(super) const TOLERANCE_ANGULAIRE: f64 = 1.0e-15;

/// The solution of `LineLineGeometricIntersection`: `nbsol` is 1 (crossing lines,
/// with the parameters and the sine of the half angle), 2 (parallel within `Tol`) or
/// 0 (parallel).
pub(super) struct LineLineSolution {
    pub(super) u1: f64,
    pub(super) u2: f64,
    pub(super) sin_demi_angle: f64,
    pub(super) nbsol: i32,
}

/// `LineLineGeometricIntersection(L1, L2, Tol, U1, U2, SinDemiAngle, nbsol)`.
pub(super) fn line_line_geometric_intersection(
    l1: &Line2,
    l2: &Line2,
    tol: f64,
) -> LineLineSolution {
    let (u1x, u1y) = (l1.direction().x(), l1.direction().y());
    let (u2x, u2y) = (l2.direction().x(), l2.direction().y());
    let uo21x: f64 = l2.location().x() - l1.location().x();
    let uo21y: f64 = l2.location().y() - l1.location().y();
    let mut d: f64 = u1y * u2x - u1x * u2y;
    if d.abs() < TOLERANCE_ANGULAIRE {
        d = u1y * uo21x - u1x * uo21y;
        let nbsol: i32 = if d.abs() <= tol { 2 } else { 0 };
        return LineLineSolution {
            u1: 0.0,
            u2: 0.0,
            sin_demi_angle: 0.0,
            nbsol,
        };
    }
    let u1: f64 = (uo21y * u2x - uo21x * u2y) / d;
    let u2: f64 = (uo21y * u1x - uo21x * u1y) / d;
    if d < 0.0 {
        d = -d;
    }
    if d > 1.0 {
        d = 1.0; // Deja vu !
    }
    LineLineSolution {
        u1,
        u2,
        sin_demi_angle: (0.5 * d.asin()).sin(),
        nbsol: 1,
    }
}

/// `DomainIntersection(Domain, U1inf, U1sup, Res1inf, Res1sup, PosInf, PosSup)`: the
/// range `[U1inf, U1sup]` clipped by the domain (with its tolerances); `(1, -1)` when
/// it misses the domain (the positions are then not set: Middle here).
pub(super) fn domain_intersection(
    domain: &Domain,
    u1inf: f64,
    u1sup: f64,
) -> Result<(f64, f64, Position, Position), Failure> {
    let (res1inf, pos_inf) = if domain.has_first_point() {
        if u1sup < (domain.first_parameter()? - domain.first_tolerance()?) {
            return Ok((1.0, -1.0, Position::Middle, Position::Middle));
        }
        if u1inf > (domain.first_parameter()? + domain.first_tolerance()?) {
            (u1inf, Position::Middle)
        } else {
            (domain.first_parameter()?, Position::Head)
        }
    } else {
        (u1inf, Position::Middle)
    };
    let (res1sup, pos_sup) = if domain.has_last_point() {
        if u1inf > (domain.last_parameter()? + domain.last_tolerance()?) {
            return Ok((1.0, -1.0, pos_inf, Position::Middle));
        }
        if u1sup < (domain.last_parameter()? - domain.last_tolerance()?) {
            (u1sup, Position::Middle)
        } else {
            (domain.last_parameter()?, Position::End)
        }
    } else {
        (u1sup, Position::Middle)
    };
    // When one point is at an end, the parameters are made consistent.
    if res1inf > res1sup {
        if pos_sup == Position::Middle {
            return Ok((res1inf, res1inf, pos_inf, pos_sup));
        }
        return Ok((res1sup, res1sup, pos_inf, pos_sup));
    }
    Ok((res1inf, res1sup, pos_inf, pos_sup))
}

/// `FindPositionLL(Param, Domain)`: Head or End when the parameter is within the
/// tolerance of that bound (the nearer one when both), the parameter then snapped
/// to it; Middle otherwise.
pub(super) fn find_position_ll(param: &mut f64, domain: &Domain) -> Result<Position, Failure> {
    let mut a_d_par: f64 = precision::infinite();
    let mut a_pos: Position = Position::Middle;
    let mut a_res_par: f64 = *param;
    if domain.has_first_point() {
        a_d_par = (*param - domain.first_parameter()?).abs();
        if a_d_par <= domain.first_tolerance()? {
            a_res_par = domain.first_parameter()?;
            a_pos = Position::Head;
        }
    }
    if domain.has_last_point() {
        let a_d2: f64 = (*param - domain.last_parameter()?).abs();
        if a_d2 <= domain.last_tolerance()? && (a_pos == Position::Middle || a_d2 < a_d_par) {
            a_res_par = domain.last_parameter()?;
            a_pos = Position::End;
        }
    }
    *param = a_res_par;
    Ok(a_pos)
}

/// `getDomainParametrs(theDomain, theFirst, theLast, theTol1, theTol2)` (gka
/// 0022833): the bounds (infinite when missing) and their tolerances (0).
pub(super) fn get_domain_parametrs(domain: &Domain) -> Result<[f64; 4], Failure> {
    let first: f64 = if domain.has_first_point() {
        domain.first_parameter()?
    } else {
        -precision::infinite()
    };
    let last: f64 = if domain.has_last_point() {
        domain.last_parameter()?
    } else {
        precision::infinite()
    };
    let tol1: f64 = if domain.has_first_point() {
        domain.first_tolerance()?
    } else {
        0.
    };
    let tol2: f64 = if domain.has_last_point() {
        domain.last_tolerance()?
    } else {
        0.
    };
    Ok([first, last, tol1, tol2])
}

/// The two lines of `computeIntPoint` in the order of the call (current, other),
/// with their domains.
pub(super) struct LinePair<'a> {
    pub(super) cur_domain: &'a Domain,
    pub(super) domain_other: &'a Domain,
    pub(super) cur_lin: &'a Line2,
    pub(super) other_lin: &'a Line2,
}

/// `computeIntPoint(theCurDomain, theDomainOther, theCurLin, theOtherLin,
/// theCosT1T2, theParCur, theParOther, theResInf, theResSup, theNum, theCurTrans,
/// theNewPoint)` (gka 0022833): the single intersection point of two lines whose
/// common zone is shorter than the tolerance, or `None` when it is outside the other
/// domain. `the_num` 2 swaps the roles back to (L1, L2).
pub(super) fn compute_int_point(
    lines: &LinePair<'_>,
    the_cos_t1t2: f64,
    (the_par_cur, the_par_other): (f64, f64),
    (the_res_inf, mut the_res_sup): (f64, f64),
    the_num: i32,
    the_cur_trans: TypeTrans,
) -> Result<Option<IntersectionPoint>, Failure> {
    if (the_res_sup - the_par_cur).abs() > (the_res_inf - the_par_cur).abs() {
        the_res_sup = the_res_inf;
    }
    let mut a_res2: f64 = the_par_other + (the_res_sup - the_par_cur) * the_cos_t1t2;
    let [a_first2, a_last2, a_tol21, a_tol22] = get_domain_parametrs(lines.domain_other)?;
    if a_res2 < a_first2 - a_tol21 || a_res2 > a_last2 + a_tol22 {
        return Ok(None);
    }
    let a_pos1a: Position = find_position_ll(&mut the_res_sup, lines.cur_domain)?;
    let a_pos2a: Position = find_position_ll(&mut a_res2, lines.domain_other)?;
    let (mut a_t1, mut a_t2) = (Transition::default(), Transition::default());
    if the_cur_trans != TypeTrans::Undecided {
        let an_other_trans: TypeTrans = match the_cur_trans {
            TypeTrans::Out => TypeTrans::In,
            TypeTrans::In => TypeTrans::Out,
            _ => TypeTrans::Undecided,
        };
        a_t1.set_value(false, a_pos1a, the_cur_trans);
        a_t2.set_value(false, a_pos2a, an_other_trans);
    } else {
        let an_opposite: bool = the_cos_t1t2 < 0.;
        a_t1.set_touch_value(false, a_pos1a, Situation::Unknown, an_opposite);
        a_t2.set_touch_value(false, a_pos2a, Situation::Unknown, an_opposite);
    }
    let bounds2 = [a_first2, a_last2, a_tol21, a_tol22];
    let Some((a_res_u1, a_res_u2)) = resolved_parameters(
        lines,
        (the_par_cur, the_par_other),
        (the_res_sup, a_res2),
        bounds2,
    )?
    else {
        return Ok(None);
    };
    let a_pres = Point2::from(
        (elclib::line_value_2d(a_res_u1, lines.cur_lin.position()).xy()
            + elclib::line_value_2d(a_res_u2, lines.other_lin.position()).xy())
            * 0.5,
    );
    if the_num == 1 {
        return Ok(Some(IntersectionPoint::new(
            a_pres, a_res_u1, a_res_u2, a_t1, a_t2, false,
        )));
    }
    Ok(Some(IntersectionPoint::new(
        a_pres, a_res_u2, a_res_u1, a_t2, a_t1, false,
    )))
}

// The parameters of the point (gka bug 0022833): the analytic ones when both are
// inside their domains, otherwise projected from the one inside; `None` (PKV) when
// both are out of their domains beyond tolerance.
fn resolved_parameters(
    lines: &LinePair<'_>,
    (the_par_cur, the_par_other): (f64, f64),
    (the_res_sup, a_res2): (f64, f64),
    [a_first2, a_last2, a_tol21, a_tol22]: [f64; 4],
) -> Result<Option<(f64, f64)>, Failure> {
    let [a_first1, a_last1, a_tol11, a_tol12] = get_domain_parametrs(lines.cur_domain)?;
    let is_inside1: bool = the_par_cur >= a_first1 && the_par_cur <= a_last1;
    let is_inside2: bool = the_par_other >= a_first2 && the_par_other <= a_last2;
    if is_inside1 && is_inside2 {
        return Ok(Some((the_par_cur, the_par_other)));
    }
    if is_inside1 {
        let pt1: Point2 = elclib::line_value_2d(a_res2, lines.other_lin.position());
        let a_par1: f64 = elclib::line_parameter_2d(lines.cur_lin.position(), pt1);
        let u1: f64 = if a_par1 >= a_first1 && a_par1 <= a_last1 {
            a_par1
        } else {
            the_res_sup
        };
        return Ok(Some((u1, a_res2)));
    }
    if is_inside2 {
        let a_pt1: Point2 = elclib::line_value_2d(the_res_sup, lines.cur_lin.position());
        let a_par2: f64 = elclib::line_parameter_2d(lines.other_lin.position(), a_pt1);
        let u2: f64 = if a_par2 >= a_first2 && a_par2 <= a_last2 {
            a_par2
        } else {
            a_res2
        };
        return Ok(Some((the_res_sup, u2)));
    }
    // check that parameters are within range on both curves
    if the_par_cur < a_first1 - a_tol11
        || the_par_cur > a_last1 + a_tol12
        || the_par_other < a_first2 - a_tol21
        || the_par_other > a_last2 + a_tol22
    {
        return Ok(None);
    }
    Ok(Some((the_res_sup, a_res2)))
}

/// `CheckLLCoincidence(L1, L2, Domain1, Domain2, theTol)`: both ends of one trimmed
/// line lie within `theTol` of the other line.
pub(super) fn check_ll_coincidence(
    l1: &Line2,
    l2: &Line2,
    domain1: &Domain,
    domain2: &Domain,
    the_tol: f64,
) -> Result<bool, Failure> {
    let is_first1: bool =
        domain1.has_first_point() && l2.distance(domain1.first_point()?) < the_tol;
    let is_last1: bool = domain1.has_last_point() && l2.distance(domain1.last_point()?) < the_tol;
    if is_first1 && is_last1 {
        return Ok(true);
    }
    let is_first2: bool =
        domain2.has_first_point() && l1.distance(domain2.first_point()?) < the_tol;
    let is_last2: bool = domain2.has_last_point() && l1.distance(domain2.last_point()?) < the_tol;
    Ok(is_first2 && is_last2)
}

/// `SegmentToPoint(Pa, T1a, T2a, Pb, T1b, T2b)`: a segment too short to be one
/// becomes the point of its end that lies on a domain end (both positions merged).
pub(super) fn segment_to_point(
    pa: &IntersectionPoint,
    (t1a, t2a): (Transition, Transition),
    pb: &IntersectionPoint,
    (t1b, t2b): (Transition, Transition),
) -> IntersectionPoint {
    if t1b.position_on_curve() == Position::Middle && t2b.position_on_curve() == Position::Middle {
        return *pa;
    }
    if t1a.position_on_curve() == Position::Middle && t2a.position_on_curve() == Position::Middle {
        return *pb;
    }
    let (mut t1, mut t2) = (t1a, t2a);
    let (mut u1, mut u2) = (pa.param_on_first(), pa.param_on_second());
    if t1.position_on_curve() == Position::Middle {
        t1.set_position(t1b.position_on_curve());
        u1 = pb.param_on_first();
    }
    if t2.position_on_curve() == Position::Middle {
        t2.set_position(t2b.position_on_curve());
        u2 = pb.param_on_second();
    }
    IntersectionPoint::new(pa.value(), u1, u2, t1, t2, false)
}
