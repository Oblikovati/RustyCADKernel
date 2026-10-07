// Rust translation of ChFiKPart_CornerSpine (ChFiKPart_ComputeData_CS.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The guide circle and torus axis of a spherical corner between a plane and a surface.
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Axis2, Axis3, Circle3, Cylinder3, Direction3, Point2, Point3, Vector3};

/// `ChFiKPart_CornerSpine(S1, S2, P1S1, P2S1, P1S2, P2S2, R, cyl, circ, First, Last)`:
/// `(cyl, circ, First, Last)`, the circle of radius `R` from the first point on the
/// second surface toward its second point, centred off the first point along the plane
/// `S1` direction to the first point on it, and the cylinder of the same axis oriented
/// as the normal of `S2`; `First` is 0 and `Last` the parameter of the second point;
/// `P2S1` is not used natively and is not taken;
/// `corner_spine((&s1, &s2), (p1s1, p1s2, p2s2), 1.0)?`.
pub fn corner_spine(
    (s1, s2): (&Handle<dyn Adaptor3dSurface>, &Handle<dyn Adaptor3dSurface>),
    (p1s1, p1s2, p2s2): (Point2, Point2, Point2),
    r: f64,
) -> Result<(Cylinder3, Circle3, f64, f64), Failure> {
    let ax = s1.plane()?.position();
    let (v1, v2) = (
        Vector3::from(ax.x_direction()),
        Vector3::from(ax.y_direction()),
    );
    let d1 = s2.d1(p1s2.x(), p1s2.y())?;
    let p = d1.point;
    let mut v = Vector3::between(p, s1.value(p1s1.x(), p1s1.y())?);
    v = v1 * v.dot(v1) + v2 * v.dot(v2);
    v.normalize()?;
    let p2 = s2.value(p2s2.x(), p2s2.y())?;
    let vorien = Vector3::between(p, p2);
    let mut dx = Direction3::from_vector(v)?;
    let cent = if v.dot(vorien) >= 0.0 {
        dx.reverse();
        Point3::new(p.x() + r * v.x(), p.y() + r * v.y(), p.z() + r * v.z())
    } else {
        Point3::new(p.x() - r * v.x(), p.y() - r * v.y(), p.z() - r * v.z())
    };
    let dy = Direction3::from_vector(Vector3::between(cent, p2))?;
    let dy = dx.crossed(dy)?.crossed(dx)?;
    let circax2 = Axis2::with_x_direction(cent, dx.crossed(dy)?, dx)?;
    let mut cylax3 = Axis3::from_axis2(circax2);
    if d1.d1u.crossed(d1.d1v).dot(Vector3::from(dx)) < 0.0 {
        cylax3.z_reverse();
    }
    let last = elclib::circle_parameter(circax2, p2);
    Ok((
        Cylinder3::new(cylax3, r)?,
        Circle3::new(circax2, r)?,
        0.0,
        last,
    ))
}
