// Rust translation of ModelingData/TKG3d/GeomAdaptor/GeomAdaptor_SurfaceOfLinearExtrusion.cxx (GetType, Plane, Cylinder) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The canonical kinds of an extrusion: a plane when a line, or a conic whose axis
//! is normal to the direction, is extruded; a cylinder for a circle whose axis is
//! parallel to it.
use crate::adaptor3d::Adaptor3dCurve;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::{Axis3, Cylinder3, Direction3, Plane3, Point3, Vector3};

/// `GetType`: the kind of the extrusion of `curve` along `direction`;
/// `extrusion_type(&*curve, dir)?`.
pub(super) fn extrusion_type(
    curve: &dyn Adaptor3dCurve,
    direction: Direction3,
) -> Result<SurfaceType, Failure> {
    let angular: f64 = precision::angular();
    let conic_axis: Direction3 = match curve.get_type()? {
        CurveType::Line => {
            let line_direction: Direction3 = curve.line()?.direction();
            if !direction.is_parallel(line_direction, angular) {
                return Ok(SurfaceType::Plane);
            }
            return Ok(SurfaceType::SurfaceOfExtrusion);
        }
        CurveType::Circle => {
            let axis: Direction3 = curve.circle()?.axis().direction();
            if direction.is_parallel(axis, angular) {
                return Ok(SurfaceType::Cylinder);
            }
            axis
        }
        CurveType::Ellipse => curve.ellipse()?.axis().direction(),
        CurveType::Parabola => curve.parabola()?.axis().direction(),
        CurveType::Hyperbola => curve.hyperbola()?.axis().direction(),
        _ => return Ok(SurfaceType::SurfaceOfExtrusion),
    };
    if direction.is_normal(conic_axis, angular) {
        return Ok(SurfaceType::Plane);
    }
    Ok(SurfaceType::SurfaceOfExtrusion)
}

/// `Plane`: the plane through a point of the curve whose normal is the first
/// non-null `T ^ D` of 21 samples of the tangent `T` over the curve range (an
/// infinite side is replaced by 200 units, a doubly infinite range by [-100, 100]),
/// the Y axis turned towards `D`; `extrusion_plane(&*curve, dir)?`.
pub(super) fn extrusion_plane(
    curve: &dyn Adaptor3dCurve,
    direction: Direction3,
) -> Result<Plane3, Failure> {
    let (first, last) = sampled_range(curve.first_parameter()?, curve.last_parameter()?);
    let delta_u: f64 = (last - first) / 20.;
    let (mut point, mut d1u, mut new_z) =
        (Point3::default(), Vector3::default(), Vector3::default());
    for i in 1..=21 {
        let parameter: f64 = first + f64::from(i - 1) * delta_u;
        (point, d1u) = curve.d1(parameter)?;
        new_z = d1u.normalized()?.crossed(Vector3::from(direction));
        if new_z.magnitude() > 1.0e-12 {
            break;
        }
    }
    let mut frame = Axis3::new(
        point,
        Direction3::from_vector(new_z)?,
        Direction3::from_vector(d1u)?,
    )?;
    if direction.dot(frame.y_direction()) < 0. {
        frame.y_reverse();
    }
    Ok(Plane3::from(frame))
}

fn sampled_range(first: f64, last: f64) -> (f64, f64) {
    let (negative, positive) = (
        precision::is_negative_infinite(first),
        precision::is_positive_infinite(last),
    );
    match (negative, positive) {
        (true, true) => (-100., 100.),
        (true, false) => (last - 200., last),
        (false, true) => (first, first + 200.),
        (false, false) => (first, last),
    }
}

/// `Cylinder`: the circle frame, its main direction turned towards `D`, and radius;
/// `extrusion_cylinder(&*curve, dir)?`.
pub(super) fn extrusion_cylinder(
    curve: &dyn Adaptor3dCurve,
    direction: Direction3,
) -> Result<Cylinder3, Failure> {
    let circle = curve.circle()?;
    let mut frame: Axis3 = Axis3::from_axis2(circle.position());
    if direction.dot(circle.axis().direction()) < 0. {
        frame.z_reverse();
    }
    Cylinder3::new(frame, circle.radius())
}
