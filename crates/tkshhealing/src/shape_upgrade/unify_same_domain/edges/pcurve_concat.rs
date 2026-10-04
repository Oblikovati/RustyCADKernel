use super::pcurve_union::OrientedPCurve;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBoundedCurve, Geom2dCurve, Geom2dTrimmedCurve,
};
use rustycad_tkgeombase::geom2d_convert::{
    CompCurveToBSplineCurve, HArrayOfBSplineCurves, c0_bspline_to_c1_bspline_curve, concat_c1,
    curve_to_bspline_curve,
};
use rustycad_tkmath::convert::ParameterisationType;

/// Builds and stores the native C1 concatenation result for two stored p-curves.
pub(super) fn union_general_curves(
    face: &Face,
    merged: &Edge,
    first: &OrientedPCurve,
    second: &OrientedPCurve,
) -> Result<(), Failure> {
    if !has_two_stored_curves(first, second) {
        return Ok(());
    }
    let curves = c1_curve_pair(first, second)?;
    let vertex_tolerance = common_vertex_tolerance(first, second)?;
    let max_tolerance = max_pcurve_tolerance(merged, vertex_tolerance)?;
    let result = concatenate_pcurves(curves, vertex_tolerance, max_tolerance)?;
    store_bspline_pcurve(face, merged, result)
}

fn has_two_stored_curves(first: &OrientedPCurve, second: &OrientedPCurve) -> bool {
    first.pcurve.is_stored
        && second.pcurve.is_stored
        && !first.pcurve.pcurve.is_null()
        && !second.pcurve.pcurve.is_null()
}

fn c1_curve_pair(
    first: &OrientedPCurve,
    second: &OrientedPCurve,
) -> Result<[Handle<Geom2dBSplineCurve>; 2], Failure> {
    Ok([
        convert_to_c1_bspline(first)?,
        convert_to_c1_bspline(second)?,
    ])
}

fn convert_to_c1_bspline(curve: &OrientedPCurve) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
    let trimmed = trimmed_curve(curve)?;
    reverse_trimmed_for_occurrence(curve, &trimmed)?;
    let mut spline = curve_to_bspline_curve(&trimmed, ParameterisationType::TgtThetaOver2)?;
    c0_bspline_to_c1_bspline_curve(&mut spline, precision::confusion())?;
    Ok(spline)
}

fn trimmed_curve(curve: &OrientedPCurve) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let trimmed = Handle::new(Geom2dTrimmedCurve::new(
        &curve.pcurve.pcurve,
        curve.pcurve.first,
        curve.pcurve.last,
        true,
        true,
    )?);
    Ok(trimmed.cast())
}

fn reverse_trimmed_for_occurrence(
    curve: &OrientedPCurve,
    trimmed: &Handle<dyn Geom2dCurve>,
) -> Result<(), Failure> {
    if curve.edge_orientation == rustycad_tkg3d::top_abs::Orientation::Reversed {
        trimmed.get().ok_or_else(null_spline)?.reverse()?;
    }
    Ok(())
}

fn common_vertex_tolerance(
    first: &OrientedPCurve,
    second: &OrientedPCurve,
) -> Result<f64, Failure> {
    let shared =
        common_vertex(&first.source_edge, &second.source_edge).ok_or_else(missing_join_vertex)?;
    BRepTool::vertex_tolerance(&shared)
}

fn common_vertex(first: &Edge, second: &Edge) -> Option<rustycad_tkbrep::topods::Vertex> {
    let analysis = crate::shape_analysis::ShapeAnalysisEdge::new();
    let vertices = [analysis.first_vertex(first), analysis.last_vertex(first)];
    let other = [analysis.first_vertex(second), analysis.last_vertex(second)];
    vertices
        .into_iter()
        .find(|vertex| other.iter().any(|candidate| vertex.is_same(candidate)))
}

fn max_pcurve_tolerance(merged: &Edge, vertex_tolerance: f64) -> Result<f64, Failure> {
    Ok(BRepTool::edge_tolerance(merged)?.max(vertex_tolerance))
}

fn concatenate_pcurves(
    curves: [Handle<Geom2dBSplineCurve>; 2],
    junction_tolerance: f64,
    max_tolerance: f64,
) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
    let mut curves = curve_array(curves)?;
    let tolerances = tolerance_array(junction_tolerance)?;
    let (mut indices, mut joined, mut closed) = (Handle::default(), Handle::default(), false);
    concat_c1(
        &mut curves,
        &tolerances,
        &mut indices,
        &mut joined,
        &mut closed,
        precision::confusion(),
    )?;
    join_groups(&joined, max_tolerance)
}

fn curve_array(
    curves: [Handle<Geom2dBSplineCurve>; 2],
) -> Result<Array1<'static, Handle<Geom2dBSplineCurve>>, Failure> {
    let mut values = Array1::with_size(2);
    values
        .set_value(0, curves[0].clone())
        .map_err(array_failure)?;
    values
        .set_value(1, curves[1].clone())
        .map_err(array_failure)?;
    Ok(values)
}

fn tolerance_array(tolerance: f64) -> Result<Array1<'static, f64>, Failure> {
    let mut values = Array1::with_size(1);
    values.set_value(0, tolerance).map_err(array_failure)?;
    Ok(values)
}

fn join_groups(
    groups: &HArrayOfBSplineCurves,
    tolerance: f64,
) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
    let groups = groups.get().ok_or_else(null_concat_groups)?;
    let curves = groups.array1();
    let first = curves.value(curves.lower()).map_err(array_failure)?.clone();
    if curves.len() == 1 {
        return Ok(first);
    }
    concatenate_groups(&curves, &first, tolerance)
}

fn concatenate_groups(
    curves: &Array1<'_, Handle<Geom2dBSplineCurve>>,
    first: &Handle<Geom2dBSplineCurve>,
    tolerance: f64,
) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
    let bounded: Handle<dyn Geom2dBoundedCurve> = first.clone().cast();
    let mut combined =
        CompCurveToBSplineCurve::with_curve(&bounded, ParameterisationType::QuasiAngular)?;
    for index in (curves.lower() + 1)..=curves.upper() {
        let next = curves.value(index).map_err(array_failure)?;
        let bounded: Handle<dyn Geom2dBoundedCurve> = next.clone().cast();
        if !combined.add(&bounded, tolerance, true)? {
            return Err(concat_failure());
        }
    }
    Ok(combined.bspline_curve())
}

fn store_bspline_pcurve(
    face: &Face,
    merged: &Edge,
    spline: Handle<Geom2dBSplineCurve>,
) -> Result<(), Failure> {
    let curve: Handle<dyn Geom2dCurve> = spline.cast();
    let (surface, location) = BRepTool::surface_with_location(face)?;
    let tolerance = BRepTool::edge_tolerance(merged)?;
    BRepBuilder::default().update_edge_pcurve(merged, &curve, &surface, &location, tolerance, None)
}

fn null_spline() -> Failure {
    Failure::with_type("Standard_NullObject", "null p-curve B-spline", "")
}

fn missing_join_vertex() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "null common vertex for merged edge p-curves; expected one shared vertex",
        "",
    )
}

fn array_failure(error: rustycad_tkernel::ncollection::ArrayError) -> Failure {
    Failure::with_type(
        "Standard_OutOfRange",
        &format!("p-curve concatenation array error {error:?}; expected valid zero-based index"),
        "",
    )
}

fn null_concat_groups() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "null concatenated p-curve table; expected at least one C1 group",
        "",
    )
}

fn concat_failure() -> Failure {
    Failure::with_type(
        "Standard_ConstructionError",
        "p-curve composite rejected adjacent C1 group; expected connected groups",
        "",
    )
}
