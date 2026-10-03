use super::coincident;
use crate::shape_analysis::ShapeAnalysisEdge;
use crate::shape_extend::ShapeExtendCompositeSurface;
use crate::shape_fix::ShapeFixWireSegment;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::toploc::Location;

/// `IsShortSegment`: 1 short in UV, -1 short only in 3D, 0 otherwise.
pub(super) fn classify_short_segment(
    segment: &ShapeFixWireSegment,
    face: &Face,
    grid: &Handle<ShapeExtendCompositeSurface>,
    location: &Location,
    resolution: (f64, f64),
) -> Result<i32, Failure> {
    let first = segment.first_vertex()?;
    if !first.is_same(&segment.last_vertex()?.into_shape()) {
        return Ok(0);
    }
    classify_closed_segment(segment, face, grid, location, resolution, &first)
}

fn classify_closed_segment(
    segment: &ShapeFixWireSegment,
    face: &Face,
    grid: &Handle<ShapeExtendCompositeSurface>,
    location: &Location,
    resolution: (f64, f64),
    vertex: &rustycad_tkbrep::topods::Vertex,
) -> Result<i32, Failure> {
    let point = BRepTool::pnt(vertex)?;
    let tolerance = BRepTool::vertex_tolerance(vertex)?;
    let mut classification = 1;
    for edge_index in 1..=segment.nb_edges() {
        let edge = segment.edge(edge_index)?;
        if !vertex.is_same(&ShapeAnalysisEdge::new().last_vertex(&edge).into_shape()) {
            return Ok(0);
        }
        let (curve, first, last) = ShapeAnalysisEdge::new().pcurve_on_face(&edge, face, false)?;
        if curve.is_null() {
            continue;
        }
        classification = check_edge_shortness(
            grid,
            location,
            (resolution.0, resolution.1),
            tolerance,
            point,
            &curve,
            (first, last),
            classification,
        )?;
        if classification == 0 {
            return Ok(0);
        }
    }
    Ok(classification)
}

#[allow(clippy::too_many_arguments)]
fn check_edge_shortness(
    grid: &Handle<ShapeExtendCompositeSurface>,
    location: &Location,
    resolution: (f64, f64),
    tolerance: f64,
    point: rustycad_tkmath::gp::Point3,
    curve: &Handle<dyn rustycad_tkg2d::geom2d::Geom2dCurve>,
    range: (f64, f64),
    classification: i32,
) -> Result<i32, Failure> {
    let live_curve = curve.get().ok_or_else(null_curve)?;
    let end = live_curve.d0(range.1)?;
    let midpoint = live_curve.d0((range.0 + range.1) / 2.0)?;
    let classification = if coincident(end, midpoint, resolution, tolerance) {
        classification
    } else {
        -1
    };
    let mut midpoint_3d = grid.get().ok_or_else(null_grid)?.value_at(midpoint)?;
    if !location.is_identity() {
        midpoint_3d.transform(location.transformation());
    }
    if midpoint_3d.square_distance(&point) > tolerance * tolerance {
        return Ok(0);
    }
    Ok(classification)
}

fn null_curve() -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        "ShapeFix_ComposeShell::IsShortSegment: null p-curve",
    )
}

fn null_grid() -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        "ShapeFix_ComposeShell::IsShortSegment: null grid",
    )
}
