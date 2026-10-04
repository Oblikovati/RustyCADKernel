//! TopOpeBRepTool_TOOL.cxx::FUN_ngF/Nt/minDUV at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use rustycad_tkbrep::{brep_adaptor::BRepAdaptorSurface, topods::Face};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::{adaptor3d::Adaptor3dSurface, top_abs::Orientation};
use rustycad_tkmath::{
    geom_abs::SurfaceType,
    gp::{Direction3, Point2, Vector3},
};

/// Smallest face parameter span, preserving native comparison order; `min_duv(&face)?`.
pub fn min_duv(face: &Face) -> Result<f64, Failure> {
    let surface = BRepAdaptorSurface::with_face(face, true)?;
    let du = surface.last_u_parameter()? - surface.first_u_parameter()?;
    let dv = surface.last_v_parameter()? - surface.first_v_parameter()?;
    Ok(if dv < du { dv } else { du })
}
/// Compute a geometric normal, retaining output on failure; `nggeom_f(uv,&face,&mut normal)?`.
pub fn nggeom_f(uv: Point2, face: &Face, normal: &mut Vector3) -> Result<bool, Failure> {
    let Some(value) = geometric_normal(uv, face)? else {
        return Ok(false);
    };
    *normal = value;
    Ok(true)
}
/// Compute an oriented unit normal, retaining output on failure; `nt(uv,&face,&mut direction)?`.
pub fn nt(uv: Point2, face: &Face, normal: &mut Direction3) -> Result<bool, Failure> {
    let Some(value) = geometric_normal(uv, face)? else {
        return Ok(false);
    };
    *normal = Direction3::from_xyz(*value.xyz())?;
    if face.orientation() == Orientation::Reversed {
        normal.reverse();
    }
    Ok(true)
}
fn geometric_normal(uv: Point2, face: &Face) -> Result<Option<Vector3>, Failure> {
    let surface = BRepAdaptorSurface::with_face(face, true)?;
    let tolerance = surface.tolerance()?;
    let tu = surface.u_resolution(tolerance)?;
    let tv = surface.v_resolution(tolerance)?;
    let derivatives = surface.d1(uv.x(), uv.y())?;
    let delta = min_duv(face)? * 0.1;
    let (du, dv) = (derivatives.d1u, derivatives.d1v);
    if du.magnitude() < tu || dv.magnitude() < tv {
        return singular_normal(&surface, uv, (tu, tv), du.magnitude(), delta);
    }
    let u = Direction3::from_xyz(*du.xyz())?;
    let v = Direction3::from_xyz(*dv.xyz())?;
    let normal = Direction3::from_xyz(*Vector3::from(u).crossed(Vector3::from(v)).xyz())?;
    Ok(Some(Vector3::from(normal)))
}
fn singular_normal(
    surface: &BRepAdaptorSurface,
    uv: Point2,
    tolerance: (f64, f64),
    du: f64,
    delta: f64,
) -> Result<Option<Vector3>, Failure> {
    match surface.get_type()? {
        SurfaceType::Cone => cone_normal(surface, uv, tolerance, du, delta),
        SurfaceType::Sphere => sphere_normal(surface, uv, tolerance.1),
        _ => Ok(None),
    }
}
fn cone_normal(
    surface: &BRepAdaptorSurface,
    uv: Point2,
    tolerance: (f64, f64),
    du: f64,
    delta: f64,
) -> Result<Option<Vector3>, Failure> {
    if uv.x().abs() < tolerance.0 && uv.y().abs() < tolerance.1 {
        return Ok(Some(
            Vector3::from(surface.cone()?.position().direction()).reversed(),
        ));
    }
    let collapsed_u = du < tolerance.0;
    if !collapsed_u {
        return Ok(None);
    }
    let mut first = surface.first_v_parameter()?;
    first += if (first - uv.y()).abs() < tolerance.0 {
        delta
    } else {
        -delta
    };
    let derivatives = surface.d1(uv.x(), first)?;
    Ok(Some(derivatives.d1u.crossed(derivatives.d1v)))
}
fn sphere_normal(
    surface: &BRepAdaptorSurface,
    uv: Point2,
    tolerance: f64,
) -> Result<Option<Vector3>, Failure> {
    let half_pi = std::f64::consts::FRAC_PI_2;
    let pole = (uv.y() - half_pi).abs() < tolerance || (uv.y() + half_pi).abs() < tolerance;
    if !pole {
        return Ok(None);
    }
    let origin = surface.sphere()?.location();
    let point = surface.value(uv.x(), uv.y())?;
    Ok(Some(Vector3::between(origin, point)))
}
