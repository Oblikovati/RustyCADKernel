//! Native projection competes with the linear estimate, rather than replacing it unconditionally.
use super::*;
pub(super) fn segment(
    tool: &ShapeAnalysisTransferParametersProj,
    state: &ProjectionTransferState,
    parameter: f64,
    to_2d: bool,
    bounds: (f64, f64),
) -> Result<f64, Failure> {
    let linear = tool.parent.perform_parameter(parameter, to_2d)?;
    let (distance, projected, deviation) = if to_2d {
        project_planar(state, parameter, linear, bounds)?
    } else {
        project_spatial(state, parameter, linear, bounds)?
    };
    if prefer_linear(deviation, distance, state.precision) {
        return Ok(linear);
    }
    Ok(projected)
}
fn project_planar(
    state: &ProjectionTransferState,
    parameter: f64,
    linear: f64,
    bounds: (f64, f64),
) -> Result<(f64, f64, f64), Failure> {
    let point = state
        .spatial
        .value(parameter)?
        .transformed(state.location.inverted()?.transformation());
    let curve = Handle::new(Geom2dAdaptorCurve::with_range(
        &state.planar,
        bounds.0,
        bounds.1,
    )?)
    .cast();
    let adaptor = Adaptor3dCurveOnSurface::with_curve(&curve, &state.adaptor.get_surface())?;
    let (distance, _, projected) =
        ShapeAnalysisCurve.project_adaptor(&adaptor, point, state.precision, true)?;
    let deviation = point.distance(&adaptor.value(linear)?);
    Ok((distance, projected, deviation))
}
fn project_spatial(
    state: &ProjectionTransferState,
    parameter: f64,
    linear: f64,
    bounds: (f64, f64),
) -> Result<(f64, f64, f64), Failure> {
    let point = state
        .adaptor
        .value(parameter)?
        .transformed(state.location.transformation());
    let (distance, _, projected) =
        ShapeAnalysisCurve.project(&state.spatial, point, state.precision, bounds, false)?;
    let deviation = point.distance(&state.spatial.value(linear)?);
    Ok((distance, projected, deviation))
}
pub(super) fn prefer_linear(linear_distance: f64, projected_distance: f64, precision: f64) -> bool {
    linear_distance <= projected_distance
        || (linear_distance < precision && linear_distance <= 2. * projected_distance)
}
pub(super) fn clamp_parameter(mut parameter: f64, bounds: (f64, f64)) -> f64 {
    if parameter < bounds.0 {
        parameter = bounds.0;
    }
    if parameter > bounds.1 {
        parameter = bounds.1;
    }
    parameter
}
