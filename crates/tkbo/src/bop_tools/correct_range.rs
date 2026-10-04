use crate::int_tools::Range;
use rustycad_tkbrep::{
    brep::BRepTool,
    brep_adaptor::BRepAdaptorCurve,
    topods::{Edge, Face},
};
use rustycad_tkernel::{precision, standard::Failure};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::geom_abs::CurveType;
/// Shrink a nonlinear edge range using both edge tolerances; `correct_range_edges(&first,&second,&range,&mut corrected)?`.
pub fn correct_range_edges(
    first: &Edge,
    second: &Edge,
    range: &Range,
    corrected: &mut Range,
) -> Result<(), Failure> {
    *corrected = *range;
    let curve = BRepAdaptorCurve::with_edge(first)?;
    if curve.get_type()? == CurveType::Line {
        return Ok(());
    }
    let tolerance = 2. * (BRepTool::edge_tolerance(first)? + BRepTool::edge_tolerance(second)?);
    shrink_range(&curve, tolerance, range, corrected)
}
/// Shrink an edge range using only the face tolerance; `correct_range_face(&edge,&face,&range,&mut corrected)?`.
pub fn correct_range_face(
    edge: &Edge,
    face: &Face,
    range: &Range,
    corrected: &mut Range,
) -> Result<(), Failure> {
    *corrected = *range;
    let curve = BRepAdaptorCurve::with_edge(edge)?;
    shrink_range(&curve, BRepTool::face_tolerance(face)?, range, corrected)
}
fn shrink_range(
    curve: &BRepAdaptorCurve,
    tolerance: f64,
    range: &Range,
    corrected: &mut Range,
) -> Result<(), Failure> {
    for (index, parameter) in [range.first(), range.last()].into_iter().enumerate() {
        let resolution = endpoint_resolution(curve, parameter, tolerance)?;
        if index == 0 {
            corrected.set_first(parameter + resolution);
        } else {
            corrected.set_last(parameter - resolution);
        }
        if corrected.last() - corrected.first() < precision::p_confusion() {
            *corrected = *range;
        }
    }
    Ok(())
}
fn endpoint_resolution(
    curve: &BRepAdaptorCurve,
    parameter: f64,
    tolerance: f64,
) -> Result<f64, Failure> {
    if matches!(
        curve.get_type()?,
        CurveType::BezierCurve
            | CurveType::BSplineCurve
            | CurveType::OffsetCurve
            | CurveType::OtherCurve
    ) {
        let magnitude = curve.d1(parameter)?.1.magnitude();
        if magnitude > 1e-12 {
            return Ok(tolerance / magnitude);
        }
    }
    curve.resolution(tolerance)
}
