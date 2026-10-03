use super::{
    Failure, FixRoot, GapCurve, GapPair, GeomBSplineCurve, GeomCurve, Handle, ShapeFixWire, Vertex,
};
use crate::{
    shape_analysis::ShapeAnalysisTransferParametersProj, shape_build::ShapeBuildEdge,
    shape_fix::ShapeFixShapeTolerance,
};
use rustycad_tkbrep::{
    brep::BRepBuilder,
    brep_tools::ReShape,
    topods::{self, Edge, ShapeIterator},
};
use rustycad_tkernel::precision;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
pub(super) fn replace_gap_curves(
    tool: &ShapeFixWire,
    pair: &GapPair,
    first: &Handle<GeomBSplineCurve>,
    second: &Handle<GeomBSplineCurve>,
) -> Result<(), Failure> {
    let last = copy_gap_vertex(tool, &pair.vertices.1)?;
    let before = if pair.vertices.0.is_same(&pair.vertices.1) {
        topods::vertex(&last.oriented(Orientation::Reversed))?
    } else {
        copy_gap_vertex(tool, &pair.vertices.0)?
    };
    if !first.is_null() {
        replace_gap_curve(tool, &pair.first, first, &Vertex::default(), &before)?;
    }
    if !second.is_null() {
        replace_gap_curve(tool, &pair.second, second, &last, &Vertex::default())?;
    }
    Ok(())
}
fn copy_gap_vertex(tool: &ShapeFixWire, vertex: &Vertex) -> Result<Vertex, Failure> {
    let copy = topods::vertex(&vertex.empty_copied()?)?;
    ShapeFixShapeTolerance.set_tolerance(&copy, precision::confusion(), ShapeEnum::Shape)?;
    ReShape::replace(&*tool.context(), vertex, &copy)?;
    Ok(copy)
}
fn replace_gap_curve(
    tool: &ShapeFixWire,
    entry: &GapCurve,
    curve: &Handle<GeomBSplineCurve>,
    first: &Vertex,
    last: &Vertex,
) -> Result<(), Failure> {
    let mut replacement = ShapeBuildEdge.copy_replace_vertices(&entry.edge, first, last)?;
    let forward = topods::edge(&replacement.oriented(Orientation::Forward))?;
    let builder = BRepBuilder::default();
    builder.update_edge_curve(&forward, &curve.cast::<dyn GeomCurve>(), 0.)?;
    ShapeBuildEdge.set_range_3d(&forward, curve.first_parameter(), curve.last_parameter())?;
    ShapeFixShapeTolerance.set_tolerance(&replacement, precision::confusion(), ShapeEnum::Edge)?;
    builder.same_range(&replacement, false)?;
    preserve_gap_nonmanifold(tool, &entry.edge, &mut replacement)?;
    ReShape::replace(&*tool.context(), &entry.edge, &replacement)?;
    tool.wire_data()?.set(&replacement, entry.index)
}
fn preserve_gap_nonmanifold(
    tool: &ShapeFixWire,
    source: &Edge,
    target: &mut Edge,
) -> Result<(), Failure> {
    for shape in ShapeIterator::new(source, false, true) {
        if !matches!(
            shape.orientation(),
            Orientation::Internal | Orientation::External
        ) {
            continue;
        }
        let vertex = topods::vertex(&shape)?;
        let copy =
            ShapeAnalysisTransferParametersProj::copy_nm_vertex_edge(&vertex, target, source)?;
        BRepBuilder::default().add(target, &copy)?;
        ReShape::replace(&*tool.context(), &vertex, &copy)?;
    }
    Ok(())
}
