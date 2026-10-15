//! The closed-form line/line and line/quadratic intersections of the regularizer
//! (`intersectLines`, `intersectLineQuadratic`). The quadratic roots are computed in the
//! x87 `long double` of the pinned Linux/GCC oracle profile.
use super::planar_region_curve::BRepFontPlanarRegionCurve as Curve;
use super::regularizer::is_valid_real;
use rustycad_tkmath::gp::Xy;
use rustycad_tkmath::math_poly::LongDouble;

/// `LineIntersectionKind`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum BRepFontLineIntersectionKind {
    #[default]
    None,
    Point,
    Overlap,
}

/// `LineIntersection`: the kind and up to two distinct points.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct BRepFontLineIntersection {
    pub(super) kind: BRepFontLineIntersectionKind,
    pub(super) points: [Xy; 2],
    pub(super) nb_points: usize,
}

impl BRepFontLineIntersection {
    fn overlap() -> Self {
        Self {
            kind: BRepFontLineIntersectionKind::Overlap,
            ..Self::default()
        }
    }
    fn point(point: Xy) -> Self {
        Self {
            kind: BRepFontLineIntersectionKind::Point,
            points: [point, Xy::default()],
            nb_points: 1,
        }
    }
}

/// `intersectLines`: the point or overlap of two line segments within the tolerance.
pub(super) fn intersect_lines(
    first: &Curve,
    second: &Curve,
    tolerance: f64,
) -> BRepFontLineIntersection {
    let first_direction = *first.end() - *first.start();
    let second_direction = *second.end() - *second.start();
    let first_length = first_direction.modulus();
    let second_length = second_direction.modulus();
    if first_length == 0.0 || second_length == 0.0 {
        return BRepFontLineIntersection::default();
    }
    let offset = *second.start() - *first.start();
    let cross = first_direction.crossed(second_direction);
    let cross_tolerance = tolerance * (first_length + second_length);
    if cross.abs() <= cross_tolerance {
        let coincident = offset.crossed(first_direction).abs() <= tolerance * first_length
            && (*second.end() - *first.start())
                .crossed(first_direction)
                .abs()
                <= tolerance * first_length;
        if coincident {
            return collinear_lines(first, second, tolerance / first_length);
        }
        if cross == 0.0 {
            return BRepFontLineIntersection::default();
        }
    }
    let first_parameter = offset.crossed(second_direction) / cross;
    let second_parameter = offset.crossed(first_direction) / cross;
    let (first_tol, second_tol) = (tolerance / first_length, tolerance / second_length);
    if first_parameter < -first_tol
        || first_parameter > 1.0 + first_tol
        || second_parameter < -second_tol
        || second_parameter > 1.0 + second_tol
    {
        return BRepFontLineIntersection::default();
    }
    let first_point = *first.start() + first_direction * first_parameter.clamp(0.0, 1.0);
    let second_point = *second.start() + second_direction * second_parameter.clamp(0.0, 1.0);
    BRepFontLineIntersection::point(0.5 * (first_point + second_point))
}

// Coincident supports: the overlap of the parameter ranges on the first line.
fn collinear_lines(
    first: &Curve,
    second: &Curve,
    first_param_tol: f64,
) -> BRepFontLineIntersection {
    let direction = *first.end() - *first.start();
    let square_length = direction.square_modulus();
    let mut first_parameter = (*second.start() - *first.start()).dot(direction) / square_length;
    let mut last_parameter = (*second.end() - *first.start()).dot(direction) / square_length;
    if last_parameter < first_parameter {
        std::mem::swap(&mut first_parameter, &mut last_parameter);
    }
    overlap_result(first, first_parameter, last_parameter, first_param_tol)
}

// The overlap of [min, max] with the line's [0, 1]: an overlap, nothing, or its midpoint.
fn overlap_result(line: &Curve, min: f64, max: f64, param_tol: f64) -> BRepFontLineIntersection {
    let overlap_first = 0.0f64.max(min);
    let overlap_last = 1.0f64.min(max);
    if overlap_last - overlap_first > param_tol {
        return BRepFontLineIntersection::overlap();
    }
    if overlap_last + param_tol < overlap_first {
        return BRepFontLineIntersection::default();
    }
    let direction = *line.end() - *line.start();
    let parameter = (0.5 * (overlap_first + overlap_last)).clamp(0.0, 1.0);
    BRepFontLineIntersection::point(*line.start() + direction * parameter)
}

/// The fixed frame of one line used by the line/quadratic intersection.
struct LineFrame<'a> {
    line: &'a Curve,
    direction: Xy,
    square_length: f64,
    param_tol: f64,
    cross_tolerance: f64,
    tolerance: f64,
}

impl LineFrame<'_> {
    fn cross_from_line(&self, point: Xy) -> f64 {
        (point - *self.line.start()).crossed(self.direction)
    }
    fn line_parameter(&self, point: Xy) -> f64 {
        (point - *self.line.start()).dot(self.direction) / self.square_length
    }
}

/// `intersectLineQuadratic`: the overlap, or up to two points, of a line and a quadratic.
pub(super) fn intersect_line_quadratic(
    line: &Curve,
    quadratic: &Curve,
    tolerance: f64,
) -> BRepFontLineIntersection {
    let direction = *line.end() - *line.start();
    let length = direction.modulus();
    if length == 0.0 {
        return BRepFontLineIntersection::default();
    }
    let frame = LineFrame {
        line,
        direction,
        square_length: direction.square_modulus(),
        param_tol: tolerance / length,
        cross_tolerance: tolerance * length,
        tolerance,
    };
    let crosses = [*quadratic.start(), *quadratic.control1(), *quadratic.end()]
        .map(|point| frame.cross_from_line(point));
    if crosses
        .iter()
        .all(|cross| cross.abs() <= frame.cross_tolerance)
    {
        return collinear_quadratic(&frame, quadratic);
    }
    transversal_quadratic(&frame, quadratic, crosses)
}

// The quadratic lies on the line support: overlap of its parameter image with [0, 1].
fn collinear_quadratic(frame: &LineFrame<'_>, quadratic: &Curve) -> BRepFontLineIntersection {
    let start = frame.line_parameter(*quadratic.start());
    let control = frame.line_parameter(*quadratic.control1());
    let end = frame.line_parameter(*quadratic.end());
    let mut min = start.min(end);
    let mut max = start.max(end);
    let denominator = start - 2.0 * control + end;
    if denominator != 0.0 {
        let extremum = (start - control) / denominator;
        if extremum > 0.0 && extremum < 1.0 {
            let u = 1.0 - extremum;
            let value =
                start * (u * u) + control * (2.0 * u * extremum) + end * (extremum * extremum);
            min = min.min(value);
            max = max.max(value);
        }
    }
    overlap_result(frame.line, min, max, frame.param_tol)
}

// The roots of the signed distance of the quadratic to the line, plus the endpoints and the
// distance extremum, filtered onto both segments and deduplicated.
fn transversal_quadratic(
    frame: &LineFrame<'_>,
    quadratic: &Curve,
    [start, control, end]: [f64; 3],
) -> BRepFontLineIntersection {
    let mut result = BRepFontLineIntersection::default();
    let ld = LongDouble::new;
    let two = ld(2.0);
    let a = ld(start) - two * ld(control) + ld(end);
    let b = two * (ld(control) - ld(start));
    let c = ld(start);
    let mut append =
        |parameter: LongDouble| append_parameter(frame, quadratic, parameter, &mut result);
    let zero = LongDouble::ZERO;
    if a == zero {
        if b != zero {
            append(-c / b);
        }
    } else {
        let discriminant = b * b - ld(4.0) * a * c;
        if discriminant == zero {
            append(-b / (two * a));
        } else if discriminant > zero {
            let q = ld(-0.5) * (b + discriminant.sqrt().copysign(b));
            append(q / a);
            append(c / q);
        }
    }
    append(zero);
    append(ld(1.0));
    if a != zero {
        append(-b / (two * a));
    }
    result
}

fn append_parameter(
    frame: &LineFrame<'_>,
    quadratic: &Curve,
    parameter: LongDouble,
    result: &mut BRepFontLineIntersection,
) {
    let parameter = parameter.narrow();
    if !is_valid_real(parameter) || !(0.0..=1.0).contains(&parameter) {
        return;
    }
    let curve_point = quadratic.value(parameter);
    if frame.cross_from_line(curve_point).abs() > frame.cross_tolerance {
        return;
    }
    let line_parameter = frame.line_parameter(curve_point);
    if line_parameter < -frame.param_tol || line_parameter > 1.0 + frame.param_tol {
        return;
    }
    let line_point = *frame.line.start() + frame.direction * line_parameter.clamp(0.0, 1.0);
    let intersection = 0.5 * (curve_point + line_point);
    let square_tolerance = frame.tolerance * frame.tolerance;
    let known = &result.points[..result.nb_points];
    if known
        .iter()
        .any(|point| (*point - intersection).square_modulus() <= square_tolerance)
    {
        return;
    }
    if result.nb_points < result.points.len() {
        result.points[result.nb_points] = intersection;
        result.nb_points += 1;
        result.kind = BRepFontLineIntersectionKind::Point;
    }
}
