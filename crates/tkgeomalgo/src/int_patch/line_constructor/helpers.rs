// Rust translation of the static helpers Recadre, Parameters, LocalFirstParameter,
// LocalLastParameter, ComputeParametricTolerance, GetVertexTolerance and IsSegmentSmall of
// ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_LineConstructor.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The parameter helpers of `IntPatch_LineConstructor`: the periodic reframing, the
//! parameters of a 3d point on two quadrics, the bounds of a line and the parametric
//! tolerances.
use crate::int_patch::{
    IntPatchALine, IntPatchGLine, IntPatchIType, IntPatchLine, IntPatchLineTransitions,
    IntPatchPointLine, IntPatchRLine, IntPatchWLine,
};
use crate::int_surf::{IntSurfQuadric, IntSurfTypeTrans};
use crate::intf::{native_int_cast, std_max};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Point3, Vector3};
use std::f64::consts::PI;

/// The periodicities `Recadre` assumes for a surface type (the bi-parametric periodic
/// surfaces are handled upstream).
fn periodicity(typ: SurfaceType) -> (bool, bool) {
    match typ {
        SurfaceType::Cylinder | SurfaceType::Cone | SurfaceType::Sphere => (true, false),
        SurfaceType::Torus => (true, true),
        _ => (false, false),
    }
}

/// Brings `x` into `[f, l]` by turns of 2 Pi.
fn reframe(x: &mut f64, f: f64, l: f64) {
    let lmf: f64 = PI + PI;
    while *x < f {
        *x += lmf;
    }
    while *x > l {
        *x -= lmf;
    }
}

/// `Recadre`: brings the parameters of the periodic directions of the quadrics (the U
/// of a cylinder, a cone or a sphere, both of a torus) into the bounds of the surfaces.
pub(super) fn recadre(
    s1: &Adaptor3dSurfaceHandle,
    s2: &Adaptor3dSurfaceHandle,
    uv: &mut [f64; 4],
) -> Result<(), Failure> {
    let (u1_periodic, v1_periodic) = periodicity(s1.get_type()?);
    let (u2_periodic, v2_periodic) = periodicity(s2.get_type()?);
    if u1_periodic {
        reframe(&mut uv[0], s1.first_u_parameter()?, s1.last_u_parameter()?);
    }
    if v1_periodic {
        reframe(&mut uv[1], s1.first_v_parameter()?, s1.last_v_parameter()?);
    }
    if u2_periodic {
        reframe(&mut uv[2], s2.first_u_parameter()?, s2.last_u_parameter()?);
    }
    if v2_periodic {
        reframe(&mut uv[3], s2.first_v_parameter()?, s2.last_v_parameter()?);
    }
    Ok(())
}

/// The quadric of a surface adaptor; Standard_ConstructionError for another type.
fn quadric_of(s: &Adaptor3dSurfaceHandle) -> Result<IntSurfQuadric, Failure> {
    let mut quad = IntSurfQuadric::new();
    match s.get_type()? {
        SurfaceType::Plane => quad.set_plane(&s.plane()?),
        SurfaceType::Cylinder => quad.set_cylinder(&s.cylinder()?),
        SurfaceType::Cone => quad.set_cone(&s.cone()?),
        SurfaceType::Sphere => quad.set_sphere(&s.sphere()?),
        SurfaceType::Torus => quad.set_torus(&s.torus()?),
        other => {
            return Err(Failure::with_message(
                "Standard_ConstructionError",
                &format!(
                    "IntPatch_IntSS::MakeCurve - the surface type is {other:?}; expected a quadric"
                ),
            ));
        }
    }
    Ok(quad)
}

/// `Parameters`: the parameters of `ptref` on the two quadrics `[u1, v1, u2, v2]`.
pub(super) fn parameters(
    s1: &Adaptor3dSurfaceHandle,
    s2: &Adaptor3dSurfaceHandle,
    ptref: Point3,
) -> Result<[f64; 4], Failure> {
    let (quad1, quad2) = (quadric_of(s1)?, quadric_of(s2)?);
    let (u1, v1) = quad1.parameters(ptref).unwrap_or_default();
    let (u2, v2) = quad2.parameters(ptref).unwrap_or_default();
    Ok([u1, v1, u2, v2])
}

/// The first or last parameter of a conic without its point: infinite on the open
/// conics, 0 or 2 Pi on the closed ones.
fn conic_bound(typ: IntPatchIType, first: bool) -> f64 {
    match (typ, first) {
        (IntPatchIType::Lin | IntPatchIType::Parabola | IntPatchIType::Hyperbola, true) => {
            -precision::infinite()
        }
        (IntPatchIType::Lin | IntPatchIType::Parabola | IntPatchIType::Hyperbola, false) => {
            precision::infinite()
        }
        (IntPatchIType::Circle | IntPatchIType::Ellipse, true) => 0.0,
        (IntPatchIType::Circle | IntPatchIType::Ellipse, false) => PI + PI,
        _ => 0.0,
    }
}

/// The bound of an analytic line: its first (last) vertex, else the bound of its curve
/// moved inside by `Epsilon` when it is not included.
fn aline_bound(alin: &IntPatchALine, first: bool) -> Result<f64, Failure> {
    if first && alin.has_first_point() {
        return Ok(alin.first_point()?.parameter_on_line());
    }
    if !first && alin.has_last_point() {
        return Ok(alin.last_point()?.parameter_on_line());
    }
    let (mut bound, included) = match first {
        true => alin.first_parameter()?,
        false => alin.last_parameter()?,
    };
    if !included {
        bound += if first {
            epsilon(bound)
        } else {
            -epsilon(bound)
        };
    }
    Ok(bound)
}

/// `LocalFirstParameter` (`first`) and `LocalLastParameter`: the parameter of the first
/// (last) vertex of the line, else the bound of its geometry.
pub(super) fn local_parameter(l: &Handle<dyn IntPatchLine>, first: bool) -> Result<f64, Failure> {
    match l.arc_type() {
        IntPatchIType::Analytic => aline_bound(&l.cast::<IntPatchALine>(), first),
        IntPatchIType::Restriction => {
            let rlin: Handle<IntPatchRLine> = l.cast::<IntPatchRLine>();
            match (
                first,
                first && rlin.has_first_point(),
                !first && rlin.has_last_point(),
            ) {
                (true, true, _) => Ok(rlin.first_point()?.parameter_on_line()),
                (false, _, true) => Ok(rlin.last_point()?.parameter_on_line()),
                // A voir selon le type de la ligne 2d.
                _ => Ok(if first {
                    -precision::infinite()
                } else {
                    precision::infinite()
                }),
            }
        }
        IntPatchIType::Walking => {
            let wlin: Handle<IntPatchWLine> = l.cast::<IntPatchWLine>();
            match (
                first,
                first && wlin.has_first_point(),
                !first && wlin.has_last_point(),
            ) {
                (true, true, _) => Ok(wlin.first_point()?.parameter_on_line()),
                (false, _, true) => Ok(wlin.last_point()?.parameter_on_line()),
                _ => Ok(if first {
                    1.0
                } else {
                    f64::from(wlin.nb_pnts()?)
                }),
            }
        }
        typ => {
            let glin: Handle<IntPatchGLine> = l.cast::<IntPatchGLine>();
            match (
                first,
                first && glin.has_first_point(),
                !first && glin.has_last_point(),
            ) {
                (true, true, _) => Ok(glin.first_point()?.parameter_on_line()),
                (false, _, true) => Ok(glin.last_point()?.parameter_on_line()),
                _ => Ok(conic_bound(typ, first)),
            }
        }
    }
}

/// `ComputeParametricTolerance`: the larger of the parametric tolerances of `tol3d`
/// along both derivatives (0.1 along a null one).
pub(super) fn compute_parametric_tolerance(tol3d: f64, d1u: Vector3, d1v: Vector3) -> f64 {
    let nad1u: f64 = d1u.magnitude();
    let nad1v: f64 = d1v.magnitude();
    let tolu: f64 = if nad1u > 1e-12 { tol3d / nad1u } else { 0.1 };
    let tolv: f64 = if nad1v > 1e-12 { tol3d / nad1v } else { 0.1 };
    if tolu > tolv { tolu } else { tolv }
}

/// The parametric tolerance of `tol3d` at `(u, v)` of the surface.
pub(super) fn surface_tolerance(
    s: &Adaptor3dSurfaceHandle,
    tol3d: f64,
    u: f64,
    v: f64,
) -> Result<f64, Failure> {
    let d1 = s.d1(u, v)?;
    Ok(compute_parametric_tolerance(tol3d, d1.d1u, d1.d1v))
}

/// `IsSegmentSmall`: whether the walking line between the vertices `iv_first` and
/// `iv_last` is not longer than the larger of their tolerances (`GetVertexTolerance`).
pub(super) fn is_segment_small(
    wline: &IntPatchWLine,
    iv_first: i32,
    iv_last: i32,
) -> Result<bool, Failure> {
    let (vtx_f, vtx_l) = (wline.vertex(iv_first)?, wline.vertex(iv_last)?);
    let ip_f: i32 = native_int_cast(vtx_f.parameter_on_line());
    let ip_l: i32 = native_int_cast(vtx_l.parameter_on_line());
    if ip_f >= ip_l {
        return Ok(true);
    }
    let tol: f64 = std_max(vtx_f.tolerance(), vtx_l.tolerance());
    let mut len: f64 = 0.0;
    let mut p1: Point3 = wline.point(ip_f)?.value();
    for i in ip_f + 1..=ip_l {
        let p2: Point3 = wline.point(i)?.value();
        len += p1.distance(&p2);
        if len > tol {
            break;
        }
        p1 = p2;
    }
    Ok(len <= tol)
}

/// The transitions of a copy of `l`, as its three constructors take them: undecided,
/// TOUCH with the situations (Standard_DomainError when the second transition is not
/// TOUCH, as natively), or the IN/OUT types.
pub(super) fn transitions_of(l: &dyn IntPatchLine) -> Result<IntPatchLineTransitions, Failure> {
    Ok(match l.transition_on_s1() {
        IntSurfTypeTrans::Undecided => IntPatchLineTransitions::Undecided,
        IntSurfTypeTrans::Touch => {
            IntPatchLineTransitions::Touch(l.situation_s1()?, l.situation_s2()?)
        }
        t1 => IntPatchLineTransitions::Types(t1, l.transition_on_s2()),
    })
}
