// Rust translation of ModelingData/TKG3d/Geom/Geom_OffsetSurface.cxx (Surface) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom_OffsetSurface::Surface`: the elementary surface equivalent to an offset of
//! a plane or a quadric.
use crate::geom::{
    GeomConicalSurface, GeomCylindricalSurface, GeomElementarySurface, GeomGeometry, GeomPlane,
    GeomRectangularTrimmedSurface, GeomSphericalSurface, GeomSurface, GeomToroidalSurface,
};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Axis3, Vector3};

/// The equivalent of `basis` offset by `offset`: the basis itself (shared) for a
/// null offset; a translated plane; a cylinder, sphere or torus of changed radius
/// (the frame reversed when the radius changes sign); a cone moved along its axis;
/// wrapped in the trim of a trimmed basis; null when the result would be degenerate
/// or for other bases (`Surface`); `equivalent_surface(&basis, 2.0)?`.
pub fn equivalent_surface(
    basis: &Handle<dyn GeomSurface>,
    offset: f64,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    if offset == 0.0 {
        return Ok(basis.clone());
    }
    let trimmed: Handle<GeomRectangularTrimmedSurface> = basis.cast();
    let (base, trim) = match trimmed.get() {
        Some(trimmed) => (trimmed.basis_surface(), Some(trimmed.bounds())),
        None => (basis.clone(), None),
    };
    let result: Handle<dyn GeomSurface> = elementary_offset(&base, offset)?;
    match (trim, result.is_some()) {
        (Some((u1, u2, v1, v2)), true) => {
            let wrapped =
                GeomRectangularTrimmedSurface::new(&result, (u1, u2), (v1, v2), (true, true))?;
            Ok(Handle::new(wrapped).cast())
        }
        _ => Ok(result),
    }
}

// The sign of the offset direction relative to the outward normal of a quadric.
fn direct_sign(frame: &Axis3) -> Result<f64, Failure> {
    Ok(if frame.direct()? { 1.0 } else { -1.0 })
}

// The dispatch on the exact class of the (untrimmed) basis.
fn elementary_offset(
    base: &Handle<dyn GeomSurface>,
    offset: f64,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    if let Some(plane) = base.cast::<GeomPlane>().get() {
        let mut t = Vector3::from(plane.position().direction());
        t *= offset;
        return Ok(plane.translated(t)?.cast());
    }
    if let Some(cylinder) = base.cast::<GeomCylindricalSurface>().get() {
        return cylinder_offset(cylinder, offset);
    }
    if let Some(cone) = base.cast::<GeomConicalSurface>().get() {
        return cone_offset(cone, offset);
    }
    if let Some(sphere) = base.cast::<GeomSphericalSurface>().get() {
        return sphere_offset(sphere, offset);
    }
    if let Some(torus) = base.cast::<GeomToroidalSurface>().get() {
        return torus_offset(torus, offset);
    }
    Ok(Handle::default())
}

fn cylinder_offset(
    cylinder: &GeomCylindricalSurface,
    offset: f64,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    let tol: f64 = precision::confusion();
    let mut axis: Axis3 = cylinder.position();
    let radius: f64 = cylinder.radius() + direct_sign(&axis)? * offset;
    if radius >= tol {
        return Ok(Handle::new(GeomCylindricalSurface::new(axis, radius)?).cast());
    }
    if radius <= -tol {
        axis.x_reverse();
        return Ok(Handle::new(GeomCylindricalSurface::new(axis, -radius)?).cast());
    }
    Ok(Handle::default())
}

fn cone_offset(cone: &GeomConicalSurface, offset: f64) -> Result<Handle<dyn GeomSurface>, Failure> {
    let mut axis: Axis3 = cone.position();
    let sign: f64 = direct_sign(&axis)?;
    let alpha: f64 = cone.semi_angle();
    let (cos, sin) = (alpha.cos(), alpha.sin());
    let radius: f64 = cone.ref_radius() + sign * offset * cos;
    if radius < 0. {
        return Ok(Handle::default());
    }
    let mut z = Vector3::from(axis.direction());
    z *= -sign * offset * sin;
    axis.translate(z);
    Ok(Handle::new(GeomConicalSurface::new(axis, alpha, radius)?).cast())
}

fn sphere_offset(
    sphere: &GeomSphericalSurface,
    offset: f64,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    let tol: f64 = precision::confusion();
    let mut axis: Axis3 = sphere.position();
    let radius: f64 = sphere.radius() + direct_sign(&axis)? * offset;
    if radius >= tol {
        return Ok(Handle::new(GeomSphericalSurface::new(axis, radius)?).cast());
    }
    if radius <= -tol {
        axis.x_reverse();
        axis.z_reverse();
        return Ok(Handle::new(GeomSphericalSurface::new(axis, -radius)?).cast());
    }
    Ok(Handle::default())
}

fn torus_offset(
    torus: &GeomToroidalSurface,
    offset: f64,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    let tol: f64 = precision::confusion();
    let major: f64 = torus.major_radius();
    let mut axis: Axis3 = torus.position();
    let minor: f64 = torus.minor_radius() + direct_sign(&axis)? * offset;
    if minor >= tol && minor <= major {
        return Ok(Handle::new(GeomToroidalSurface::new(axis, major, minor)?).cast());
    }
    if minor <= -tol && -minor <= major {
        axis.x_reverse();
        return Ok(Handle::new(GeomToroidalSurface::new(axis, major, -minor)?).cast());
    }
    Ok(Handle::default())
}
