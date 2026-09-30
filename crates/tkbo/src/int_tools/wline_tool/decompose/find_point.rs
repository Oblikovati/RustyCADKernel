use super::*;
pub(super) fn find_point(
    first: Point2,
    last: Point2,
    bounds: SurfaceBounds,
) -> Result<Option<Point2>, Failure> {
    let vector = Vector2::between(first, last);
    for index in 0..4 {
        let mut normal = vector.normalized()?;
        refine_vector(&mut normal);
        if let Some(point) = boundary_candidate(vector, normal, last, bounds, index)? {
            return Ok(Some(point));
        }
    }
    Ok(None)
}
fn boundary_candidate(
    vector: Vector2,
    normal: Vector2,
    last: Point2,
    bounds: SurfaceBounds,
    index: usize,
) -> Result<Option<Point2>, Failure> {
    let vertical = index.is_multiple_of(2);
    let dot = if vertical { normal.x() } else { normal.y() };
    if dot.abs() < precision::angular() {
        return Ok(None);
    }
    let boundary = [bounds.0, bounds.2, bounds.1, bounds.3][index];
    let value = if vertical { last.x() } else { last.y() };
    let outside = is_outside(value, boundary, index);
    let tangent = if vertical { normal.y() } else { normal.x() };
    let offset = (value - boundary).abs() * tangent / dot;
    for sign in [1., -1.] {
        let point = boundary_point(last, boundary, offset * sign, vertical);
        if accept_candidate(vector, last, point, bounds, vertical, outside)? {
            return Ok(Some(point));
        }
    }
    Ok(None)
}
fn accept_candidate(
    vector: Vector2,
    last: Point2,
    point: Point2,
    bounds: SurfaceBounds,
    vertical: bool,
    outside: bool,
) -> Result<bool, Failure> {
    let mut candidate = Vector2::between(last, point);
    if outside {
        candidate.reverse();
    }
    let dot = vector.dot(candidate);
    let angle = vector.angle(candidate)?;
    if !(dot > 0. && angle.abs() < precision::p_confusion()) {
        return Ok(false);
    }
    Ok(if vertical {
        point.y() >= bounds.2 && point.y() <= bounds.3
    } else {
        point.x() >= bounds.0 && point.x() <= bounds.1
    })
}
fn refine_vector(vector: &mut Vector2) {
    let mut coords = [vector.x(), vector.y()];
    for index in 0..2 {
        let magnitude = coords[index].abs();
        if magnitude > 1. - f64::EPSILON && magnitude < 1. + f64::EPSILON {
            coords[index] = if coords[index] < 0. { -1. } else { 1. };
            coords[(index + 1) % 2] = 0.;
            break;
        }
    }
    *vector = Vector2::new(coords[0], coords[1]);
}

fn is_outside(value: f64, boundary: f64, index: usize) -> bool {
    if index < 2 {
        return value < boundary;
    }
    value > boundary
}
fn boundary_point(last: Point2, boundary: f64, offset: f64, vertical: bool) -> Point2 {
    if vertical {
        return Point2::new(boundary, last.y() + offset);
    }
    Point2::new(last.x() + offset, boundary)
}
