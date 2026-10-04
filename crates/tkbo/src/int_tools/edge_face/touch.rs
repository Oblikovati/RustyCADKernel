//! EdgeFace touch classification using the native Extrema_ExtCS primitive.
use super::EdgeFace;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkernel::{precision, standard::Failure};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkgeombase::extrema::ExtCS;

type SurfaceBounds = ((f64, f64), (f64, f64));

pub(super) fn check_touch(
    edge_face: &mut EdgeFace,
    first: f64,
    last: f64,
) -> Result<Option<f64>, Failure> {
    if full_input_range(edge_face, first, last)? {
        return Ok(None);
    }
    closest_extremum(edge_face, first, last)
}

fn full_input_range(edge_face: &EdgeFace, first: f64, last: f64) -> Result<bool, Failure> {
    let curve = edge_face.curve.get().expect("initialized edge-face curve");
    let resolution = curve.resolution(edge_face.criteria)?;
    Ok((first - edge_face.range.first()).abs() < resolution
        && (last - edge_face.range.last()).abs() < resolution)
}

fn closest_extremum(
    edge_face: &mut EdgeFace,
    first: f64,
    last: f64,
) -> Result<Option<f64>, Failure> {
    let curve_handle = edge_face.curve.clone();
    let surface_handle = edge_face.surface.clone();
    let curve = curve_handle.get().expect("initialized edge-face curve");
    let surface = surface_handle.get().expect("initialized edge-face surface");
    let (u, v) = surface_bounds(surface)?;
    let mut extrema = ExtCS::new();
    extrema.initialize_range(
        surface,
        u,
        v,
        (precision::p_confusion(), precision::p_confusion()),
    )?;
    extrema.perform(curve, first, last)?;
    let mut nearest = (1e100, 0.0);
    if extrema.is_done() {
        if extrema.is_parallel()? {
            return Ok(None);
        }
        nearest = closest_extremum_parameter(&extrema)?;
    }
    for parameter in [first, last, (first + last) * 0.5] {
        let distance = projected_distance(edge_face, parameter)?;
        if distance < nearest.0 {
            nearest = (distance, parameter);
        }
    }
    if nearest.0 > edge_face.criteria * edge_face.criteria {
        return Ok(None);
    }
    if (nearest.1 - first).abs() < precision::p_confusion()
        || (nearest.1 - last).abs() < precision::p_confusion()
        || nearest.1 > first && nearest.1 < last
    {
        return Ok(Some(nearest.1));
    }
    Ok(None)
}

fn surface_bounds(surface: &BRepAdaptorSurface) -> Result<SurfaceBounds, Failure> {
    Ok((
        (surface.first_u_parameter()?, surface.last_u_parameter()?),
        (surface.first_v_parameter()?, surface.last_v_parameter()?),
    ))
}

fn closest_extremum_parameter(extrema: &ExtCS<'_>) -> Result<(f64, f64), Failure> {
    let mut nearest = (1e100, 0.0);
    for index in 1..=extrema.nb_ext()? {
        let distance = extrema.square_distance(index)?;
        if distance < nearest.0 {
            nearest = (distance, extrema.points(index)?.0.parameter());
        }
    }
    Ok(nearest)
}

fn projected_distance(edge_face: &mut EdgeFace, parameter: f64) -> Result<f64, Failure> {
    let curve_handle = edge_face.curve.clone();
    let context_handle = edge_face.context.clone();
    let curve = curve_handle.get().expect("initialized edge-face curve");
    let context = context_handle.get().expect("edge-face context");
    let projector = context.proj_ps(&edge_face.face)?;
    let mut projector = projector.write();
    projector.perform(curve.value(parameter)?)?;
    if !projector.is_done() {
        drop(projector);
        edge_face.error_status = 4;
        return Ok(99.0);
    }
    let distance = projector.lower_distance()?;
    Ok(distance * distance)
}
