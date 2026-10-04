use super::EdgeVertexPart;
use rustycad_tkbrep::{
    brep::BRepTool,
    brep_adaptor::BRepAdaptorCurve,
    topods::{Edge, Vertex},
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::geom_abs::CurveType;

pub(super) fn intersection_tolerance(
    event: &EdgeVertexPart,
    first_edge: &Edge,
    second_edge: &Edge,
    vertex: &Vertex,
) -> Result<f64, Failure> {
    let first_type = BRepAdaptorCurve::with_edge(first_edge)?.get_type()?;
    let second_type = BRepAdaptorCurve::with_edge(second_edge)?.get_type()?;
    if !is_line_circle_pair(first_type, second_type) {
        return BRepTool::vertex_tolerance(vertex);
    }
    tolerance_for_line_circle(event, first_type, vertex)
}

fn is_line_circle_pair(first: CurveType, second: CurveType) -> bool {
    (first == CurveType::Line && second == CurveType::Circle)
        || (first == CurveType::Circle && second == CurveType::Line)
}

fn tolerance_for_line_circle(
    event: &EdgeVertexPart,
    first_type: CurveType,
    vertex: &Vertex,
) -> Result<f64, Failure> {
    let (range_first, range_last) = if first_type == CurveType::Line {
        event.common.range1_bounds()
    } else {
        event
            .common
            .ranges2()
            .first()
            .ok_or_else(|| Failure::new("PerformEE line/circle part requires range2[0]"))?
            .range()
    };
    let minimum = (range_last - range_first) / 2.0;
    let current = BRepTool::vertex_tolerance(vertex)?;
    Ok(if minimum > current { minimum } else { current })
}
