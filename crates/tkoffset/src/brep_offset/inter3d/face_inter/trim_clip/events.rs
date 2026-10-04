//! Parameter events where a plane-intersection line crosses straight wire edges.
use rustycad_tkbo::int_tools::Curve;
use rustycad_tkbrep::{
    brep::BRepTool,
    topexp::Explorer,
    topods::{self, Face, Shape},
};
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine, Geom2dTrimmedCurve};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Point2, Vector2};

pub(super) fn boundary_events(
    curve: &Curve,
    face: &Face,
    surface_index: usize,
    range: (f64, f64),
) -> Result<Vec<f64>, Failure> {
    let section = line_basis(select_pcurve(curve, surface_index)?)?;
    let face_shape: Shape = face.clone().into();
    let mut explorer = Explorer::new(&face_shape, ShapeEnum::Edge, ShapeEnum::Shape)?;
    collect_boundary_events(&mut explorer, face, section, range)
}

fn collect_boundary_events(
    explorer: &mut Explorer,
    face: &Face,
    section: Line2Data,
    range: (f64, f64),
) -> Result<Vec<f64>, Failure> {
    let mut parameters = Vec::new();
    while explorer.more() {
        let parameter = edge_event(explorer.current(), face, section)?;
        append_if_in_range(&mut parameters, parameter, range);
        explorer.next()?;
    }
    Ok(parameters)
}

fn edge_event(shape: &Shape, face: &Face, section: Line2Data) -> Result<Option<f64>, Failure> {
    let edge = topods::edge(shape)?;
    let pcurve = BRepTool::curve_on_face(&edge, face)?;
    let segment = boundary_segment(&pcurve.pcurve, pcurve.first, pcurve.last)?;
    line_segment_intersection(section, segment)
}

fn append_if_in_range(parameters: &mut Vec<f64>, parameter: Option<f64>, range: (f64, f64)) {
    let Some(parameter) = parameter else {
        return;
    };
    let tolerance = precision::p_confusion();
    if parameter >= range.0 - tolerance && parameter <= range.1 + tolerance {
        parameters.push(parameter.clamp(range.0, range.1));
    }
}

fn select_pcurve(curve: &Curve, index: usize) -> Result<&Handle<dyn Geom2dCurve>, Failure> {
    let pcurve = if index == 0 {
        curve.first_curve2d()
    } else {
        curve.second_curve2d()
    };
    if pcurve.is_null() {
        return Err(not_implemented("section pcurve missing on input face"));
    }
    Ok(pcurve)
}

fn line_basis(curve: &Handle<dyn Geom2dCurve>) -> Result<Line2Data, Failure> {
    let line = curve.cast::<Geom2dLine>();
    if let Some(line) = line.get() {
        return Ok(Line2Data {
            origin: line.location(),
            direction: Vector2::from(line.direction()),
        });
    }
    let trimmed = curve.cast::<Geom2dTrimmedCurve>();
    if let Some(trimmed) = trimmed.get() {
        return line_basis(&trimmed.basis_curve());
    }
    Err(not_implemented("non-line pcurve intersection"))
}

fn boundary_segment(
    curve: &Handle<dyn Geom2dCurve>,
    first: f64,
    last: f64,
) -> Result<(Point2, Point2), Failure> {
    if line_basis(curve).is_err() {
        return Err(not_implemented("curved boundary pcurve"));
    }
    let curve = curve
        .get()
        .ok_or_else(|| not_implemented("null boundary pcurve"))?;
    Ok((curve.eval_d0(first)?, curve.eval_d0(last)?))
}

fn line_segment_intersection(
    line: Line2Data,
    (first, last): (Point2, Point2),
) -> Result<Option<f64>, Failure> {
    let segment = Vector2::new(last.x() - first.x(), last.y() - first.y());
    if segment.magnitude() <= precision::p_confusion() {
        return Ok(None);
    }
    let offset = Vector2::new(first.x() - line.origin.x(), first.y() - line.origin.y());
    let determinant = cross(line.direction, segment);
    if is_parallel(determinant, segment) {
        return parallel_event(offset, line);
    }
    let (parameter, edge_parameter) = solve_line_parameters(offset, line, segment, determinant);
    if !on_segment(edge_parameter) {
        return Ok(None);
    }
    Ok(Some(parameter))
}

fn is_parallel(determinant: f64, segment: Vector2) -> bool {
    determinant.abs() <= precision::angular() * segment.magnitude()
}

fn parallel_event(offset: Vector2, line: Line2Data) -> Result<Option<f64>, Failure> {
    if cross(offset, line.direction).abs() > precision::confusion() {
        return Ok(None);
    }
    Err(not_implemented(
        "section curve coincident with a boundary edge",
    ))
}

fn solve_line_parameters(
    offset: Vector2,
    line: Line2Data,
    segment: Vector2,
    determinant: f64,
) -> (f64, f64) {
    (
        cross(offset, segment) / determinant,
        cross(offset, line.direction) / determinant,
    )
}

fn on_segment(parameter: f64) -> bool {
    let tolerance = precision::p_confusion();
    parameter >= -tolerance && parameter <= 1.0 + tolerance
}

fn cross(first: Vector2, second: Vector2) -> f64 {
    first.x() * second.y() - first.y() * second.x()
}

fn not_implemented(branch: &str) -> Failure {
    Failure::with_message(
        "Standard_NotImplemented",
        &format!("BRepOffset_Inter3d::FaceInter does not support {branch}"),
    )
}

#[derive(Clone, Copy)]
struct Line2Data {
    origin: Point2,
    direction: Vector2,
}
