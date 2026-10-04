//! Restricts the current finite-plane path to domains represented exactly by UV bounds.
use rustycad_tkbrep::{
    brep::BRepTool,
    topexp::Explorer,
    topods::{Face, Shape},
};
use rustycad_tkernel::{precision, standard::Failure};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point2;

pub(super) fn require_supported_face_domain(face: &Face) -> Result<(), Failure> {
    let wires = descendants(face, ShapeEnum::Wire)?;
    if wires.is_empty() {
        return Ok(());
    }
    if wires.len() != 1 || !is_rectangular_wire(face, &wires[0])? {
        return Err(not_implemented(
            "finite plane domain; expected one rectangular wire",
        ));
    }
    Ok(())
}

fn is_rectangular_wire(face: &Face, wire: &Shape) -> Result<bool, Failure> {
    let edges = descendants(wire, ShapeEnum::Edge)?;
    if edges.len() != 4 {
        return Ok(false);
    }
    let mut segments = Vec::with_capacity(4);
    for shape in edges {
        let edge = rustycad_tkbrep::topods::edge(&shape)?;
        let pcurve = BRepTool::curve_on_face(&edge, face)?;
        let line_handle = pcurve.pcurve.cast::<Geom2dLine>();
        let Some(line) = line_handle.get() else {
            return Ok(false);
        };
        segments.push((line.eval_d0(pcurve.first)?, line.eval_d0(pcurve.last)?));
    }
    Ok(is_axis_aligned_rectangle(&segments))
}

fn is_axis_aligned_rectangle(segments: &[(Point2, Point2)]) -> bool {
    let tolerance = precision::p_confusion();
    let points = unique_endpoints(segments, tolerance);
    let Some(bounds) = rectangle_bounds(&points, tolerance) else {
        return false;
    };
    points
        .iter()
        .all(|point| bounds.is_corner(*point, tolerance))
        && segments_are_rectangle_sides(segments, bounds, tolerance)
}

fn rectangle_bounds(points: &[Point2], tolerance: f64) -> Option<RectangleBounds> {
    if points.len() != 4 {
        return None;
    }
    let bounds = RectangleBounds::from_points(points);
    (bounds.u_max - bounds.u_min > tolerance && bounds.v_max - bounds.v_min > tolerance)
        .then_some(bounds)
}

fn segments_are_rectangle_sides(
    segments: &[(Point2, Point2)],
    bounds: RectangleBounds,
    tolerance: f64,
) -> bool {
    segments.iter().all(|(start, finish)| {
        let one_axis_constant =
            close(start.x(), finish.x(), tolerance) ^ close(start.y(), finish.y(), tolerance);
        one_axis_constant
            && bounds.is_corner(*start, tolerance)
            && bounds.is_corner(*finish, tolerance)
    })
}

fn unique_endpoints(segments: &[(Point2, Point2)], tolerance: f64) -> Vec<Point2> {
    let mut points = Vec::with_capacity(4);
    for point in segments
        .iter()
        .flat_map(|(first, second)| [*first, *second])
    {
        if points
            .iter()
            .all(|known| !same_point(*known, point, tolerance))
        {
            points.push(point);
        }
    }
    points
}

fn is_corner(point: Point2, u_min: f64, u_max: f64, v_min: f64, v_max: f64, tol: f64) -> bool {
    [u_min, u_max].iter().any(|u| {
        [v_min, v_max]
            .iter()
            .any(|v| close(point.x(), *u, tol) && close(point.y(), *v, tol))
    })
}

#[derive(Clone, Copy)]
struct RectangleBounds {
    u_min: f64,
    u_max: f64,
    v_min: f64,
    v_max: f64,
}

impl RectangleBounds {
    fn from_points(points: &[Point2]) -> Self {
        Self {
            u_min: points.iter().map(Point2::x).fold(f64::INFINITY, f64::min),
            u_max: points
                .iter()
                .map(Point2::x)
                .fold(f64::NEG_INFINITY, f64::max),
            v_min: points.iter().map(Point2::y).fold(f64::INFINITY, f64::min),
            v_max: points
                .iter()
                .map(Point2::y)
                .fold(f64::NEG_INFINITY, f64::max),
        }
    }

    fn is_corner(self, point: Point2, tolerance: f64) -> bool {
        is_corner(
            point, self.u_min, self.u_max, self.v_min, self.v_max, tolerance,
        )
    }
}

fn same_point(first: Point2, second: Point2, tolerance: f64) -> bool {
    close(first.x(), second.x(), tolerance) && close(first.y(), second.y(), tolerance)
}

fn close(first: f64, second: f64, tolerance: f64) -> bool {
    (first - second).abs() <= tolerance
}

fn descendants<T: Clone + Into<Shape>>(shape: &T, kind: ShapeEnum) -> Result<Vec<Shape>, Failure> {
    let root = shape.clone().into();
    let mut explorer = Explorer::new(&root, kind, ShapeEnum::Shape)?;
    let mut shapes = Vec::new();
    while explorer.more() {
        shapes.push(explorer.current().clone());
        explorer.next()?;
    }
    Ok(shapes)
}

fn not_implemented(branch: &str) -> Failure {
    Failure::with_message(
        "Standard_NotImplemented",
        &format!("BRepOffset_Inter3d::FaceInter does not support {branch}"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_a_triangle_boundary_as_not_a_parameter_box() {
        let points = [
            Point2::new(-2.0, -1.0),
            Point2::new(2.0, -1.0),
            Point2::new(0.0, 1.0),
        ];
        let segments = [
            (points[0], points[1]),
            (points[1], points[2]),
            (points[2], points[0]),
        ];
        assert!(!is_axis_aligned_rectangle(&segments));
    }
}
