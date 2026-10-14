//! `intersections`: the sweep over curve bounds that finds intersections within one loop
//! and between loops of a planar region.
use super::planar_region::BRepFontPlanarRegion;
use super::planar_region_curve::BRepFontPlanarRegionCurve as Curve;
use super::regularizer_line_intersect::{BRepFontLineIntersection, BRepFontLineIntersectionKind};
use super::regularizer_line_intersect::{intersect_line_quadratic, intersect_lines};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBezierCurve, Geom2dCurve};
use rustycad_tkgeomalgo::geom2d_api::InterCurveCurve;
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::gp::{Point2, Xy};
use rustycad_tkservice::font::FontGlyphOutlineSegmentKind as Kind;
use std::collections::HashSet;

/// `IntersectionKind`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum BRepFontIntersectionKind {
    None,
    BetweenLoops,
    WithinLoop,
}

/// `IntersectionCurve`: the enlarged bounds of one curve in the x sweep.
struct SweepCurve {
    index: u32,
    bounds: BoundingBox2,
    min_x: f64,
    max_x: f64,
}

/// `loopPairKey`: the unordered pair of loop indices as one key.
pub(super) fn loop_pair_key(first: u32, second: u32) -> u64 {
    (u64::from(first.min(second)) << 32) | u64::from(first.max(second))
}

/// `curveGeometry`: the Geom2d Bezier of a region curve (2, 3 or 4 poles).
pub(super) fn curve_geometry(curve: &Curve) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let point = |xy: &Xy| Point2::new(xy.x(), xy.y());
    let mut poles: Vec<Point2> = match curve.kind() {
        Kind::Line => vec![point(curve.start()), point(curve.end())],
        Kind::QuadraticBezier => vec![
            point(curve.start()),
            point(curve.control1()),
            point(curve.end()),
        ],
        Kind::CubicBezier => vec![
            point(curve.start()),
            point(curve.control1()),
            point(curve.control2()),
            point(curve.end()),
        ],
    };
    let bezier = Geom2dBezierCurve::new(&Array1::from_slice(&mut poles))?;
    Ok(Handle::new(bezier).cast())
}

/// The per-curve loop index and position in its loop, and lazily built Geom2d curves.
struct SweepContext<'a> {
    region: &'a BRepFontPlanarRegion,
    tolerance: f64,
    curve_loops: Vec<u32>,
    curve_offsets: Vec<u32>,
    geometry: Vec<Handle<dyn Geom2dCurve>>,
}

impl SweepContext<'_> {
    fn curves(&self) -> &[Curve] {
        self.region.curves().as_slice()
    }
    fn geometry(&mut self, curve_index: u32) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        let slot = curve_index as usize;
        if self.geometry[slot].is_null() {
            self.geometry[slot] = curve_geometry(&self.curves()[slot])?;
        }
        Ok(self.geometry[slot].clone())
    }
}

/// `intersections`: `WithinLoop` at the first intersection inside one loop, otherwise
/// `BetweenLoops` with every intersecting loop pair recorded, or `None`.
/// A Geom2d intersector failure (a native exception) also reports `WithinLoop`, which
/// the caller turns into a failed build.
pub(super) fn intersections(
    region: &BRepFontPlanarRegion,
    tolerance: f64,
    loop_pairs: &mut HashSet<u64>,
) -> BRepFontIntersectionKind {
    let Some(mut context) = sweep_context(region, tolerance) else {
        return BRepFontIntersectionKind::WithinLoop;
    };
    let sweep = match sweep_curves(&mut context) {
        Ok(Some(sweep)) => sweep,
        Ok(None) | Err(_) => return BRepFontIntersectionKind::WithinLoop,
    };
    sweep_pairs(&mut context, &sweep, loop_pairs).unwrap_or(BRepFontIntersectionKind::WithinLoop)
}

fn sweep_context(region: &BRepFontPlanarRegion, tolerance: f64) -> Option<SweepContext<'_>> {
    let nb_curves = region.curves().len();
    let mut curve_loops = vec![0u32; nb_curves];
    let mut curve_offsets = vec![0u32; nb_curves];
    for (loop_index, loop_) in region.loops().iter().enumerate() {
        for offset in 0..loop_.nb_uses() {
            let use_ = &region.uses().as_slice()[(loop_.first_use() + offset) as usize];
            let curve_index = use_.curve_index().value() as usize;
            if curve_index >= nb_curves {
                return None;
            }
            curve_loops[curve_index] = loop_index as u32;
            curve_offsets[curve_index] = offset;
        }
    }
    Some(SweepContext {
        region,
        tolerance,
        curve_loops,
        curve_offsets,
        geometry: vec![Handle::default(); nb_curves],
    })
}

// A quadratic needs no self check when it is monotonic in x or in y.
fn needs_self_check(curve: &Curve) -> bool {
    match curve.kind() {
        Kind::Line => false,
        Kind::CubicBezier => true,
        Kind::QuadraticBezier => {
            let first = *curve.control1() - *curve.start();
            let last = *curve.end() - *curve.control1();
            !is_monotonic(first.x(), last.x()) && !is_monotonic(first.y(), last.y())
        }
    }
}

fn is_monotonic(first: f64, last: f64) -> bool {
    (first >= 0.0 && last >= 0.0 && (first > 0.0 || last > 0.0))
        || (first <= 0.0 && last <= 0.0 && (first < 0.0 || last < 0.0))
}

// The self checks and the sorted sweep list; None when a curve intersects itself.
fn sweep_curves(context: &mut SweepContext<'_>) -> Result<Option<Vec<SweepCurve>>, Failure> {
    let mut sweep = Vec::with_capacity(context.curves().len());
    for index in 0..context.curves().len() as u32 {
        let curve = context.curves()[index as usize];
        if needs_self_check(&curve) {
            let geometry = context.geometry(index)?;
            let intersector = InterCurveCurve::of_self(&geometry, context.tolerance)?;
            if intersector.nb_points() > 0 || intersector.nb_segments() > 0 {
                return Ok(None);
            }
        }
        let mut bounds = curve.bounds();
        bounds.enlarge(context.tolerance);
        let (min_x, max_x) = (bounds.x_min(), bounds.x_max());
        sweep.push(SweepCurve {
            index,
            bounds,
            min_x,
            max_x,
        });
    }
    sweep.sort_by(|left, right| {
        (left.min_x, left.max_x, left.index)
            .partial_cmp(&(right.min_x, right.max_x, right.index))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    Ok(Some(sweep))
}

fn sweep_pairs(
    context: &mut SweepContext<'_>,
    sweep: &[SweepCurve],
    loop_pairs: &mut HashSet<u64>,
) -> Result<BRepFontIntersectionKind, Failure> {
    let mut result = BRepFontIntersectionKind::None;
    for (position, first) in sweep.iter().enumerate() {
        for second in &sweep[position + 1..] {
            if second.min_x > first.max_x {
                break;
            }
            if first.bounds.is_out_box(&second.bounds) {
                continue;
            }
            match pair_intersection(context, first.index, second.index)? {
                PairOutcome::Disjoint => {}
                PairOutcome::WithinLoop => return Ok(BRepFontIntersectionKind::WithinLoop),
                PairOutcome::BetweenLoops => {
                    let loops = &context.curve_loops;
                    loop_pairs.insert(loop_pair_key(
                        loops[first.index as usize],
                        loops[second.index as usize],
                    ));
                    result = BRepFontIntersectionKind::BetweenLoops;
                }
            }
        }
    }
    Ok(result)
}

/// The outcome of one curve pair.
enum PairOutcome {
    Disjoint,
    WithinLoop,
    BetweenLoops,
}

/// The shared endpoint of two consecutive curves of one loop (`isAdjacent`, `aSharedPoint`).
fn shared_point(context: &SweepContext<'_>, first: u32, second: u32) -> Option<Xy> {
    let loop_ = &context.region.loops().as_slice()[context.curve_loops[first as usize] as usize];
    let (mut earlier, mut earlier_offset) = (first, context.curve_offsets[first as usize]);
    let (mut later, mut later_offset) = (second, context.curve_offsets[second as usize]);
    if later_offset < earlier_offset {
        std::mem::swap(&mut earlier, &mut later);
        std::mem::swap(&mut earlier_offset, &mut later_offset);
    }
    if later_offset == earlier_offset + 1 {
        return Some(*context.curves()[earlier as usize].end());
    }
    if earlier_offset == 0 && later_offset + 1 == loop_.nb_uses() {
        return Some(*context.curves()[earlier as usize].start());
    }
    None
}

fn pair_intersection(
    context: &mut SweepContext<'_>,
    first: u32,
    second: u32,
) -> Result<PairOutcome, Failure> {
    let same_loop = context.curve_loops[first as usize] == context.curve_loops[second as usize];
    let shared = if same_loop {
        shared_point(context, first, second)
    } else {
        None
    };
    let found = match direct_intersection(context, first, second) {
        Some(direct) => direct_crossing(&direct, shared, context.tolerance),
        None => general_crossing(context, first, second, shared)?,
    };
    Ok(match (found, same_loop) {
        (false, _) => PairOutcome::Disjoint,
        (true, true) => PairOutcome::WithinLoop,
        (true, false) => PairOutcome::BetweenLoops,
    })
}

// The closed forms for line/line and line/quadratic pairs; None for the other kinds.
fn direct_intersection(
    context: &SweepContext<'_>,
    first: u32,
    second: u32,
) -> Option<BRepFontLineIntersection> {
    let curves = context.curves();
    let (a, b) = (&curves[first as usize], &curves[second as usize]);
    match (a.kind(), b.kind()) {
        (Kind::Line, Kind::Line) => Some(intersect_lines(a, b, context.tolerance)),
        (Kind::Line, Kind::QuadraticBezier) => {
            Some(intersect_line_quadratic(a, b, context.tolerance))
        }
        (Kind::QuadraticBezier, Kind::Line) => {
            Some(intersect_line_quadratic(b, a, context.tolerance))
        }
        _ => None,
    }
}

// An overlap, or a point away from the shared endpoint of adjacent curves, intersects.
fn direct_crossing(direct: &BRepFontLineIntersection, shared: Option<Xy>, tolerance: f64) -> bool {
    if direct.kind == BRepFontLineIntersectionKind::Overlap {
        return true;
    }
    let square_tolerance = tolerance * tolerance;
    direct.points[..direct.nb_points].iter().any(|point| {
        shared.is_none_or(|shared| (*point - shared).square_modulus() > square_tolerance)
    })
}

// Geom2dAPI_InterCurveCurve for the remaining pairs.
fn general_crossing(
    context: &mut SweepContext<'_>,
    first: u32,
    second: u32,
    shared: Option<Xy>,
) -> Result<bool, Failure> {
    let (first_curve, second_curve) = (context.geometry(first)?, context.geometry(second)?);
    let intersector = InterCurveCurve::between(&first_curve, &second_curve, context.tolerance)?;
    if intersector.nb_segments() > 0 {
        return Ok(true);
    }
    let square_tolerance = context.tolerance * context.tolerance;
    for index in 1..=intersector.nb_points() {
        let point = intersector.point(index)?.xy();
        if shared.is_none_or(|shared| (point - shared).square_modulus() > square_tolerance) {
            return Ok(true);
        }
    }
    Ok(false)
}
