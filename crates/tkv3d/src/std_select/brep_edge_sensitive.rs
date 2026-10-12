//! Actual edge selection preserves untouched output on missing geometry and the small-circle direct append.
use super::StdSelectBrepSelectionTool;
use crate::{
    select_mgr::{SelectMgrEntityOwnerSource, SelectMgrSelection},
    select3d::*,
};
use rustycad_tkbrep::{brep::BRepTool, brep_adaptor::BRepAdaptorCurve, topods::Shape};
use rustycad_tkernel::standard::HandleReference;
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::{geom_abs::CurveType, gp::Point3};
impl StdSelectBrepSelectionTool {
    /// Replace the original sensitive field with native destruction ordering, or leave it untouched.
    /// Example: `Self::get_edge_sensitive_into(&shape,&owner,&selection,0.1,0.5,9,500.,&slot)?`.
    #[allow(clippy::too_many_arguments)] // Original overload exposes sampling, selection and mutable output separately.
    pub fn get_edge_sensitive_into(
        shape: &Shape,
        owner: &dyn HandleReference<dyn SelectMgrEntityOwnerSource>,
        selection: &dyn HandleReference<SelectMgrSelection>,
        deflection: f64,
        angle: f64,
        edge_points: i32,
        max_parameter: f64,
        output: &rustycad_tkernel::standard::IntrusiveReferenceSlot<dyn Select3DSensitiveEntity>,
    ) -> Result<(), Failure> {
        let references = (owner, selection);
        let sampling = (deflection, angle, edge_points, max_parameter);
        let mut destination = output;
        brep_checked_edge_sensitive(shape, references, sampling, &mut destination)
    }
    /// Preserve native append/output ordering for an edge; `Self::get_edge_sensitive(&shape,&owner,&selection,0.1,0.5,9,500.,&mut entity)?`.
    #[allow(clippy::too_many_arguments)] // Actual OCCT output overload carries both selection and retained sensitive output.
    pub fn get_edge_sensitive(
        shape: &Shape,
        owner: &dyn HandleReference<dyn SelectMgrEntityOwnerSource>,
        selection: &dyn HandleReference<SelectMgrSelection>,
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
    &'a dyn HandleReference<dyn SelectMgrEntityOwnerSource>,
    &'a dyn HandleReference<SelectMgrSelection>,
);
type EdgeSamplingCriteria = (f64, f64, i32, f64);
fn brep_checked_edge_sensitive(
    shape: &Shape,
    references: EdgeSelectionReferences<'_>,
    sampling: EdgeSamplingCriteria,
    output: &mut dyn super::brep_edge_destination::BrepEdgeEntityDestination,
) -> Result<(), Failure> {
    rustycad_tkbrep::topods::Edge::with_shape_reference(shape, |edge| {
        build_borrowed_brep_edge(edge, references, sampling, output)
    })
}
fn build_borrowed_brep_edge(
    edge: &rustycad_tkbrep::topods::Edge,
    references: EdgeSelectionReferences<'_>,
    sampling: EdgeSamplingCriteria,
    output: &mut dyn super::brep_edge_destination::BrepEdgeEntityDestination,
) -> Result<(), Failure> {
    let polygon = super::brep_edge_polygons::brep_selection_polygon_points(edge)?;
    if let Some(points) = polygon.filter(|points| !points.is_empty()) {
        output.replace(brep_polygon_sensitive(references.0, &points)?)?;
        return Ok(());
    }
    let Some(curve) = load_brep_edge_curve(edge)? else {
        return Ok(());
    };
    brep_analytical_edge_sensitive(edge, &curve, references, sampling, output)
}
fn load_brep_edge_curve(
    edge: &rustycad_tkbrep::topods::Edge,
) -> Result<Option<BRepAdaptorCurve>, Failure> {
    match rustycad_tkernel::standard::catch_native_failure(|| BRepAdaptorCurve::with_edge(edge)) {
        Ok(curve) => Ok(Some(curve)),
        Err(failure) if failure.exception_type() == "Standard_NullObject" => Ok(None),
        Err(failure) => Err(failure),
    }
}
fn brep_analytical_edge_sensitive(
    edge: &rustycad_tkbrep::topods::Edge,
    curve: &BRepAdaptorCurve,
    references: EdgeSelectionReferences<'_>,
    sampling: EdgeSamplingCriteria,
    output: &mut dyn super::brep_edge_destination::BrepEdgeEntityDestination,
) -> Result<(), Failure> {
    match curve.get_type()? {
        CurveType::Line => output.replace(brep_line_sensitive(edge, curve, references.0)?)?,
        CurveType::Circle => brep_circle_sensitive(curve, references.0, references.1, output)?,
        _ => output.replace(brep_sampled_sensitive(curve, references.0, sampling)?)?,
    }
    Ok(())
}
fn brep_line_sensitive(
    edge: &rustycad_tkbrep::topods::Edge,
    curve: &BRepAdaptorCurve,
    owner: &dyn HandleReference<dyn SelectMgrEntityOwnerSource>,
) -> Result<Handle<dyn Select3DSensitiveEntity>, Failure> {
    let (first, last) = BRepTool::range(edge)?;
    let points = (curve.value(first)?, curve.value(last)?);
    let segment = Select3DSensitiveSegment::new(&owner.observe(), points.0, points.1);
    Ok(Handle::new(segment).cast())
}
fn brep_sampled_sensitive(
    curve: &BRepAdaptorCurve,
    owner: &dyn HandleReference<dyn SelectMgrEntityOwnerSource>,
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
    Ok(Handle::new(Select3DSensitiveCurve::from_points(
        &owner.observe(),
        &points,
    )?)
    .cast())
}
fn brep_polygon_sensitive(
    owner: &dyn HandleReference<dyn SelectMgrEntityOwnerSource>,
    points: &[Point3],
) -> Result<Handle<dyn Select3DSensitiveEntity>, Failure> {
    if points.len() == 2 {
        return Ok(Handle::new(Select3DSensitiveSegment::new(
            &owner.observe(),
            points[0],
            points[1],
        ))
        .cast());
    }
    let curve = Select3DSensitiveCurve::from_points(&owner.observe(), points)?;
    Ok(Handle::new(curve).cast())
}
fn brep_circle_sensitive(
    curve: &BRepAdaptorCurve,
    owner: &dyn HandleReference<dyn SelectMgrEntityOwnerSource>,
    selection: &dyn HandleReference<SelectMgrSelection>,
    output: &mut dyn super::brep_edge_destination::BrepEdgeEntityDestination,
) -> Result<(), Failure> {
    let circle = curve.circle()?;
    if circle.radius() <= precision::confusion() {
        return selection.observe().add(
            &Handle::new(Select3DSensitivePoint::new(
                &owner.observe(),
                circle.location(),
            ))
            .cast(),
        );
    }
    output.replace(sampled_brep_circle(curve, owner, &circle)?)?;
    Ok(())
}
fn sampled_brep_circle(
    curve: &BRepAdaptorCurve,
    owner: &dyn HandleReference<dyn SelectMgrEntityOwnerSource>,
    circle: &rustycad_tkmath::gp::Circle3,
) -> Result<Handle<dyn Select3DSensitiveEntity>, Failure> {
    let range = (curve.first_parameter()?, curve.last_parameter()?);
    let sensitive =
        Select3DSensitivePoly::circle(&owner.observe(), circle, range.0, range.1, false, 12)?;
    Ok(Handle::new(sensitive).cast())
}
