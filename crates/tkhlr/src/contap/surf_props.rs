// Rust translation of ModelingAlgorithms/TKHLR/Contap/Contap_SurfProps.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Contap_SurfProps: the normal of a surface and its derivatives, in closed form on the
//! elementary surfaces. Null handles return `Standard_NullObject` instead of dereferencing null.
use super::geometry::vector_of_dir;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::real_epsilon;
use rustycad_tkg3d::adaptor3d::h_surface_tool::{self as hst, Adaptor3dSurfaceHandle};
use rustycad_tkmath::elslib;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Cone3, Point3, Vector3};

/// Contap_SurfProps: static queries. The normal is not unit: on a plane, a sphere, a
/// cylinder and a cone it is the unit normal (reversed on an indirect frame), on other
/// surfaces the cross product of the first derivatives.
///
/// ```
/// use rustycad_tkhlr::contap::ContapSurfProps;
/// assert_eq!(std::mem::size_of::<ContapSurfProps>(), 0);
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct ContapSurfProps;

/// The point, the first derivatives and the normal of DerivAndNorm.
pub type ContapDerivAndNorm = (Point3, Vector3, Vector3, Vector3);

/// The point, the normal and its derivatives of NormAndDn.
pub type ContapNormAndDn = (Point3, Vector3, Vector3, Vector3);

/// The unit normal of a cone at `u` on the nappe of `vcalc` (`-sina` on the side of
/// positive radius, `+sina` past the apex), reversed on an indirect cone.
fn cone_normal(co: &Cone3, u: f64, vcalc: f64) -> Result<Vector3, Failure> {
    let mut norm = cone_normal_unreversed(co, u, vcalc);
    if !co.direct()? {
        norm.reverse();
    }
    Ok(norm)
}

/// The V used for the normal of a cone at the apex (`|V sina + Rad| <= RealEpsilon`):
/// one unit away on the valid nappe of the surface.
fn cone_v_off_apex(s: &Adaptor3dSurfaceHandle, co: &Cone3, v: f64) -> Result<f64, Failure> {
    let angle = co.semi_angle();
    let (sina, rad) = (angle.sin(), co.ref_radius());
    if (v * sina + rad).abs() > real_epsilon() {
        return Ok(v);
    }
    let vfi = hst::first_v_parameter(s)?;
    Ok(if vfi < -rad / sina { v - 1. } else { v + 1. })
}

/// The unit normal of a sphere: the radius vector over the radius (reversed on an
/// indirect sphere).
fn sphere_normal(location: Point3, p: Point3, radius: f64, direct: bool) -> Vector3 {
    let norm = Vector3::between(location, p);
    if direct {
        norm / radius
    } else {
        norm / -radius
    }
}

/// The unit normal of a cylinder at `u` (reversed on an indirect cylinder).
fn cylinder_normal(x: Vector3, y: Vector3, u: f64, direct: bool) -> Vector3 {
    let mut norm = Vector3::default();
    norm.set_linear_form(u.cos(), x, u.sin(), y);
    if !direct {
        norm.reverse();
    }
    norm
}

impl ContapSurfProps {
    /// Normale(S, U, V, P, N): the point and the normal. At the apex of a cone
    /// (`|V sina + Rad| <= 1e-12`) the normal is null;
    /// `ContapSurfProps::normale(&s, 0.5, 0.5)?`.
    pub fn normale(
        s: &Adaptor3dSurfaceHandle,
        u: f64,
        v: f64,
    ) -> Result<(Point3, Vector3), Failure> {
        match checked_surface_kind(s)? {
            SurfaceType::Plane => Self::normale_plane(s, u, v),
            SurfaceType::Sphere => Self::normale_sphere(s, u, v),
            SurfaceType::Cylinder => Self::normale_cylinder(s, u, v),
            SurfaceType::Cone => Self::normale_cone(s, u, v),
            _ => Self::normale_generic(s, u, v),
        }
    }

    /// DerivAndNorm(S, U, V, P, d1u, d1v, N): the point, the first derivatives and the
    /// normal; at the apex of a cone the normal is taken one unit away on the valid
    /// nappe; `ContapSurfProps::deriv_and_norm(&s, 0.5, 0.5)?`.
    pub fn deriv_and_norm(
        s: &Adaptor3dSurfaceHandle,
        u: f64,
        v: f64,
    ) -> Result<ContapDerivAndNorm, Failure> {
        match checked_surface_kind(s)? {
            SurfaceType::Plane => Self::deriv_and_norm_plane(s, u, v),
            SurfaceType::Sphere => Self::deriv_and_norm_sphere(s, u, v),
            SurfaceType::Cylinder => Self::deriv_and_norm_cylinder(s, u, v),
            SurfaceType::Cone => Self::deriv_and_norm_cone(s, u, v),
            _ => Self::deriv_and_norm_generic(s, u, v),
        }
    }

    /// NormAndDn(S, U, V, P, N, Dnu, Dnv): the point, the normal and its derivatives in
    /// U and V (null derivatives on a plane, null in V on a cylinder and a cone);
    /// `ContapSurfProps::norm_and_dn(&s, 0.5, 0.5)?`.
    pub fn norm_and_dn(
        s: &Adaptor3dSurfaceHandle,
        u: f64,
        v: f64,
    ) -> Result<ContapNormAndDn, Failure> {
        match checked_surface_kind(s)? {
            SurfaceType::Plane => Self::norm_and_dn_plane(s, u, v),
            SurfaceType::Sphere => Self::norm_and_dn_sphere(s, u, v),
            SurfaceType::Cylinder => Self::norm_and_dn_cylinder(s, u, v),
            SurfaceType::Cone => Self::cone_norm_and_dn(s, u, v),
            _ => Self::norm_and_dn_generic(s, u, v),
        }
    }

    fn cone_norm_and_dn(
        s: &Adaptor3dSurfaceHandle,
        u: f64,
        v: f64,
    ) -> Result<ContapNormAndDn, Failure> {
        let co = hst::cone(s)?;
        let p = elslib::value(u, v, &co);
        let vcalc = cone_v_off_apex(s, &co, v)?;
        // The native normal is built before the cone is reversed, then both reverse.
        let mut norm = cone_normal_unreversed(&co, u, vcalc);
        let cosa = co.semi_angle().cos();
        let mut dnu = Vector3::default();
        dnu.set_linear_form(
            -cosa * u.sin(),
            vector_of_dir(co.x_axis().direction()),
            cosa * u.cos(),
            vector_of_dir(co.y_axis().direction()),
        );
        if !co.direct()? {
            norm.reverse();
            dnu.reverse();
        }
        Ok((p, norm, dnu, Vector3::new(0., 0., 0.)))
    }
}

/// The cone normal before the indirect reversal (NormAndDn reverses it with its
/// derivative).
fn cone_normal_unreversed(co: &Cone3, u: f64, vcalc: f64) -> Vector3 {
    let angle = co.semi_angle();
    let (sina, cosa, rad) = (angle.sin(), angle.cos(), co.ref_radius());
    let axial = if rad + vcalc * sina < 0. { sina } else { -sina };
    let mut norm = Vector3::default();
    norm.set_linear_form_three(
        [axial, cosa * u.cos(), cosa * u.sin()],
        [
            vector_of_dir(co.axis().direction()),
            vector_of_dir(co.x_axis().direction()),
            vector_of_dir(co.y_axis().direction()),
        ],
    );
    norm
}

mod deriv_and_norm;
mod norm_and_dn;
mod normale;

// Native calls dereference the handle; undefined null inputs get an explicit Rust error.
fn checked_surface_kind(s: &Adaptor3dSurfaceHandle) -> Result<SurfaceType, Failure> {
    if s.is_null() {
        return Err(Failure::with_message(
            "Standard_NullObject",
            "Contap_SurfProps: null surface; expected a live Adaptor3d_Surface",
        ));
    }
    hst::get_type(s)
}
