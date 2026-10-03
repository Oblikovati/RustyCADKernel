use super::super::ShapeFixComposeShell;
use crate::shape_analysis::{ShapeAnalysisCurve, ShapeAnalysisSurface};
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use crate::shape_fix::FixRoot;
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::{self, Edge, Shape, ShapeIterator, Vertex},
};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeombase::extrema_pc2d::{Curve as ExtremaCurve, ExtremaResult, SearchMode};
use rustycad_tkmath::gp::Point3;

pub(super) fn make_cut_vertex(point: Point3, tolerance: f64) -> Result<Vertex, Failure> {
    let mut vertex = Vertex::default();
    BRepBuilder::default().make_vertex_at(&mut vertex, point, tolerance)?;
    Ok(vertex)
}

pub(super) fn mark_fail2(composer: &ShapeFixComposeShell) {
    let mut fields = composer.fields.write();
    fields.status |= ShapeExtend::encode_status(ShapeExtendStatus::Fail2);
}

pub(super) fn vertex_tolerances(
    composer: &ShapeFixComposeShell,
    first: &Vertex,
    last: &Vertex,
) -> Result<(f64, f64), Failure> {
    let first = composer.limit_tolerance(BRepTool::vertex_tolerance(first)?);
    let last = composer.limit_tolerance(BRepTool::vertex_tolerance(last)?);
    Ok((first, last))
}

pub(super) fn non_manifold_vertices(edge: &Edge) -> Vec<Shape> {
    let mut children = ShapeIterator::new(&edge.clone().into_shape(), false, true);
    let mut vertices = Vec::new();
    while children.more() {
        if let Ok(child) = children.value()
            && !matches!(
                child.orientation(),
                Orientation::Forward | Orientation::Reversed
            )
        {
            vertices.push(child.clone());
        }
        children.next();
    }
    vertices
}

pub(super) fn non_manifold_parameters(
    vertices: &[Shape],
    curve_3d: &Handle<dyn rustycad_tkg3d::geom::GeomCurve>,
    pcurve: &Handle<dyn rustycad_tkg2d::geom2d::Geom2dCurve>,
    face: &rustycad_tkbrep::topods::Face,
    range_3d: (f64, f64),
    range_2d: (f64, f64),
) -> Result<Vec<f64>, Failure> {
    vertices
        .iter()
        .map(|vertex| {
            projected_vertex_parameter(vertex, curve_3d, pcurve, face, range_3d, range_2d)
        })
        .collect()
}

fn projected_vertex_parameter(
    vertex: &Shape,
    curve_3d: &Handle<dyn rustycad_tkg3d::geom::GeomCurve>,
    pcurve: &Handle<dyn rustycad_tkg2d::geom2d::Geom2dCurve>,
    face: &rustycad_tkbrep::topods::Face,
    range_3d: (f64, f64),
    range_2d: (f64, f64),
) -> Result<f64, Failure> {
    let point = BRepTool::pnt(&topods::vertex(vertex)?)?;
    if !curve_3d.is_null() {
        return project_on_curve_3d(curve_3d, point, range_3d);
    }
    project_on_pcurve(face, pcurve, point, range_2d)
}

fn project_on_curve_3d(
    curve: &Handle<dyn rustycad_tkg3d::geom::GeomCurve>,
    point: Point3,
    range: (f64, f64),
) -> Result<f64, Failure> {
    let projection =
        ShapeAnalysisCurve.project(curve, point, precision::confusion(), range, false)?;
    Ok(projection.2)
}

fn project_on_pcurve(
    face: &rustycad_tkbrep::topods::Face,
    pcurve: &Handle<dyn rustycad_tkg2d::geom2d::Geom2dCurve>,
    point: Point3,
    range: (f64, f64),
) -> Result<f64, Failure> {
    let surface = ShapeAnalysisSurface::new(&BRepTool::surface(face)?)?;
    let uv = surface.value_of_uv(point, precision::confusion())?;
    let mut extrema = ExtremaCurve::from_geom_range(pcurve, range.0, range.1)?;
    let computed = extrema.perform(uv, precision::confusion(), SearchMode::MinMax)?;
    let index = computed.min_index();
    if index == ExtremaResult::INVALID_INDEX {
        return Ok(range.0);
    }
    Ok(computed.extrema[index].parameter)
}
