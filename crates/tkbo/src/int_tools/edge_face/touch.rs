//! EdgeFace touch classification using the native Extrema_ExtCS primitive.
use super::EdgeFace;
use super::exact_intersection::exact_intersection_parameter;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkernel::{precision, standard::Failure};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkgeombase::extrema::ExtCS;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::{Line3, Point3};

type SurfaceBounds = ((f64, f64), (f64, f64));

#[cfg(test)]
#[path = "touch_tests.rs"]
mod tests;

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
    let Some(mut nearest) = extrema_candidate(edge_face, first, last)? else {
        return Ok(None);
    };
    for parameter in [first, last, (first + last) * 0.5] {
        let distance = distance_function(edge_face, parameter)? + edge_face.criteria;
        let distance = distance * distance;
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

fn extrema_candidate(
    edge_face: &EdgeFace,
    first: f64,
    last: f64,
) -> Result<Option<(f64, f64)>, Failure> {
    let curve = edge_face.curve.get().expect("initialized edge-face curve");
    let surface = edge_face
        .surface
        .get()
        .expect("initialized edge-face surface");
    let (u, v) = surface_bounds(surface)?;
    let mut extrema = ExtCS::new();
    extrema.initialize_range(
        surface,
        u,
        v,
        (precision::p_confusion(), precision::p_confusion()),
    )?;
    extrema.perform(curve, first, last)?;
    extrema_candidate_result(&extrema, curve, surface, first, last)
}

fn extrema_candidate_result(
    extrema: &ExtCS<'_>,
    curve: &dyn Adaptor3dCurve,
    surface: &dyn Adaptor3dSurface,
    first: f64,
    last: f64,
) -> Result<Option<(f64, f64)>, Failure> {
    if !extrema.is_done() {
        return Ok(Some((1e100, 0.0)));
    }
    if extrema.is_parallel()? {
        return Ok(None);
    }
    if extrema.nb_ext()? > 0 {
        return Ok(Some(closest_extremum_parameter(extrema)?));
    }
    Ok(Some(
        exact_intersection_parameter(curve, surface, first, last)?
            .map_or((1e100, 0.0), |t| (0.0, t)),
    ))
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

fn distance_function(edge_face: &mut EdgeFace, parameter: f64) -> Result<f64, Failure> {
    let curve_handle = edge_face.curve.clone();
    let context_handle = edge_face.context.clone();
    let curve = curve_handle.get().expect("initialized edge-face curve");
    let surface = edge_face
        .surface
        .get()
        .expect("initialized edge-face surface");
    let point = curve.value(parameter)?;
    if let Some(distance) = equal_surface_distance(surface, point)? {
        return Ok(distance - edge_face.criteria);
    }
    let context = context_handle.get().expect("edge-face context");
    let projector = context.proj_ps(&edge_face.face)?;
    let mut projector = projector.write();
    projector.perform(curve.value(parameter)?)?;
    if !projector.is_done() {
        drop(projector);
        edge_face.error_status = 4;
        return Ok(99.0);
    }
    Ok(projector.lower_distance()? - edge_face.criteria)
}

fn equal_surface_distance(
    surface: &impl Adaptor3dSurface,
    point: Point3,
) -> Result<Option<f64>, Failure> {
    match surface.get_type()? {
        SurfaceType::Cylinder => cylinder_axis_distance(surface, point),
        SurfaceType::Cone => cone_axis_distance(surface, point),
        SurfaceType::Torus => torus_ring_distance(surface, point),
        _ => Ok(None),
    }
}

fn cylinder_axis_distance(
    surface: &impl Adaptor3dSurface,
    point: Point3,
) -> Result<Option<f64>, Failure> {
    let cylinder = surface.cylinder()?;
    if Line3::from_axis(cylinder.axis()).distance(point) < 1e-7 {
        return Ok(Some(cylinder.radius()));
    }
    Ok(None)
}

fn cone_axis_distance(
    surface: &impl Adaptor3dSurface,
    point: Point3,
) -> Result<Option<f64>, Failure> {
    let cone = surface.cone()?;
    if Line3::from_axis(cone.axis()).distance(point) < 1e-7 {
        return Ok(Some(point.distance(&cone.apex()) * cone.semi_angle().tan()));
    }
    Ok(None)
}

fn torus_ring_distance(
    surface: &impl Adaptor3dSurface,
    point: Point3,
) -> Result<Option<f64>, Failure> {
    let torus = surface.torus()?;
    let ring_distance = (torus.location().distance(&point) - torus.major_radius()).abs();
    if ring_distance < 1e-7 {
        return Ok(Some(torus.minor_radius()));
    }
    Ok(None)
}

pub(super) fn check_touch_vertex(
    edge_face: &mut EdgeFace,
    first: f64,
    last: f64,
) -> Result<Option<f64>, Failure> {
    let middle = (first + last) * 0.5;
    let midpoint = distance_function(edge_face, middle)?;
    let Some(extrema_minimum) = vertex_extrema_minimum(edge_face, first, last)? else {
        return Ok(None);
    };
    select_touch_vertex(extrema_minimum, midpoint, edge_face, first, last)
}

fn vertex_extrema_minimum(
    edge_face: &EdgeFace,
    first: f64,
    last: f64,
) -> Result<Option<(f64, f64)>, Failure> {
    let curve_handle = edge_face.curve.clone();
    let surface_handle = edge_face.surface.clone();
    let curve = curve_handle.get().expect("initialized edge-face curve");
    let surface = surface_handle.get().expect("initialized edge-face surface");
    let mut extrema = ExtCS::new();
    let (u, v) = surface_bounds(surface)?;
    extrema.initialize_range(
        surface,
        u,
        v,
        (precision::p_confusion(), precision::p_confusion()),
    )?;
    extrema.perform(curve, first, last)?;
    if !extrema.is_done() || extrema.is_parallel()? || extrema.nb_ext()? == 0 {
        return Ok(None);
    }
    Ok(Some(closest_extremum_parameter(&extrema)?))
}

fn select_touch_vertex(
    (distance, parameter): (f64, f64),
    midpoint_distance: f64,
    edge_face: &EdgeFace,
    first: f64,
    last: f64,
) -> Result<Option<f64>, Failure> {
    if distance > midpoint_distance * midpoint_distance {
        return Ok(Some((first + last) * 0.5));
    }
    if distance > edge_face.criteria * edge_face.criteria
        || near_touch_endpoint(edge_face, parameter, first, last)?
    {
        return Ok(None);
    }
    if parameter > first && parameter < last {
        return Ok(Some(parameter));
    }
    Ok(None)
}

fn near_touch_endpoint(
    edge_face: &EdgeFace,
    parameter: f64,
    first: f64,
    last: f64,
) -> Result<bool, Failure> {
    let curve = edge_face.curve.get().expect("initialized edge-face curve");
    let epsilon = if curve.get_type()? == CurveType::Line {
        9e-5
    } else {
        8e-5
    };
    Ok((parameter - first).abs() < epsilon || (parameter - last).abs() < epsilon)
}
