//! Endpoint copies protect the original topology; non-manifold points retain native projection.
use super::*;
use super::{perform::WireSupport, segments::DividedCurves};
use crate::shape_analysis::ShapeAnalysisCurve;
use rustycad_tkbrep::{
    topexp,
    topods::{self, ShapeIterator, Vertex},
};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::{
    adaptor3d::Adaptor3dCurveOnSurface, geom::GeomCurve, geom_adaptor::GeomAdaptorSurface,
    top_abs::Orientation,
};
use rustycad_tkmath::gp::Point3;
pub(super) struct EdgeVertices {
    pub first: Vertex,
    pub last: Vertex,
    pub first_point: Point3,
    pub non_manifold: Vec<(Vertex, f64)>,
    pub tolerance: f64,
    pub degenerate: bool,
    pub forward: bool,
}
impl EdgeVertices {
    pub fn prepare<T: WireDivider + ?Sized>(
        tool: &T,
        support: &WireSupport,
        edge: &Edge,
        curves: &DividedCurves,
    ) -> Result<Self, Failure> {
        let original_first = topexp::first_vertex(edge, false);
        let original_last = topexp::last_vertex(edge, false);
        let tolerance = BRepTool::edge_tolerance(edge)?;
        let degenerate = BRepTool::degenerated(edge)?;
        let first = topods::vertex(&original_first.empty_copied()?)?;
        support.context.replace(&original_first, &first)?;
        let last = copy_last(&original_first, &original_last, &first, &support.context)?;
        let non_manifold = collect_non_manifold(tool, support, edge, curves)?;
        let first_point = Point3::default();
        Ok(Self {
            first,
            last,
            first_point,
            non_manifold,
            tolerance,
            degenerate,
            forward: edge.orientation() == Orientation::Forward,
        })
    }
}
fn copy_last(
    original_first: &Vertex,
    original_last: &Vertex,
    first: &Vertex,
    context: &Handle<ShapeBuildReShape>,
) -> Result<Vertex, Failure> {
    if original_first.is_same(original_last) {
        return topods::vertex(&first.oriented(original_last.orientation()));
    }
    let last = topods::vertex(&original_last.empty_copied()?)?;
    context.replace(original_last, &last)?;
    Ok(last)
}
struct VertexProjection {
    curve: Handle<dyn GeomCurve>,
    adaptor: Adaptor3dCurveOnSurface,
    bounds: (f64, f64),
}
fn vertex_projection(
    support: &WireSupport,
    edge: &Edge,
    curves: &DividedCurves,
) -> Result<VertexProjection, Failure> {
    let mut projection = VertexProjection {
        curve: Handle::default(),
        adaptor: Adaptor3dCurveOnSurface::default(),
        bounds: (0., 0.),
    };
    if curves.has3 {
        let (curve, first, last) = ShapeAnalysisEdge::new().curve_3d(edge, false)?;
        projection.curve = curve;
        projection.bounds = (first, last);
        return Ok(projection);
    }
    if curves.has2 && !support.surface.is_null() {
        let (curve, first, last) =
            ShapeAnalysisEdge::new().pcurve_on_face(edge, &support.face, false)?;
        let planar = Handle::new(Geom2dAdaptorCurve::with_range(&curve, first, last)?).cast();
        let surface = Handle::new(GeomAdaptorSurface::with_surface(&support.surface)?).cast();
        projection.adaptor = Adaptor3dCurveOnSurface::with_curve(&planar, &surface)?;
        projection.bounds = (first, last);
    }
    Ok(projection)
}
fn collect_non_manifold<T: WireDivider + ?Sized>(
    tool: &T,
    support: &WireSupport,
    edge: &Edge,
    curves: &DividedCurves,
) -> Result<Vec<(Vertex, f64)>, Failure> {
    let projection = vertex_projection(support, edge, curves)?;
    let mut vertices = Vec::new();
    for child in ShapeIterator::new(edge, false, true) {
        if !matches!(
            child.orientation(),
            Orientation::Internal | Orientation::External
        ) {
            continue;
        }
        let vertex = topods::vertex(&child)?;
        let point = BRepTool::pnt(&vertex)?;
        let parameter = project_vertex(&projection, point, tool.wire_divide_base().precision())?;
        vertices.push((vertex, parameter));
    }
    Ok(vertices)
}
fn project_vertex(
    projection: &VertexProjection,
    point: Point3,
    precision: f64,
) -> Result<f64, Failure> {
    if !projection.curve.is_null() {
        return Ok(ShapeAnalysisCurve
            .project(
                &projection.curve,
                point,
                precision,
                projection.bounds,
                false,
            )?
            .2);
    }
    Ok(ShapeAnalysisCurve
        .project_adaptor(&projection.adaptor, point, precision, true)?
        .2)
}
