//! Actual edge selection preserves untouched output on missing geometry and the small-circle direct append.
use super::StdSelectBrepSelectionTool;
use crate::{
    select_mgr::{SelectMgrEntityOwnerSource, SelectMgrSelection},
    select3d::*,
};
use rustycad_tkbrep::{
    brep::BRepTool,
    brep_adaptor::BRepAdaptorCurve,
    topods::{self, Shape},
};
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::{geom_abs::CurveType, gp::Point3};
impl StdSelectBrepSelectionTool {
    /// Preserve native append/output ordering for an edge; `Self::get_edge_sensitive(&shape,&owner,&selection,0.1,0.5,9,500.,&mut entity)?`.
    #[allow(clippy::too_many_arguments)] // Actual OCCT output overload carries both selection and retained sensitive output.
    pub fn get_edge_sensitive(
        shape: &Shape,
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        selection: &Handle<SelectMgrSelection>,
        deflection: f64,
        angle: f64,
        edge_points: i32,
        max_parameter: f64,
        output: &mut Handle<dyn Select3DSensitiveEntity>,
    ) -> Result<(), Failure> {
        let references = (owner, selection);
        let sampling = (deflection, angle, edge_points, max_parameter);
        brep_checked_edge_sensitive(shape, references, sampling, output)
    }
}
type EdgeSelectionReferences<'a> = (
    &'a Handle<dyn SelectMgrEntityOwnerSource>,
    &'a Handle<SelectMgrSelection>,
);
type EdgeSamplingCriteria = (f64, f64, i32, f64);
fn brep_checked_edge_sensitive(
    shape: &Shape,
    references: EdgeSelectionReferences<'_>,
    sampling: EdgeSamplingCriteria,
    output: &mut Handle<dyn Select3DSensitiveEntity>,
) -> Result<(), Failure> {
    let edge = topods::edge(shape)?;
    let polygon = super::brep_edge_polygons::brep_selection_polygon_points(&edge)?;
    if let Some(points) = polygon.filter(|points| !points.is_empty()) {
        *output = brep_polygon_sensitive(references.0, &points)?;
        return Ok(());
    }
    let curve = match rustycad_tkernel::standard::catch_native_failure(|| {
        BRepAdaptorCurve::with_edge(&edge)
    }) {
        Ok(curve) => curve,
        Err(failure) if failure.exception_type() == "Standard_NullObject" => return Ok(()),
        Err(failure) => return Err(failure),
    };
    brep_analytical_edge_sensitive(&edge, &curve, references, sampling, output)
}
fn brep_analytical_edge_sensitive(
    edge: &rustycad_tkbrep::topods::Edge,
    curve: &BRepAdaptorCurve,
    references: EdgeSelectionReferences<'_>,
    sampling: EdgeSamplingCriteria,
    output: &mut Handle<dyn Select3DSensitiveEntity>,
) -> Result<(), Failure> {
    match curve.get_type()? {
        CurveType::Line => *output = brep_line_sensitive(edge, curve, references.0)?,
        CurveType::Circle => brep_circle_sensitive(curve, references.0, references.1, output)?,
        _ => *output = brep_sampled_sensitive(curve, references.0, sampling)?,
    }
    Ok(())
}
fn brep_line_sensitive(
    edge: &rustycad_tkbrep::topods::Edge,
    curve: &BRepAdaptorCurve,
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
) -> Result<Handle<dyn Select3DSensitiveEntity>, Failure> {
    let (first, last) = BRepTool::range(edge)?;
    let segment = Select3DSensitiveSegment::new(owner, curve.value(first)?, curve.value(last)?);
    Ok(Handle::new(segment).cast())
}
fn brep_sampled_sensitive(
    curve: &BRepAdaptorCurve,
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    sampling: EdgeSamplingCriteria,
) -> Result<Handle<dyn Select3DSensitiveEntity>, Failure> {
    let (deflection, angle, edge_points, max_parameter) = sampling;
    let points = super::brep_edge_sampling::brep_selection_curve_points(
        curve,
        deflection,
        angle,
        edge_points,
        max_parameter,
    )?;
    Ok(Handle::new(Select3DSensitiveCurve::from_points(owner, &points)?).cast())
}
fn brep_polygon_sensitive(
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    points: &[Point3],
) -> Result<Handle<dyn Select3DSensitiveEntity>, Failure> {
    if points.len() == 2 {
        return Ok(Handle::new(Select3DSensitiveSegment::new(owner, points[0], points[1])).cast());
    }
    let curve = Select3DSensitiveCurve::from_points(owner, points)?;
    Ok(Handle::new(curve).cast())
}
fn brep_circle_sensitive(
    curve: &BRepAdaptorCurve,
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    selection: &Handle<SelectMgrSelection>,
    output: &mut Handle<dyn Select3DSensitiveEntity>,
) -> Result<(), Failure> {
    let circle = curve.circle()?;
    if circle.radius() <= precision::confusion() {
        return selection
            .add(&Handle::new(Select3DSensitivePoint::new(owner, circle.location())).cast());
    }
    let sensitive = Select3DSensitivePoly::circle(
        owner,
        &circle,
        curve.first_parameter()?,
        curve.last_parameter()?,
        false,
        12,
    )?;
    *output = Handle::new(sensitive).cast();
    Ok(())
}
