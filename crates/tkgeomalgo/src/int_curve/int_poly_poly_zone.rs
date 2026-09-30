// Rust translation of IntCurve_IntPolyPolyGen.gxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The steps that the self and the pair intersections of IntCurve_IntPolyPolyGen
//! repeat: the position of a point on a domain, the transitions of a point, and the
//! ends of a tangent zone. The self intersection passes its curve and domain twice,
//! which gives its native code exactly.
use super::{PCurveTool, Polygon2dGen};
use crate::int_imp_par_gen::{
    IntersectorTypes, ParTool, ProjectOnPCurveTool, TransitionSide, determine_crossing_transition,
    determine_transition,
};
use crate::int_res2d::{Domain, IntersectionPoint, Position, Transition};
use crate::intf::TangentZone;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point2, Vector2};

/// The parametric curve of an instantiation.
pub(super) type CurveOf<T> = <<T as IntersectorTypes>::Par as ParTool>::Curve;

/// One curve of an intersection with its search domain (the bounds of
/// `FindParameter`) and the domain that gives the positions (`DomainOnCurve`).
pub(super) struct PolyCurve<'c, T: IntersectorTypes> {
    pub(super) curve: &'c CurveOf<T>,
    pub(super) domain: &'c Domain,
    pub(super) on_curve: &'c Domain,
}

impl<T: IntersectorTypes> Clone for PolyCurve<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: IntersectorTypes> Copy for PolyCurve<'_, T> {}

/// A point of both curves with its tangents, as the native `P1`, `Tan1`, `P2`,
/// `Tan2` locals that the transition code rewrites.
#[derive(Clone, Copy, Default)]
pub(super) struct PointPair {
    pub(super) p1: Point2,
    pub(super) tan1: Vector2,
    pub(super) p2: Point2,
    pub(super) tan2: Vector2,
}

/// The position of `p` on `domain`: Head within the first tolerance of the first
/// point, else End within the last tolerance of the last point.
pub(super) fn position_on(domain: &Domain, p: Point2) -> Result<Position, Failure> {
    if p.distance(domain.first_point()?) <= domain.first_tolerance()? {
        return Ok(Position::Head);
    }
    if p.distance(domain.last_point()?) <= domain.last_tolerance()? {
        return Ok(Position::End);
    }
    Ok(Position::Middle)
}

/// The transitions at `(u, v)`: the crossing test on the first derivatives, then,
/// when it cannot decide, the second-order test after D2 at `(u, v)` rewrites the
/// points and tangents of `pair`.
pub(super) fn point_transitions<T: IntersectorTypes>(
    (c1, u): (&CurveOf<T>, f64),
    (c2, v): (&CurveOf<T>, f64),
    (pos1, pos2): (Position, Position),
    pair: &mut PointPair,
    (trans1, trans2): (&mut Transition, &mut Transition),
    tol_conf: f64,
) -> Result<(), Failure> {
    if determine_crossing_transition(
        (pos1, pair.tan1, &mut *trans1),
        (pos2, pair.tan2, &mut *trans2),
        tol_conf,
    ) {
        return Ok(());
    }
    let (mut norm1, mut norm2) = (Vector2::default(), Vector2::default());
    T::Par::d2(c1, u, &mut pair.p1, &mut pair.tan1, &mut norm1)?;
    T::Par::d2(c2, v, &mut pair.p2, &mut pair.tan2, &mut norm2)?;
    determine_transition(
        TransitionSide {
            pos: pos1,
            tan: &mut pair.tan1,
            norm: norm1,
            trans: trans1,
        },
        TransitionSide {
            pos: pos2,
            tan: &mut pair.tan2,
            norm: norm2,
            trans: trans2,
        },
        tol_conf,
    );
    Ok(())
}

/// The parameter range of a tangent zone on both polygons:
/// `[inf on 1, sup on 1, inf on 2, sup on 2]` (RealLast and -RealLast for an empty
/// zone). A point past the last chord is the end of that chord, one before the
/// first chord its start.
pub(super) fn zone_parameter_range<P1: PCurveTool, P2: PCurveTool>(
    zone: &TangentZone,
    poly1: &Polygon2dGen<P1>,
    poly2: &Polygon2dGen<P2>,
) -> Result<[f64; 4], Failure> {
    let (mut inf1, mut sup1) = (f64::MAX, -f64::MAX);
    let (mut inf2, mut sup2) = (f64::MAX, -f64::MAX);
    for qq in 1..=zone.number_of_points() {
        let spnt1 = zone.get_point(qq)?;
        let (_, seg1, param1) = spnt1.info_first_address();
        let poly_u_inf: f64 = seat_parameter(poly1, seg1, param1)?;
        let (_, seg2, param2) = spnt1.info_second_address();
        let poly_v_inf: f64 = seat_parameter(poly2, seg2, param2)?;
        if inf1 > poly_u_inf {
            inf1 = poly_u_inf;
        }
        if inf2 > poly_v_inf {
            inf2 = poly_v_inf;
        }
        if sup1 < poly_u_inf {
            sup1 = poly_u_inf;
        }
        if sup2 < poly_v_inf {
            sup2 = poly_v_inf;
        }
    }
    Ok([inf1, sup1, inf2, sup2])
}

fn seat_parameter<P: PCurveTool>(
    poly: &Polygon2dGen<P>,
    mut seg: i32,
    mut param_on_line: f64,
) -> Result<f64, Failure> {
    if seg > poly.nb_segments() {
        seg -= 1;
        param_on_line = 1.0;
    }
    if seg <= 0 {
        seg = 1;
        param_on_line = 0.0;
    }
    poly.approx_param_on_curve(seg, param_on_line)
}

/// The parameters of the zone ends `[u inf, u sup, v inf, v sup]`, the second curve
/// reversed when its inf end is nearer the sup end of the first curve's inf end.
pub(super) fn orient_zone<T: IntersectorTypes>(
    c1: &CurveOf<T>,
    c2: &CurveOf<T>,
    [inf1, sup1, inf2, sup2]: [f64; 4],
) -> Result<[f64; 4], Failure>
where
    T::Par: PCurveTool,
{
    let (mut v_inf, mut v_sup) = (inf2, sup2);
    let p1: Point2 = <T::Par as PCurveTool>::d0(c1, inf1)?;
    let mut p2: Point2 = <T::Par as PCurveTool>::d0(c2, v_inf)?;
    let distmemesens: f64 = p1.square_distance(p2);
    p2 = <T::Par as PCurveTool>::d0(c2, v_sup)?;
    let distdiffsens: f64 = p1.square_distance(p2);
    if distmemesens > distdiffsens {
        std::mem::swap(&mut v_inf, &mut v_sup);
    }
    Ok([inf1, sup1, v_inf, v_sup])
}

/// One end of a tangent zone at `(u, v)`: the positions, one parameter projected
/// from the other curve's point (the curve not at a domain end, else the curve of
/// the narrower range), the transitions and the point. Returns the point and the
/// final `(u, v)`; `pair` keeps the tangents for the opposition test.
pub(super) fn zone_end<T: IntersectorTypes>(
    side1: PolyCurve<'_, T>,
    side2: PolyCurve<'_, T>,
    (mut u, mut v): (f64, f64),
    wide_on_first: bool,
    pair: &mut PointPair,
    trans: (&mut Transition, &mut Transition),
    tol_conf: f64,
) -> Result<(IntersectionPoint, f64, f64), Failure> {
    T::Par::d1(side1.curve, u, &mut pair.p1, &mut pair.tan1)?;
    T::Par::d1(side2.curve, v, &mut pair.p2, &mut pair.tan2)?;
    let pos1: Position = position_on(side1.on_curve, pair.p1)?;
    let pos2: Position = position_on(side2.on_curve, pair.p2)?;
    if pos1 == Position::Middle && pos2 != Position::Middle {
        u = project::<T>(side1, pair.p2)?;
    } else if (pos1 != Position::Middle && pos2 == Position::Middle) || wide_on_first {
        // The native code has the two conditions as separate branches.
        v = project::<T>(side2, pair.p1)?;
    } else {
        u = project::<T>(side1, pair.p2)?;
    }
    point_transitions::<T>(
        (side1.curve, u),
        (side2.curve, v),
        (pos1, pos2),
        pair,
        (&mut *trans.0, &mut *trans.1),
        tol_conf,
    )?;
    let point = IntersectionPoint::new(pair.p1, u, v, *trans.0, *trans.1, false);
    Ok((point, u, v))
}

// `TheProjPCur::FindParameter(C, P, D.FirstParameter(), D.LastParameter(), EpsX(C))`.
fn project<T: IntersectorTypes>(side: PolyCurve<'_, T>, p: Point2) -> Result<f64, Failure> {
    T::Project::find_parameter_in(
        side.curve,
        p,
        side.domain.first_parameter()?,
        side.domain.last_parameter()?,
        T::Par::eps_x(side.curve),
    )
}
