use super::*;
use rustycad_tkernel::precision;
use rustycad_tkmath::{elslib, geom_abs::SurfaceType};
use std::f64::consts::FRAC_PI_2;
fn finite_interval(mut first: f64, mut last: f64) -> [f64; 2] {
    if last < first {
        std::mem::swap(&mut first, &mut last);
    }
    match (
        precision::is_negative_infinite(first),
        precision::is_positive_infinite(last),
    ) {
        (true, true) => [-1e5, 1e5],
        (true, false) => [last - 2e5, last],
        (false, true) => [first, first + 2e5],
        _ => [first, last],
    }
}
pub(super) fn finite_bounds(surface: &dyn Adaptor3dSurface) -> Result<[f64; 4], Failure> {
    let u = finite_interval(surface.first_u_parameter()?, surface.last_u_parameter()?);
    let v = finite_interval(surface.first_v_parameter()?, surface.last_v_parameter()?);
    Ok([u[0], u[1], v[0], v[1]])
}
fn angular_count(radius: f64, span: f64) -> i32 {
    let angle = if radius > 1e-2 {
        (1.0 - 1e-2 / radius).acos() * 2.0
    } else {
        FRAC_PI_2
    };
    if angle > precision::angular() {
        (span / angle) as i32
    } else {
        0
    }
}
fn bounded_counts(counts: [i32; 2], minimum: i32) -> [i32; 2] {
    [counts[0].clamp(minimum, 50), counts[1].clamp(minimum, 50)]
}
fn cone_counts(surface: &dyn Adaptor3dSurface, b: [f64; 4]) -> Result<[i32; 2], Failure> {
    let cone = surface.cone()?;
    let first =
        elslib::cone_v_iso(cone.position(), cone.ref_radius(), cone.semi_angle(), b[2]).radius();
    let last =
        elslib::cone_v_iso(cone.position(), cone.ref_radius(), cone.semi_angle(), b[3]).radius();
    Ok(bounded_counts(
        [
            angular_count(first.max(last), b[1] - b[0]),
            ((b[3] - b[2]) as i32) / 10,
        ],
        10,
    ))
}
fn torus_radii(surface: &dyn Adaptor3dSurface, b: [f64; 4]) -> Result<[f64; 2], Failure> {
    let t = surface.torus()?;
    let u0 = elslib::torus_u_iso(t.position(), t.major_radius(), t.minor_radius(), b[0]).radius();
    let u1 = elslib::torus_u_iso(t.position(), t.major_radius(), t.minor_radius(), b[1]).radius();
    let v0 = elslib::torus_v_iso(t.position(), t.major_radius(), t.minor_radius(), b[2]).radius();
    let v1 = elslib::torus_v_iso(t.position(), t.major_radius(), t.minor_radius(), b[3]).radius();
    Ok([v0.max(v1), u0.max(u1)])
}
fn round_counts(
    surface: &dyn Adaptor3dSurface,
    b: [f64; 4],
    kind: SurfaceType,
) -> Result<[i32; 2], Failure> {
    let radii = if kind == SurfaceType::Sphere {
        [surface.sphere()?.radius(); 2]
    } else {
        torus_radii(surface, b)?
    };
    Ok(bounded_counts(
        [
            angular_count(radii[0], b[1] - b[0]),
            angular_count(radii[1], b[3] - b[2]),
        ],
        10,
    ))
}
fn initial_spline_counts(
    surface: &dyn Adaptor3dSurface,
    bspline: bool,
) -> Result<[i32; 2], Failure> {
    Ok(if bspline {
        [
            (surface.nb_u_knots()? * surface.u_degree()?).max(4),
            (surface.nb_v_knots()? * surface.v_degree()?).max(4),
        ]
    } else {
        [3 + surface.nb_u_poles()?, 3 + surface.nb_v_poles()?]
    })
}
fn spline_counts(
    surface: &dyn Adaptor3dSurface,
    b: [f64; 4],
    bspline: bool,
) -> Result<[i32; 2], Failure> {
    let mut counts = initial_spline_counts(surface, bspline)?;
    if counts[0] > 10 || counts[1] > 10 {
        let poles = if bspline {
            surface.bspline()?.poles()
        } else {
            surface.bezier()?.poles()
        };
        counts = super::analyse::pole_counts(&poles)?;
    }
    counts = [counts[0].max(10), counts[1].max(10)];
    if bspline {
        counts = anisotropic_counts(surface, b, counts)?;
    }
    Ok(counts)
}
fn anisotropic_counts(
    surface: &dyn Adaptor3dSurface,
    b: [f64; 4],
    mut counts: [i32; 2],
) -> Result<[i32; 2], Failure> {
    let ratio = ((b[1] - b[0]) / surface.u_resolution(1.0)?)
        / ((b[3] - b[2]) / surface.v_resolution(1.0)?);
    if ratio >= 10.0 {
        counts[0] = (counts[0] * 2).min(50);
    } else if ratio <= 0.1 {
        counts[1] = (counts[1] * 2).min(50);
    }
    Ok(counts)
}
pub(super) fn sample_counts(s: &dyn Adaptor3dSurface, b: [f64; 4]) -> Result<[i32; 2], Failure> {
    let kind = s.get_type()?;
    match kind {
        SurfaceType::Cylinder => Ok(bounded_counts(
            [
                angular_count(s.cylinder()?.radius(), b[1] - b[0]),
                ((b[3] - b[2]) as i32) / 10,
            ],
            2,
        )),
        SurfaceType::Cone => cone_counts(s, b),
        SurfaceType::Sphere | SurfaceType::Torus => round_counts(s, b, kind),
        SurfaceType::BezierSurface | SurfaceType::BSplineSurface => {
            spline_counts(s, b, kind == SurfaceType::BSplineSurface)
        }
        SurfaceType::SurfaceOfExtrusion => Ok([15, (((b[3] - b[2]) as i32) / 10).clamp(15, 50)]),
        SurfaceType::SurfaceOfRevolution => Ok([15, 15]),
        _ => Ok([10, 10]),
    }
}
