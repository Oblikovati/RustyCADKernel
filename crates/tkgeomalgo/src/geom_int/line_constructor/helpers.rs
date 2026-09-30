//! The quadric parameters of a point, their periodic adjustment and the points of the
//! conic lines.
use crate::geom_int::adjust_periodic::adjust_periodic as geom_int_adjust_periodic;
use crate::int_patch::{IntPatchGLine, IntPatchIType};
use crate::int_surf::IntSurfQuadric;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::Point3;
use std::f64::consts::PI;

/// The periodic directions of a quadric (`myHS1IsUPeriodic`, `myHS1IsVPeriodic`): U on
/// the cylinder, the cone and the sphere, both on the torus; the periodic
/// bi-parametric surfaces are processed upstream.
fn periodic_directions(surface: &GeomAdaptorSurface) -> Result<(bool, bool), Failure> {
    Ok(match surface.get_type()? {
        SurfaceType::Cylinder | SurfaceType::Cone | SurfaceType::Sphere => (true, false),
        SurfaceType::Torus => (true, true),
        _ => (false, false),
    })
}

/// `AdjustPeriodic(myHS1, myHS2, u1, v1, u2, v2)`: the periodic parameters of the
/// quadrics brought into the adaptor ranges by turns of 2 Pi (`GeomInt::AdjustPeriodic`
/// with no tolerance).
pub(super) fn adjust_periodic(
    hs1: &GeomAdaptorSurface,
    hs2: &GeomAdaptorSurface,
    uv: &mut [f64; 4],
) -> Result<(), Failure> {
    let lmf: f64 = PI + PI;
    for (k, hs) in [hs1, hs2].into_iter().enumerate() {
        let (u_periodic, v_periodic) = periodic_directions(hs)?;
        if u_periodic {
            let (f, l) = (hs.first_u_parameter()?, hs.last_u_parameter()?);
            uv[2 * k] = geom_int_adjust_periodic(uv[2 * k], f, l, lmf, 0.0).1;
        }
        if v_periodic {
            let (f, l) = (hs.first_v_parameter()?, hs.last_v_parameter()?);
            uv[2 * k + 1] = geom_int_adjust_periodic(uv[2 * k + 1], f, l, lmf, 0.0).1;
        }
    }
    Ok(())
}

/// `Parameters(myHS1, Ptref, U1, V1)`: the parameters of the point on the quadric of the
/// surface; Standard_ConstructionError for another surface.
/// Shared by the matching TKBO helper; `quadric_parameters(&surface, point, "IntTools")?`.
pub fn quadric_parameters(
    hs: &GeomAdaptorSurface,
    ptref: Point3,
    member: &str,
) -> Result<(f64, f64), Failure> {
    let quad: IntSurfQuadric = match hs.get_type()? {
        SurfaceType::Plane => IntSurfQuadric::from_plane(&hs.plane()?),
        SurfaceType::Cylinder => IntSurfQuadric::from_cylinder(&hs.cylinder()?),
        SurfaceType::Cone => IntSurfQuadric::from_cone(&hs.cone()?),
        SurfaceType::Sphere => IntSurfQuadric::from_sphere(&hs.sphere()?),
        SurfaceType::Torus => IntSurfQuadric::from_torus(&hs.torus()?),
        typ => {
            return Err(Failure::with_message(
                "Standard_ConstructionError",
                &format!(
                    "{member}: surface kind {typ:?}, expected a plane, cylinder, cone, sphere or torus"
                ),
            ));
        }
    };
    // Every quadric kind above is defined, so the parameters exist.
    Ok(quad.parameters(ptref).unwrap_or_default())
}

/// `Parameters(myHS1, myHS2, Ptref, U1, V1, U2, V2)`: the parameters on both quadrics.
pub(super) fn parameters(
    hs1: &GeomAdaptorSurface,
    hs2: &GeomAdaptorSurface,
    ptref: Point3,
) -> Result<[f64; 4], Failure> {
    const MEMBER: &str = "GeomInt_LineConstructor::Parameters";
    let (u1, v1) = quadric_parameters(hs1, ptref, MEMBER)?;
    let (u2, v2) = quadric_parameters(hs2, ptref, MEMBER)?;
    Ok([u1, v1, u2, v2])
}

/// `GLinePoint(typl, GLine, aT, aP)`: the point of the conic line at `t`;
/// Standard_ConstructionError for another line kind.
pub(super) fn gline_point(
    typl: IntPatchIType,
    gline: &IntPatchGLine,
    t: f64,
) -> Result<Point3, Failure> {
    Ok(match typl {
        IntPatchIType::Lin => elclib::value(t, &gline.line()?),
        IntPatchIType::Circle => elclib::value(t, &gline.circle()?),
        IntPatchIType::Ellipse => elclib::value(t, &gline.ellipse()?),
        IntPatchIType::Hyperbola => elclib::value(t, &gline.hyperbola()?),
        IntPatchIType::Parabola => elclib::value(t, &gline.parabola()?),
        typ => {
            return Err(Failure::with_message(
                "Standard_ConstructionError",
                &format!(
                    "GeomInt_LineConstructor::Parameters: line kind {typ:?}, expected a conic"
                ),
            ));
        }
    })
}

/// `RejectMicroCircle(aGLine, aType, aTol3D)`: whether the circle (or the major radius
/// of the ellipse) is smaller than the tolerance.
pub(super) fn reject_micro_circle(
    gline: &IntPatchGLine,
    typ: IntPatchIType,
    tol3d: f64,
) -> Result<bool, Failure> {
    Ok(match typ {
        IntPatchIType::Circle => gline.circle()?.radius() < tol3d,
        IntPatchIType::Ellipse => gline.ellipse()?.major_radius() < tol3d,
        _ => false,
    })
}
