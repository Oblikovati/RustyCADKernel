use rustycad_tkmath::gp::{Line2, Point2};

const INTERSECTION_TOLERANCE: f64 = 1.0e-10;

pub(super) fn point_line_deviation(point: Point2, line: Line2) -> f64 {
    let direction = line.direction();
    let dx = point.x() - line.location().x();
    let dy = point.y() - line.location().y();
    -direction.y() * dx + direction.x() * dy
}

pub(super) fn point_line_position(point: Point2, line: Line2) -> (i32, f64) {
    let deviation = point_line_deviation(point, line);
    let position = if deviation > INTERSECTION_TOLERANCE {
        1
    } else if deviation < -INTERSECTION_TOLERANCE {
        2
    } else {
        0
    };
    (position, deviation)
}

pub(super) fn parameter_on_line(point: Point2, line: Line2) -> f64 {
    let direction = line.direction();
    (point.x() - line.location().x()) * direction.x()
        + (point.y() - line.location().y()) * direction.y()
}

pub(super) fn parameter_between_points(first: Point2, last: Point2, line: Line2) -> f64 {
    let first_deviation = point_line_deviation(first, line);
    let last_deviation = point_line_deviation(last, line);
    interpolate_line_parameter(first, last, line, first_deviation, last_deviation)
}

fn interpolate_line_parameter(
    first: Point2,
    last: Point2,
    line: Line2,
    first_deviation: f64,
    last_deviation: f64,
) -> f64 {
    let first_parameter = parameter_on_line(first, line);
    let last_parameter = parameter_on_line(last, line);
    if first_deviation.abs() < rustycad_tkernel::precision::p_confusion() {
        return choose_near_line_parameter(first_parameter, last_parameter, last_deviation);
    }
    if last_deviation.abs() < rustycad_tkernel::precision::p_confusion() {
        return last_parameter;
    }
    interpolate_opposite_sides(
        first_parameter,
        last_parameter,
        first_deviation,
        last_deviation,
    )
}

fn choose_near_line_parameter(first: f64, last: f64, other_deviation: f64) -> f64 {
    if other_deviation.abs() < rustycad_tkernel::precision::p_confusion() {
        (first + last) * 0.5
    } else {
        first
    }
}

fn interpolate_opposite_sides(
    first: f64,
    last: f64,
    first_deviation: f64,
    last_deviation: f64,
) -> f64 {
    if last_deviation * first_deviation > 0.0 {
        return (first + last) * 0.5;
    }
    (first * last_deviation - last * first_deviation) / (last_deviation - first_deviation)
}

pub(super) fn project_point_on_line(point: Point2, line: Line2) -> Point2 {
    let parameter = parameter_on_line(point, line);
    let direction = line.direction();
    line.location()
        .translated(rustycad_tkmath::gp::Vector2::new(
            direction.x() * parameter,
            direction.y() * parameter,
        ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkmath::gp::Direction2;

    #[test]
    fn signed_side_and_projected_intersection_match_native_line_arithmetic() {
        let line = Line2::new(Point2::default(), Direction2::new(0.0, 1.0).unwrap());
        assert_eq!(point_line_position(Point2::new(2.0, 0.0), line), (2, -2.0));
        assert_eq!(point_line_position(Point2::new(-2.0, 0.0), line), (1, 2.0));
        assert_eq!(
            parameter_between_points(Point2::new(2.0, -1.0), Point2::new(-2.0, 1.0), line),
            0.0
        );
        assert_eq!(
            project_point_on_line(Point2::new(2.0, 0.0), line),
            Point2::new(0.0, 0.0)
        );
    }
}
