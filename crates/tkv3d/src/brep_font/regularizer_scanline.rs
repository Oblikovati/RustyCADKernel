//! The scanline point-in-loop test and interior-point search of the regularizer
//! (`yIntervals`, `horizontalCrossing`, `contains`, `interiorPoint`).
use super::planar_region::{
    BRepFontPlanarRegion, BRepFontPlanarRegionLoop, BRepFontPlanarRegionUse,
};
use super::planar_region_curve::BRepFontPlanarRegionCurve as Curve;
use super::regularizer::is_valid_real;
use rustycad_tkmath::gp::{Point2, Xy};
use rustycad_tkmath::math_poly::LongDouble;
use rustycad_tkservice::font::FontGlyphOutlineSegmentKind as Kind;

/// `coordinateValue`: one Bernstein coordinate of a segment at `t`.
fn coordinate_value(kind: Kind, [start, control1, control2, end]: [f64; 4], t: f64) -> f64 {
    let u = 1.0 - t;
    match kind {
        Kind::Line => start * u + end * t,
        Kind::QuadraticBezier => start * (u * u) + control1 * (2.0 * u * t) + end * (t * t),
        Kind::CubicBezier => {
            start * (u * u * u)
                + control1 * (3.0 * u * u * t)
                + control2 * (3.0 * u * t * t)
                + end * (t * t * t)
        }
    }
}

/// `curveX`.
fn curve_x(curve: &Curve, t: f64) -> f64 {
    let xs = [
        curve.start().x(),
        curve.control1().x(),
        curve.control2().x(),
        curve.end().x(),
    ];
    coordinate_value(curve.kind(), xs, t)
}

/// `curveY`.
fn curve_y(curve: &Curve, t: f64) -> f64 {
    let ys = [
        curve.start().y(),
        curve.control1().y(),
        curve.control2().y(),
        curve.end().y(),
    ];
    coordinate_value(curve.kind(), ys, t)
}

/// The sorted parameters splitting `[first, last]` into y-monotonic pieces (`yIntervals`):
/// `first`, the interior roots of `dy/dt` (computed in `long double`), then `last`.
fn y_intervals(curve: &Curve, first: f64, last: f64) -> ([f64; 4], usize) {
    let mut parameters = [first, 0.0, 0.0, 0.0];
    let mut count = 1;
    let mut append = |root: LongDouble| {
        let root = root.narrow();
        if is_valid_real(root) && root > first && root < last {
            parameters[count] = root;
            count += 1;
        }
    };
    match curve.kind() {
        Kind::Line => {}
        Kind::QuadraticBezier => quadratic_y_roots(curve, &mut append),
        Kind::CubicBezier => cubic_y_roots(curve, &mut append),
    }
    parameters[..count].sort_by(f64::total_cmp);
    parameters[count] = last;
    (parameters, count + 1)
}

fn quadratic_y_roots(curve: &Curve, append: &mut impl FnMut(LongDouble)) {
    let ld = LongDouble::new;
    let (start, control, end) = (
        ld(curve.start().y()),
        ld(curve.control1().y()),
        ld(curve.end().y()),
    );
    let quadratic = start - ld(2.0) * control + end;
    if quadratic != LongDouble::ZERO {
        let linear = ld(2.0) * (control - start);
        append(-linear / (ld(2.0) * quadratic));
    }
}

fn cubic_y_roots(curve: &Curve, append: &mut impl FnMut(LongDouble)) {
    let ld = LongDouble::new;
    let (p0, p1) = (ld(curve.start().y()), ld(curve.control1().y()));
    let (p2, p3) = (ld(curve.control2().y()), ld(curve.end().y()));
    let three = ld(3.0);
    let cubic = -p0 + three * p1 - three * p2 + p3;
    let quadratic = three * (p0 - ld(2.0) * p1 + p2);
    let linear = three * (p1 - p0);
    let a = three * cubic;
    let b = ld(2.0) * quadratic;
    let zero = LongDouble::ZERO;
    if a == zero {
        if b != zero {
            append(-linear / b);
        }
        return;
    }
    let discriminant = b * b - ld(4.0) * a * linear;
    if discriminant < zero {
        return;
    }
    if discriminant == zero {
        append(-b / (ld(2.0) * a));
        return;
    }
    let q = ld(-0.5) * (b + discriminant.sqrt().copysign(b));
    append(q / a);
    append(linear / q);
}

/// `horizontalCrossing`: the parameter in a y-monotonic piece where the curve reaches `y`
/// (closed form for lines, bisection to the last representable midpoint otherwise).
fn horizontal_crossing(
    curve: &Curve,
    (first, first_y): (f64, f64),
    (last, last_y): (f64, f64),
    y: f64,
) -> f64 {
    if curve.kind() == Kind::Line {
        return first + (last - first) * (y - first_y) / (last_y - first_y);
    }
    let (mut low, mut high) = (first, last);
    let is_first_above = first_y > y;
    loop {
        let middle = 0.5 * (low + high);
        if middle == low || middle == high {
            break;
        }
        if (curve_y(curve, middle) > y) == is_first_above {
            low = middle;
        } else {
            high = middle;
        }
    }
    0.5 * (low + high)
}

/// One y-monotonic piece of a use, in use orientation, with its end ordinates.
struct MonotonicPiece {
    first: (f64, f64),
    last: (f64, f64),
}

// The pieces of a use in its own orientation (a reversed use walks the intervals backwards).
fn use_pieces(curve: &Curve, use_: &BRepFontPlanarRegionUse) -> Vec<MonotonicPiece> {
    let (parameters, count) = y_intervals(curve, use_.first_parameter(), use_.last_parameter());
    (0..count - 1)
        .map(|interval| {
            let (first, last) = if use_.is_reversed() {
                (
                    parameters[count - interval - 1],
                    parameters[count - interval - 2],
                )
            } else {
                (parameters[interval], parameters[interval + 1])
            };
            MonotonicPiece {
                first: (first, curve_y(curve, first)),
                last: (last, curve_y(curve, last)),
            }
        })
        .collect()
}

// The uses of a loop with their curves; None when a use names a missing curve.
fn loop_curves<'a>(
    region: &'a BRepFontPlanarRegion,
    loop_: &BRepFontPlanarRegionLoop,
) -> Option<Vec<(&'a Curve, &'a BRepFontPlanarRegionUse)>> {
    (0..loop_.nb_uses())
        .map(|offset| {
            let use_ = &region.uses().as_slice()[(loop_.first_use() + offset) as usize];
            let curve = region
                .curves()
                .as_slice()
                .get(use_.curve_index().value() as usize)?;
            Some((curve, use_))
        })
        .collect()
}

/// `contains`: the nonzero winding number of the loop around `point` (rightward ray).
pub(super) fn contains(
    region: &BRepFontPlanarRegion,
    loop_: &BRepFontPlanarRegionLoop,
    point: &Xy,
) -> bool {
    if loop_
        .bounds()
        .is_out_point(Point2::new(point.x(), point.y()))
    {
        return false;
    }
    let Some(uses) = loop_curves(region, loop_) else {
        return false;
    };
    let mut winding = 0;
    for (curve, use_) in uses {
        for piece in use_pieces(curve, use_) {
            let is_upward = piece.first.1 <= point.y() && piece.last.1 > point.y();
            let is_downward = piece.first.1 > point.y() && piece.last.1 <= point.y();
            if !is_upward && !is_downward {
                continue;
            }
            let parameter = horizontal_crossing(curve, piece.first, piece.last, point.y());
            if curve_x(curve, parameter) > point.x() {
                winding += if is_upward { 1 } else { -1 };
            }
        }
    }
    winding != 0
}

/// `interiorPoint`: the midpoint of the first crossing gap, on the slab midlines between
/// the sorted distinct critical ordinates, that the loop contains.
pub(super) fn interior_point(
    region: &BRepFontPlanarRegion,
    loop_: &BRepFontPlanarRegionLoop,
) -> Option<Xy> {
    let uses = loop_curves(region, loop_)?;
    let critical = critical_ordinates(&uses);
    for slab in critical.windows(2) {
        let y = 0.5 * slab[0] + 0.5 * slab[1];
        if !(y > slab[0] && y < slab[1]) {
            continue;
        }
        let crossings = slab_crossings(&uses, y);
        for gap in crossings.windows(2) {
            let x = 0.5 * gap[0] + 0.5 * gap[1];
            if x > gap[0] && x < gap[1] && contains(region, loop_, &Xy::new(x, y)) {
                return Some(Xy::new(x, y));
            }
        }
    }
    None
}

// The sorted distinct ordinates of every piece boundary.
fn critical_ordinates(uses: &[(&Curve, &BRepFontPlanarRegionUse)]) -> Vec<f64> {
    let mut ordinates = Vec::new();
    for (curve, use_) in uses {
        let (parameters, count) = y_intervals(curve, use_.first_parameter(), use_.last_parameter());
        ordinates.extend(parameters[..count].iter().map(|&t| curve_y(curve, t)));
    }
    ordinates.sort_by(f64::total_cmp);
    ordinates.dedup_by(|current, previous| current == previous);
    ordinates
}

// The sorted abscissae where the loop crosses the horizontal line `y`.
fn slab_crossings(uses: &[(&Curve, &BRepFontPlanarRegionUse)], y: f64) -> Vec<f64> {
    let mut crossings = Vec::new();
    for (curve, use_) in uses {
        for piece in use_pieces(curve, use_) {
            let (first_y, last_y) = (piece.first.1, piece.last.1);
            if (first_y <= y && last_y > y) || (first_y > y && last_y <= y) {
                let parameter = horizontal_crossing(curve, piece.first, piece.last, y);
                crossings.push(curve_x(curve, parameter));
            }
        }
    }
    crossings.sort_by(f64::total_cmp);
    crossings
}
