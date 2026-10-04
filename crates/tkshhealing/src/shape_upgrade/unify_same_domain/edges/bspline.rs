use crate::shape_analysis::ShapeAnalysisEdge;
use crate::shape_build::ShapeBuildReShape;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Edge, Shape, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomBoundedCurve, GeomCurve, GeomTrimmedCurve};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeombase::geom_convert::{
    CompCurveToBSplineCurve, c0_bspline_to_c1_bspline_curve, curve_to_bspline_curve,
};
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;

pub(super) fn merge_first_pair(
    shape: &Shape,
    edges: &[Edge],
    angular_tolerance: f64,
) -> Result<Option<Shape>, Failure> {
    let Some((first, second, start, end)) = first_pair(edges, angular_tolerance)? else {
        return Ok(None);
    };
    let curve = concatenate_curves(&first, &second)?;
    replace_pair(shape, &first, &second, &start, &end, curve).map(Some)
}

pub(super) fn has_candidate(shape: &Shape, tolerance: f64) -> Result<bool, Failure> {
    let edges = Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)?
        .map(|edge| rustycad_tkbrep::topods::edge(&edge))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(first_pair(&edges, tolerance)?.is_some())
}

fn first_pair(
    edges: &[Edge],
    tolerance: f64,
) -> Result<Option<(Edge, Edge, Vertex, Vertex)>, Failure> {
    for (index, first) in edges.iter().enumerate() {
        for second in edges.iter().skip(index + 1) {
            if let Some(pair) = compatible_pair(first, second, tolerance)? {
                return Ok(Some(pair));
            }
        }
    }
    Ok(None)
}

fn compatible_pair(
    first: &Edge,
    second: &Edge,
    tolerance: f64,
) -> Result<Option<(Edge, Edge, Vertex, Vertex)>, Failure> {
    if !both_spline_based(first, second)? {
        return Ok(None);
    }
    let vertices = edge_vertices(first, second);
    let Some((first_start, first_end, second_start, second_end)) = vertices else {
        return Ok(None);
    };
    let Some(pair) = ordered_pair(
        first,
        second,
        first_start,
        first_end,
        second_start,
        second_end,
    )?
    else {
        return Ok(None);
    };
    if !aligned_join(&pair.0, &pair.1, tolerance)? {
        return Ok(None);
    }
    Ok(Some(pair))
}

fn both_spline_based(first: &Edge, second: &Edge) -> Result<bool, Failure> {
    let first_type = BRepAdaptorCurve::with_edge(first)?.get_type()?;
    let second_type = BRepAdaptorCurve::with_edge(second)?.get_type()?;
    Ok(is_spline_based(first_type) && is_spline_based(second_type))
}

fn is_spline_based(curve_type: CurveType) -> bool {
    matches!(curve_type, CurveType::BezierCurve | CurveType::BSplineCurve)
}

fn edge_vertices(first: &Edge, second: &Edge) -> Option<(Vertex, Vertex, Vertex, Vertex)> {
    let analysis = ShapeAnalysisEdge::new();
    let (first_start, first_end) = (analysis.first_vertex(first), analysis.last_vertex(first));
    let (second_start, second_end) = (analysis.first_vertex(second), analysis.last_vertex(second));
    if first_start.is_null()
        || first_end.is_null()
        || second_start.is_null()
        || second_end.is_null()
    {
        return None;
    }
    Some((first_start, first_end, second_start, second_end))
}

pub(super) fn aligned_join(first: &Edge, second: &Edge, tolerance: f64) -> Result<bool, Failure> {
    let first_tangent = tangent_at(first, true)?;
    let second_tangent = tangent_at(second, false)?;
    Ok(first_tangent.angle(second_tangent)? <= tolerance)
}

fn tangent_at(edge: &Edge, at_end: bool) -> Result<rustycad_tkmath::gp::Vector3, Failure> {
    let adaptor = BRepAdaptorCurve::with_edge(edge)?;
    let (first, last) = (adaptor.first_parameter()?, adaptor.last_parameter()?);
    let parameter = oriented_parameter(edge, at_end, first, last);
    let (_, mut tangent) = adaptor.d1(parameter)?;
    if edge.orientation() == Orientation::Reversed {
        tangent = -tangent;
    }
    Ok(tangent)
}

fn oriented_parameter(edge: &Edge, at_end: bool, first: f64, last: f64) -> f64 {
    match (edge.orientation(), at_end) {
        (Orientation::Forward, true) | (Orientation::Reversed, false) => last,
        _ => first,
    }
}

fn ordered_pair(
    first: &Edge,
    second: &Edge,
    first_start: Vertex,
    first_end: Vertex,
    second_start: Vertex,
    second_end: Vertex,
) -> Result<Option<(Edge, Edge, Vertex, Vertex)>, Failure> {
    if first_end.is_same(&second_start) {
        return Ok(Some((
            first.clone(),
            second.clone(),
            first_start,
            second_end,
        )));
    }
    if second_end.is_same(&first_start) {
        return Ok(Some((
            second.clone(),
            first.clone(),
            second_start,
            first_end,
        )));
    }
    Ok(None)
}

fn concatenate_curves(
    first: &Edge,
    second: &Edge,
) -> Result<Handle<rustycad_tkg3d::geom::GeomBSplineCurve>, Failure> {
    let mut first_curve = edge_as_c1_bspline(first)?;
    let mut second_curve = edge_as_c1_bspline(second)?;
    reverse_for_occurrence(first, &mut first_curve)?;
    reverse_for_occurrence(second, &mut second_curve)?;
    let mut combined = new_composer(&first_curve)?;
    append_curve(&mut combined, &second_curve, join_tolerance(first, second)?)?;
    Ok(combined.bspline_curve())
}

fn new_composer(
    curve: &Handle<rustycad_tkg3d::geom::GeomBSplineCurve>,
) -> Result<CompCurveToBSplineCurve, Failure> {
    let bounded: Handle<dyn GeomBoundedCurve> = curve.cast();
    CompCurveToBSplineCurve::with_curve(&bounded, ParameterisationType::TgtThetaOver2)
}

fn append_curve(
    combined: &mut CompCurveToBSplineCurve,
    curve: &Handle<rustycad_tkg3d::geom::GeomBSplineCurve>,
    tolerance: f64,
) -> Result<(), Failure> {
    let bounded: Handle<dyn GeomBoundedCurve> = curve.cast();
    if !combined.add(&bounded, tolerance, true, true, 0)? {
        return Err(Failure::with_type(
            "Standard_ConstructionError",
            "GeomConvert_CompCurveToBSplineCurve::Add rejected adjacent B-splines",
            "",
        ));
    }
    Ok(())
}

fn edge_as_c1_bspline(
    edge: &Edge,
) -> Result<Handle<rustycad_tkg3d::geom::GeomBSplineCurve>, Failure> {
    let (curve, first, last) = BRepTool::curve(edge)?;
    let trimmed: Handle<dyn GeomCurve> =
        Handle::new(GeomTrimmedCurve::new(&curve, first, last, true, true)?).cast();
    let mut spline = curve_to_bspline_curve(&trimmed, ParameterisationType::TgtThetaOver2)?;
    c0_bspline_to_c1_bspline_curve(&mut spline, precision::confusion(), precision::angular())?;
    Ok(spline)
}

fn reverse_for_occurrence(
    edge: &Edge,
    curve: &mut Handle<rustycad_tkg3d::geom::GeomBSplineCurve>,
) -> Result<(), Failure> {
    if edge.orientation() == Orientation::Reversed {
        curve.get().ok_or_else(null_bspline)?.reverse()?;
    }
    Ok(())
}

fn join_tolerance(first: &Edge, second: &Edge) -> Result<f64, Failure> {
    let vertices = edge_vertices(first, second).ok_or_else(null_vertex)?;
    let mut tolerance = precision::confusion();
    for vertex in [vertices.0, vertices.1, vertices.2, vertices.3] {
        tolerance = tolerance.max(BRepTool::vertex_tolerance(&vertex)?);
    }
    Ok(tolerance)
}

fn replace_pair(
    shape: &Shape,
    first: &Edge,
    second: &Edge,
    start: &Vertex,
    end: &Vertex,
    curve: Handle<rustycad_tkg3d::geom::GeomBSplineCurve>,
) -> Result<Shape, Failure> {
    let geometry: Handle<dyn GeomCurve> = curve.cast();
    let edge = BRepBuilderAPIMakeEdge::from_curve_vertices_params(
        &geometry,
        start,
        end,
        curve.get().ok_or_else(null_bspline)?.first_parameter(),
        curve.get().ok_or_else(null_bspline)?.last_parameter(),
    )?
    .edge()?;
    apply_replacement(shape, first, second, &edge)
}

fn apply_replacement(
    shape: &Shape,
    first: &Edge,
    second: &Edge,
    edge: &Edge,
) -> Result<Shape, Failure> {
    let context = Handle::new(ShapeBuildReShape::new());
    context.merge(
        &[first.clone().into_shape(), second.clone().into_shape()],
        &edge.clone().into_shape(),
    )?;
    context.apply(shape, ShapeEnum::Shape)
}

fn null_bspline() -> Failure {
    Failure::with_type("Standard_NullObject", "null B-spline curve", "")
}

fn null_vertex() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "null vertex; expected two curve-chain endpoints",
        "",
    )
}
