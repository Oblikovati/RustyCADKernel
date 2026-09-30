//! Private Contap_Contour math at OCCT 3d097a0; validated before orchestration integration.
use crate::contap::{ContapIType, ContapLine, ContapSurfFunction, ContapSurfProps};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::h_surface_tool::{self as surface_tool, Adaptor3dSurfaceHandle};
use rustycad_tkgeomalgo::int_surf::IntSurfTypeTrans;
use rustycad_tkmath::{
    elclib, elslib,
    geom_abs::SurfaceType,
    gp::{Circle3, Line3, Point2, Point3, Vector3},
    math::{Matrix, Vector},
};

pub(super) fn recadre(
    surface: &Adaptor3dSurfaceHandle,
    u: &mut f64,
    v: &mut f64,
) -> Result<(), Failure> {
    let kind = surface_tool::get_type(surface)?;
    if matches!(
        kind,
        SurfaceType::Cylinder | SurfaceType::Cone | SurfaceType::Sphere | SurfaceType::Torus
    ) {
        let first = surface.first_u_parameter()?;
        let last = surface.last_u_parameter()?;
        wrap_parameter(u, first, last);
    }
    if kind == SurfaceType::Torus {
        let first = surface.first_v_parameter()?;
        let last = surface.last_v_parameter()?;
        wrap_parameter(v, first, last);
    }
    Ok(())
}
fn wrap_parameter(parameter: &mut f64, first: f64, last: f64) {
    let period = std::f64::consts::PI + std::f64::consts::PI;
    while *parameter < first {
        *parameter += period;
    }
    while *parameter > last {
        *parameter -= period;
    }
}
fn tangent_parameters(du: Vector3, dv: Vector3, tangent: Vector3) -> Option<(f64, f64)> {
    let dut = du.dot(tangent);
    let dvt = dv.dot(tangent);
    let uu = du.dot(du);
    let vv = dv.dot(dv);
    let uv = du.dot(dv);
    let determinant = uu * vv - uv * uv;
    if determinant < f64::EPSILON {
        return None;
    }
    Some((
        (dut * vv - dvt * uv) / determinant,
        (uu * dvt - uv * dut) / determinant,
    ))
}
pub(super) fn transition_on_line(
    function: &mut ContapSurfFunction,
    u: f64,
    v: f64,
    tangent: Vector3,
) -> Result<IntSurfTypeTrans, Failure> {
    let derivative = surface_tool::d1(&function.surface(), u, v)?;
    let Some((alpha, beta)) = tangent_parameters(derivative.d1u, derivative.d1v, tangent) else {
        return Ok(IntSurfTypeTrans::Undecided);
    };
    let mut parameters = Vector::with_bounds(1, 2)?;
    parameters[1] = u;
    parameters[2] = v;
    let mut gradient = Matrix::with_bounds(1, 1, 1, 2).expect("fixed contour gradient bounds");
    function.derivatives(&parameters, &mut gradient)?;
    let determinant = -gradient[(1, 1)] * beta + gradient[(1, 2)] * alpha;
    // OCCT 3d097a0 compares the signed determinant, leaving negative values undecided.
    if determinant < f64::EPSILON {
        return Ok(IntSurfTypeTrans::Undecided);
    }
    Ok(if determinant > 0. {
        IntSurfTypeTrans::Out
    } else {
        IntSurfTypeTrans::In
    })
}
fn analytic_parameters(
    surface: &Adaptor3dSurfaceHandle,
    point: Point3,
) -> Result<(f64, f64), Failure> {
    Ok(match surface_tool::get_type(surface)? {
        SurfaceType::Cylinder => elslib::parameters(&surface_tool::cylinder(surface)?, point),
        SurfaceType::Cone => elslib::parameters(&surface_tool::cone(surface)?, point),
        SurfaceType::Sphere => elslib::parameters(&surface_tool::sphere(surface)?, point),
        _ => (0., 0.),
    })
}
pub(super) fn line_transition(
    function: &mut ContapSurfFunction,
    line: Line3,
) -> Result<IntSurfTypeTrans, Failure> {
    let (point, tangent) = elclib::d1(0., &line);
    let (u, v) = analytic_parameters(&function.surface(), point)?;
    transition_on_line(function, u, v, tangent)
}
pub(super) fn circle_transition(
    function: &mut ContapSurfFunction,
    circle: Circle3,
) -> Result<IntSurfTypeTrans, Failure> {
    let (point, tangent) = elclib::d1(0., &circle);
    let (u, v) = analytic_parameters(&function.surface(), point)?;
    transition_on_line(function, u, v, tangent)
}
#[derive(Default)]
pub(super) struct LineMatch {
    pub point: Point3,
    pub parameter: f64,
    pub tangent: Vector3,
    pub normal: Vector3,
}
pub(super) fn find_line(
    line: &ContapLine,
    surface: &Adaptor3dSurfaceHandle,
    uv: Point2,
    found: &mut LineMatch,
) -> Result<bool, Failure> {
    (found.point, found.normal) = ContapSurfProps::normale(surface, uv.x(), uv.y())?;
    let (parameter, point, tangent, normal_term) =
        project_analytic(line, found.point, found.normal)?;
    let distance = point.distance(&found.point) + normal_term.abs();
    let mut minimum_point = Point3::default();
    if distance < f64::MAX {
        found.parameter = parameter;
        found.tangent = tangent;
        minimum_point = point;
    }
    Ok(minimum_point.square_distance(&found.point) <= 1e-10)
}
fn project_analytic(
    line: &ContapLine,
    reference: Point3,
    normal: Vector3,
) -> Result<(f64, Point3, Vector3, f64), Failure> {
    if line.type_contour() == ContapIType::Lin {
        let geometry = line.line()?;
        let parameter = elclib::parameter(&geometry, reference)?;
        let (point, tangent) = elclib::d1(parameter, &geometry);
        return Ok((
            parameter,
            point,
            tangent,
            normal.dot(Vector3::from(geometry.direction().xyz())),
        ));
    }
    let geometry = line.circle()?;
    let parameter = elclib::parameter(&geometry, reference)?;
    let (point, tangent) = elclib::d1(parameter, &geometry);
    Ok((
        parameter,
        point,
        tangent,
        normal.dot(tangent / geometry.radius()),
    ))
}
