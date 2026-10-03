use crate::shape_analysis::{ShapeAnalysisCurve, ShapeAnalysisEdge, ShapeAnalysisSurface};
use crate::shape_fix::ShapeFixWireSegment;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::gp::Point2;

pub(super) fn wire_middle_points(
    wires: &Sequence<ShapeFixWireSegment>,
    face: &Face,
) -> Result<Vec<Point2>, Failure> {
    (1..=wires.length() as usize)
        .map(|index| wire_middle_point(wires.value(index)?, face))
        .collect()
}

pub(super) fn wire_middle_point(
    wire: &ShapeFixWireSegment,
    face: &Face,
) -> Result<Point2, Failure> {
    if wire.is_vertex() {
        return middle_point_for_vertex(wire, face);
    }
    middle_point_for_wire(wire, face)
}

fn middle_point_for_vertex(wire: &ShapeFixWireSegment, face: &Face) -> Result<Point2, Failure> {
    let point = BRepTool::pnt(&wire.vertex())?;
    let surface = BRepTool::surface(face)?;
    ShapeAnalysisSurface::new(&surface)?
        .value_of_uv(point, rustycad_tkernel::precision::confusion())
}

fn middle_point_for_wire(wire: &ShapeFixWireSegment, face: &Face) -> Result<Point2, Failure> {
    let data = wire.wire_data().get().ok_or_else(null_wire)?;
    let mut bounds = BoundingBox2::default();
    for index in 1..=data.nb_edges() {
        add_edge_bounds(&mut bounds, &data.edge(index)?, face)?;
    }
    if bounds.is_void() {
        return Ok(Point2::default());
    }
    Ok(bounds.center().unwrap_or_default())
}

fn add_edge_bounds(
    bounds: &mut BoundingBox2,
    edge: &rustycad_tkbrep::topods::Edge,
    face: &Face,
) -> Result<(), Failure> {
    let (curve, first, last) = ShapeAnalysisEdge::new().pcurve_on_face(edge, face, false)?;
    ShapeAnalysisCurve.fill_bnd_box(&curve, first, last, 3, false, bounds)
}

fn null_wire() -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        "ShapeFix_ComposeShell::GetMiddlePoint: null segment wire; expected wire data",
    )
}
