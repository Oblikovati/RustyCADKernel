use super::super::wire_gap_topology::{copy_gap_vertex, preserve_gap_nonmanifold};
use super::{
    Face, Failure, FixRoot, Geom2dBSplineCurve, Handle, PcurveGap, ShapeAnalysisEdge, ShapeFixWire,
};
use crate::{shape_build::ShapeBuildEdge, shape_fix::ShapeFixShapeTolerance};
use rustycad_tkbrep::{
    brep::BRepBuilder,
    brep_tools::ReShape,
    topods::{self, Vertex},
};
use rustycad_tkernel::precision;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
pub(super) fn replace_pcurves(
    tool: &ShapeFixWire,
    face: &Face,
    entries: (&PcurveGap, &PcurveGap),
    curves: (&Handle<Geom2dBSplineCurve>, &Handle<Geom2dBSplineCurve>),
) -> Result<(), Failure> {
    let analysis = ShapeAnalysisEdge::new();
    let first = analysis.last_vertex(&entries.0.edge);
    let last = analysis.first_vertex(&entries.1.edge);
    let after = copy_gap_vertex(tool, &last)?;
    let before = if first.is_same(&last) {
        topods::vertex(&after.oriented(Orientation::Reversed))?
    } else {
        copy_gap_vertex(tool, &first)?
    };
    replace_pcurve(
        tool,
        face,
        entries.0,
        curves.0,
        (&Vertex::default(), &before),
    )?;
    replace_pcurve(
        tool,
        face,
        entries.1,
        curves.1,
        (&after, &Vertex::default()),
    )
}
fn replace_pcurve(
    tool: &ShapeFixWire,
    face: &Face,
    entry: &PcurveGap,
    curve: &Handle<Geom2dBSplineCurve>,
    vertices: (&Vertex, &Vertex),
) -> Result<(), Failure> {
    if curve.is_null() {
        return Ok(());
    }
    let mut replacement =
        ShapeBuildEdge.copy_replace_vertices(&entry.edge, vertices.0, vertices.1)?;
    let forward = topods::edge(&replacement.oriented(Orientation::Forward))?;
    let builder = BRepBuilder::default();
    builder.update_edge_pcurve_on_face(&forward, &curve.cast(), face, 0.)?;
    builder.range_on_face(
        &forward,
        face,
        curve.first_parameter(),
        curve.last_parameter(),
    )?;
    ShapeFixShapeTolerance.set_tolerance(&replacement, precision::confusion(), ShapeEnum::Edge)?;
    builder.same_range(&replacement, false)?;
    preserve_gap_nonmanifold(tool, &entry.edge, &mut replacement)?;
    ReShape::replace(&*tool.context(), &entry.edge, &replacement)?;
    tool.wire_data()?.set(&replacement, entry.index)
}
