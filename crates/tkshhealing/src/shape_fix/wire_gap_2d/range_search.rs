//! The three searches of the FixGap2d range branch, in native order: the intersections
//! of both pcurves, then their extrema, then the projections of the junction points.
use super::Failure;
use super::ranges::RangeGap;
use rustycad_tkernel::precision;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkgeomalgo::geom2d_api::{ExtremaCurveCurve, ProjectPointOnCurve};
use rustycad_tkgeomalgo::geom2d_int::GInter;
use rustycad_tkgeomalgo::int_res2d::{Domain, Intersection, IntersectionPoint};
use rustycad_tkmath::gp::Point2;

/// Sets `state.ipar` and returns true when a search finds an acceptable pair;
/// `range_search::find(&mut state)?`.
pub(super) fn find(state: &mut RangeGap) -> Result<bool, Failure> {
    for search in [by_intersection, by_extrema, by_projection] {
        if let Some(pair) = search(state)? {
            state.ipar = [pair.0, pair.1];
            return Ok(true);
        }
    }
    Ok(false)
}

fn domain_of(state: &RangeGap, i: usize) -> Result<Domain, Failure> {
    let (first, last) = state.domains[i];
    let curve = &state.curves[i];
    let tolerance = precision::p_confusion();
    Ok(Domain::bounded(
        curve.value(first)?,
        first,
        tolerance,
        curve.value(last)?,
        last,
        tolerance,
    ))
}

fn by_intersection(state: &RangeGap) -> Result<Option<(f64, f64)>, Failure> {
    let adaptors = [
        Geom2dAdaptorCurve::with_curve(&state.curves[0])?,
        Geom2dAdaptorCurve::with_curve(&state.curves[1])?,
    ];
    let (d1, d2) = (domain_of(state, 0)?, domain_of(state, 1)?);
    let tolerance = precision::p_confusion();
    let inter = GInter::with_domains(&adaptors[0], &d1, &adaptors[1], &d2, tolerance, tolerance)?;
    if !inter.is_done() || (inter.nb_points()? == 0 && inter.nb_segments()? == 0) {
        return Ok(None);
    }
    let chosen = nearest_to_ends(state, &inter)?;
    let point = intersection_point(&inter, chosen)?;
    let u = (
        state.adjust(0, point.param_on_first())?,
        state.adjust(1, point.param_on_second())?,
    );
    Ok(state.accepts(u)?.then_some(u))
}

/// An intersection: `(index, flag)` with flag 0 for a point, 1 or 2 for the first or
/// last point of a segment.
type Pick = (i32, i32);

/// The points of `inter` with their picks in native order: the points, then the first and
/// last point of each segment that has them; `candidates(&inter)?`.
pub(super) fn candidates(inter: &Intersection) -> Result<Vec<(Pick, IntersectionPoint)>, Failure> {
    let mut all = Vec::new();
    for i in 1..=inter.nb_points()? {
        all.push(((i, 0), inter.point(i)?));
    }
    for i in 1..=inter.nb_segments()? {
        let segment = inter.segment(i)?;
        if segment.has_first_point() {
            all.push(((i, 1), segment.first_point()?));
        }
        if segment.has_last_point() {
            all.push(((i, 2), segment.last_point()?));
        }
    }
    Ok(all)
}

/// Keeps `pick` when `distance` beats the running minimum, the native
/// `if (mindist > dist || mindist < 0.)`; `nearer(&mut best, (1, 0), 0.5)`.
pub(super) fn nearer(best: &mut (Pick, f64), pick: Pick, distance: f64) {
    if best.1 > distance || best.1 < 0. {
        *best = (pick, distance);
    }
}

// For each curve, the intersection nearest to its far end in parameter; when they
// differ, the one nearer to the middle of the junction.
fn nearest_to_ends(state: &RangeGap, inter: &Intersection) -> Result<Pick, Failure> {
    let mut best = [((0, 0), -1.0_f64); 2];
    for (pick, ip) in candidates(inter)? {
        let u = [
            state.adjust(0, ip.param_on_first())?,
            state.adjust(1, ip.param_on_second())?,
        ];
        for k in 0..2 {
            nearer(&mut best[k], pick, (state.far[k] - u[k]).abs());
        }
    }
    closer_to_middle(state, inter, [best[0].0, best[1].0])
}

fn closer_to_middle(
    state: &RangeGap,
    inter: &Intersection,
    best: [Pick; 2],
) -> Result<Pick, Failure> {
    if best[0] == best[1] {
        return Ok(best[0]);
    }
    let middle = middle_point(state.points[0], state.points[1]);
    let p1 = intersection_point(inter, best[0])?.value();
    let p2 = intersection_point(inter, best[1])?.value();
    Ok(if p2.distance(middle) < p1.distance(middle) {
        best[1]
    } else {
        best[0]
    })
}

/// `Inter.Point(index)` or an end point of `Inter.Segment(index)`; an index of 0 raises
/// Standard_OutOfRange as natively.
pub(super) fn intersection_point(
    inter: &Intersection,
    (index, flag): Pick,
) -> Result<IntersectionPoint, Failure> {
    match flag {
        0 => inter.point(index),
        1 => inter.segment(index)?.first_point(),
        _ => inter.segment(index)?.last_point(),
    }
}

/// `gp_Pnt2d((a.XY() + b.XY()) * 0.5)`.
pub(super) fn middle_point(a: Point2, b: Point2) -> Point2 {
    Point2::new((a.x() + b.x()) * 0.5, (a.y() + b.y()) * 0.5)
}

fn by_extrema(state: &RangeGap) -> Result<Option<(f64, f64)>, Failure> {
    let ((f1, l1), (f2, l2)) = (state.domains[0], state.domains[1]);
    let extrema = ExtremaCurveCurve::new(&state.curves[0], &state.curves[1], f1, l1, f2, l2)?;
    if extrema.nb_extrema() == 0 {
        return Ok(None);
    }
    let (u1, u2) = extrema.lower_distance_parameters()?;
    let u = (state.adjust(0, u1)?, state.adjust(1, u2)?);
    Ok(state.accepts(u)?.then_some(u))
}

/// The nearest projection of `point` on curve `i` over its search domain, as the native
/// loop picks it (first minimal distance); None without projection.
fn project(state: &RangeGap, i: usize, point: Point2) -> Result<Option<(Point2, f64)>, Failure> {
    let (first, last) = state.domains[i];
    let projection = ProjectPointOnCurve::with_range(point, &state.curves[i], first, last)?;
    let mut nearest: Option<(Point2, f64)> = None;
    let mut minimum = -1.;
    for k in 1..=projection.nb_points() {
        let distance = point.distance(projection.point(k)?);
        if minimum > distance || minimum < 0. {
            minimum = distance;
            nearest = Some((projection.point(k)?, projection.parameter(k)?));
        }
    }
    Ok(nearest)
}

fn by_projection(state: &RangeGap) -> Result<Option<(f64, f64)>, Failure> {
    let mut ipnt = state.points;
    let mut u = state.near;
    // cpnt2 on pc1 and cpnt1 on pc2.
    for i in 0..2 {
        if let Some((point, parameter)) = project(state, i, state.points[1 - i])? {
            (ipnt[i], u[i]) = (point, parameter);
        }
    }
    u = [state.adjust(0, u[0])?, state.adjust(1, u[1])?];
    if both_on_one_side(state, u) {
        // Both projections lie inside or both outside: project the middle point.
        let middle = middle_point(state.points[0], state.points[1]);
        u = state.near;
        for (i, parameter) in u.iter_mut().enumerate() {
            if let Some((_, projected)) = project(state, i, middle)? {
                *parameter = projected;
            }
        }
    } else if state.points[0].distance(ipnt[1]) < state.points[1].distance(ipnt[0]) {
        u[0] = state.near[0];
    } else {
        u[1] = state.near[1];
    }
    let u = (state.adjust(0, u[0])?, state.adjust(1, u[1])?);
    Ok(state.accepts(u)?.then_some(u))
}

// ((rev1 && u1 > clast1) || (!rev1 && u1 < clast1)) && ((rev2 && u2 < cfirst2) || (!rev2 && u2 > cfirst2))
// or the same with every comparison turned.
fn both_on_one_side(state: &RangeGap, u: [f64; 2]) -> bool {
    let (r1, r2) = (state.reversed[0], state.reversed[1]);
    let (c1, c2) = (state.near[0], state.near[1]);
    let inward =
        ((r1 && u[0] > c1) || (!r1 && u[0] < c1)) && ((r2 && u[1] < c2) || (!r2 && u[1] > c2));
    let outward =
        ((r1 && u[0] < c1) || (!r1 && u[0] > c1)) && ((r2 && u[1] > c2) || (!r2 && u[1] < c2));
    inward || outward
}
