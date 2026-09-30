//! Projection/linear selection remains dimension-sensitive at representation boundaries.
use super::*;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
pub(super) fn transfer(
    context: &TransferRangeContext,
    curve: &Handle<dyn BRepGCurve>,
) -> Result<bool, Failure> {
    let Some((mut parameters, planar)) = representation_parameters(context, curve)? else {
        return Ok(true);
    };
    if parameters[0] > parameters[1] {
        parameters.swap(0, 1);
    }
    if planar {
        let pcurve = curve.pcurve()?;
        parameters[0] = correction::correct_parameter(&pcurve, parameters[0])?;
        parameters[1] = correction::correct_parameter(&pcurve, parameters[1])?;
    }
    correction::expand_collapsed(&mut parameters, curve.range());
    curve.set_range(parameters[0], parameters[1])?;
    Ok(parameters == context.parameters)
}
fn representation_parameters(
    context: &TransferRangeContext,
    curve: &Handle<dyn BRepGCurve>,
) -> Result<Option<([f64; 2], bool)>, Failure> {
    Ok(Some(if curve.is_curve3d() {
        let Some(parameters) = spatial_parameters(context, curve)? else {
            return Ok(None);
        };
        (parameters, false)
    } else if curve.is_curve_on_surface() {
        (planar_parameters(context, curve)?, true)
    } else {
        return Ok(None);
    }))
}
fn spatial_parameters(
    context: &TransferRangeContext,
    curve: &Handle<dyn BRepGCurve>,
) -> Result<Option<[f64; 2]>, Failure> {
    if !context.from_planar {
        return Ok(Some(context.parameters));
    }
    let spatial = curve.curve3d()?;
    if spatial.is_null() {
        return Ok(None);
    }
    let adaptor = GeomAdaptorCurve::with_range(&spatial, curve.first(), curve.last())?;
    let points = local_points(context, curve)?;
    let linear = linear_parameters(context, curve);
    let (parameters, distances) = project_pair(context, &adaptor, points, linear)?;
    let positions = [spatial.value(linear[0])?, spatial.value(linear[1])?];
    let ends = context.linear_ends;
    Ok(Some(select_parameters(
        context, points, positions, linear, parameters, distances, ends,
    )))
}
fn planar_parameters(
    context: &TransferRangeContext,
    curve: &Handle<dyn BRepGCurve>,
) -> Result<[f64; 2], Failure> {
    let planar = Handle::new(Geom2dAdaptorCurve::with_range(
        &curve.pcurve()?,
        curve.first(),
        curve.last(),
    )?)
    .cast();
    let surface = Handle::new(GeomAdaptorSurface::with_surface(&curve.surface()?)?).cast();
    let adaptor = Adaptor3dCurveOnSurface::with_curve(&planar, &surface)?;
    let points = local_points(context, curve)?;
    let linear = linear_parameters(context, curve);
    let (parameters, distances) = project_pair(context, &adaptor, points, linear)?;
    let positions = [adaptor.value(linear[0])?, adaptor.value(linear[1])?];
    let span = curve.last() - curve.first();
    let ends = [
        context.linear_ends[0] || (parameters[0] - curve.first()) / span < precision::p_confusion(),
        context.linear_ends[1] || (curve.last() - parameters[1]) / span < precision::p_confusion(),
    ];
    Ok(select_parameters(
        context, points, positions, linear, parameters, distances, ends,
    ))
}
fn local_points(
    context: &TransferRangeContext,
    curve: &Handle<dyn BRepGCurve>,
) -> Result<[Point3; 2], Failure> {
    let location = (&context.edge_location * &curve.location())?.inverted()?;
    let transform = location.transformation();
    Ok([
        context.points[0].transformed(transform),
        context.points[1].transformed(transform),
    ])
}
fn linear_parameters(context: &TransferRangeContext, curve: &Handle<dyn BRepGCurve>) -> [f64; 2] {
    let span = curve.last() - curve.first();
    [
        curve.first() + context.fractions[0] * span,
        curve.first() + context.fractions[1] * span,
    ]
}
fn project_pair(
    context: &TransferRangeContext,
    adaptor: &dyn Adaptor3dCurve,
    points: [Point3; 2],
    linear: [f64; 2],
) -> Result<([f64; 2], [f64; 2]), Failure> {
    let first = ShapeAnalysisCurve.next_project_adaptor(
        linear[0],
        adaptor,
        points[0],
        context.precision,
    )?;
    let last = ShapeAnalysisCurve.next_project_adaptor(
        linear[1],
        adaptor,
        points[1],
        context.precision,
    )?;
    Ok(([first.2, last.2], [first.0, last.0]))
}
fn select_parameters(
    context: &TransferRangeContext,
    points: [Point3; 2],
    positions: [Point3; 2],
    linear: [f64; 2],
    mut projected: [f64; 2],
    distances: [f64; 2],
    ends: [bool; 2],
) -> [f64; 2] {
    // CATIA bplseitli.model FAC1155: degenerate projections retain the linear interval.
    let collapsed = (projected[0] - projected[1]).abs() < precision::p_confusion();
    for index in 0..2 {
        let deviation = positions[index].distance(&points[index]);
        if ends[index]
            || collapsed
            || projection::prefer_linear(deviation, distances[index], context.precision)
        {
            projected[index] = linear[index];
        }
    }
    projected
}
