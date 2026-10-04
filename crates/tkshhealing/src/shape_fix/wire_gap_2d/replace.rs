use super::super::wire_gap_topology::{copy_gap_vertex, preserve_gap_nonmanifold};
use super::{Face, Failure, FixRoot, Handle, PcurveGap, ShapeAnalysisEdge, ShapeFixWire};
use crate::{shape_build::ShapeBuildEdge, shape_fix::ShapeFixShapeTolerance};
use rustycad_tkbrep::{
    brep::BRepBuilder,
    brep_tools::ReShape,
    topods::{self, Vertex},
};
use rustycad_tkernel::precision;
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
pub(super) struct CurveRepair {
    pub(super) curve: Handle<dyn Geom2dCurve>,
    pub(super) first: f64,
    pub(super) last: f64,
}
impl CurveRepair {
    pub(super) fn from_spline(curve: &Handle<Geom2dBSplineCurve>) -> Option<Self> {
        let spline = curve.get()?;
        Some(Self {
            curve: curve.cast(),
            first: spline.first_parameter(),
            last: spline.last_parameter(),
        })
    }
}
pub(super) fn replace_pcurves(
    tool: &ShapeFixWire,
    face: &Face,
    entries: (&PcurveGap, &PcurveGap),
    curves: (&Handle<Geom2dBSplineCurve>, &Handle<Geom2dBSplineCurve>),
) -> Result<(), Failure> {
    let first = CurveRepair::from_spline(curves.0);
    let second = CurveRepair::from_spline(curves.1);
    replace_pcurves_with_ranges(tool, face, entries, (&first, &second))
}
pub(super) fn replace_pcurves_with_ranges(
    tool: &ShapeFixWire,
    face: &Face,
    entries: (&PcurveGap, &PcurveGap),
    curves: (&Option<CurveRepair>, &Option<CurveRepair>),
) -> Result<(), Failure> {
    let (before, after) = neighboring_vertices(tool, entries)?;
    let first_vertices = (&Vertex::default(), &before);
    let second_vertices = (&after, &Vertex::default());
    replace_entry(tool, face, entries.0, curves.0, first_vertices)?;
    replace_entry(tool, face, entries.1, curves.1, second_vertices)
}

fn neighboring_vertices(
    tool: &ShapeFixWire,
    entries: (&PcurveGap, &PcurveGap),
) -> Result<(Vertex, Vertex), Failure> {
    let analysis = ShapeAnalysisEdge::new();
    let first = analysis.last_vertex(&entries.0.edge);
    let last = analysis.first_vertex(&entries.1.edge);
    let after = copy_gap_vertex(tool, &last)?;
    let before = clone_other_end(tool, &first, &last, &after)?;
    Ok((before, after))
}

fn replace_entry(
    tool: &ShapeFixWire,
    face: &Face,
    entry: &PcurveGap,
    curve: &Option<CurveRepair>,
    vertices: (&Vertex, &Vertex),
) -> Result<(), Failure> {
    replace_pcurve(tool, face, entry, curve, vertices)
}

pub(super) fn replace_single_pcurve_range(
    tool: &ShapeFixWire,
    face: &Face,
    entry: &PcurveGap,
    curve: &Handle<dyn Geom2dCurve>,
    range: (f64, f64),
) -> Result<(), Failure> {
    let (before, _) = neighboring_vertices(tool, (entry, entry))?;
    replace_single_curve(tool, face, entry, curve, range, &before)
}

fn replace_single_curve(
    tool: &ShapeFixWire,
    face: &Face,
    entry: &PcurveGap,
    curve: &Handle<dyn Geom2dCurve>,
    range: (f64, f64),
    before: &Vertex,
) -> Result<(), Failure> {
    replace_pcurve(
        tool,
        face,
        entry,
        &Some(CurveRepair {
            curve: curve.clone(),
            first: range.0,
            last: range.1,
        }),
        (&Vertex::default(), before),
    )
}

fn clone_other_end(
    tool: &ShapeFixWire,
    first: &Vertex,
    last: &Vertex,
    after: &Vertex,
) -> Result<Vertex, Failure> {
    if first.is_same(last) {
        return topods::vertex(&after.oriented(Orientation::Reversed));
    }
    copy_gap_vertex(tool, first)
}
fn replace_pcurve(
    tool: &ShapeFixWire,
    face: &Face,
    entry: &PcurveGap,
    curve: &Option<CurveRepair>,
    vertices: (&Vertex, &Vertex),
) -> Result<(), Failure> {
    let Some(curve) = curve else {
        return Ok(());
    };
    let mut replacement =
        ShapeBuildEdge.copy_replace_vertices(&entry.edge, vertices.0, vertices.1)?;
    update_replacement_pcurve(&mut replacement, face, curve)?;
    preserve_gap_nonmanifold(tool, &entry.edge, &mut replacement)?;
    ReShape::replace(&*tool.context(), &entry.edge, &replacement)?;
    tool.wire_data()?.set(&replacement, entry.index)
}

fn update_replacement_pcurve(
    replacement: &mut rustycad_tkbrep::topods::Edge,
    face: &Face,
    curve: &CurveRepair,
) -> Result<(), Failure> {
    let forward = topods::edge(&replacement.oriented(Orientation::Forward))?;
    let builder = BRepBuilder::default();
    builder.update_edge_pcurve_on_face(&forward, &curve.curve, face, 0.)?;
    builder.range_on_face(&forward, face, curve.first, curve.last)?;
    ShapeFixShapeTolerance.set_tolerance(replacement, precision::confusion(), ShapeEnum::Edge)?;
    builder.same_range(replacement, false)
}
