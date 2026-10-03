use crate::shape_analysis::{ShapeAnalysisEdge, ShapeAnalysisSurface};
use crate::shape_build::ShapeBuildEdge;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{self, Edge, Face, ShapeIterator, Wire};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkmath::gp::Vector2;

pub(super) fn shift_wire_pcurves(
    face: &Face,
    wire: &Wire,
    shift: Vector2,
    recompute_surface: Option<&ShapeAnalysisSurface>,
) -> Result<(), Failure> {
    let mut iterator = ShapeIterator::new(&wire.clone().into_shape(), false, true);
    while iterator.more() {
        let edge = topods::edge(iterator.value()?)?;
        shift_edge_pcurve(face, &edge, shift, recompute_surface)?;
        iterator.next();
    }
    Ok(())
}

fn shift_edge_pcurve(
    face: &Face,
    edge: &Edge,
    shift: Vector2,
    recompute_surface: Option<&ShapeAnalysisSurface>,
) -> Result<(), Failure> {
    let (curve, first, _) = ShapeAnalysisEdge::new().pcurve_on_face(edge, face, true)?;
    if curve.is_null() {
        return Ok(());
    }
    curve.translate(shift)?;
    if let Some(surface) = recompute_surface {
        recompute_edge_curve(edge, &curve, first, surface)?;
    }
    Ok(())
}

fn recompute_edge_curve(
    edge: &Edge,
    curve: &Handle<dyn Geom2dCurve>,
    first: f64,
    surface: &ShapeAnalysisSurface,
) -> Result<(), Failure> {
    let builder = ShapeBuildEdge;
    builder.remove_curve_3d(edge)?;
    builder.build_curve_3d(edge);
    let vertex = ShapeAnalysisEdge::new().first_vertex(edge);
    let point = surface.value_uv(curve.value(first)?)?;
    BRepBuilder::default().update_vertex_point(&vertex, point, 0.0)
}
