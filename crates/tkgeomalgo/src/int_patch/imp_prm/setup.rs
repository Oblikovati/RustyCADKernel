// Rust translation of GetLocalStep, IsSeamOrPole and Recadre of
// ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_ImpPrmIntersection.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The step of the walking, the special points between two points of a line, and the
//! periodic parameters of the end vertices.
use super::super::IntPatchSpecPntType;
use super::super::iwline::IntPatchTheIWLineOfTheIWalking;
use super::super::point::IntPatchPoint;
use crate::int_surf::IntSurfLineOn2S;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::std_min;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkmath::elslib;
use rustycad_tkmath::geom_abs::{Shape, SurfaceType};
use rustycad_tkmath::gp::Point3;
use std::f64::consts::{FRAC_PI_2, PI};

/// The smallest C1 interval of a C0 direction relative to its range, or None when the
/// direction has one interval.
fn min_interval(s: &Adaptor3dSurfaceHandle, is_u: bool) -> Result<Option<f64>, Failure> {
    let nb_int: i32 = if is_u {
        s.nb_u_intervals(Shape::C1)?
    } else {
        s.nb_v_intervals(Shape::C1)?
    };
    if nb_int <= 1 {
        return Ok(None);
    }
    let mut ints: Array1<'static, f64> =
        Array1::with_bounds(1, nb_int + 1).map_err(|e| e.into_failure("NCollection_Array1"))?;
    if is_u {
        s.u_intervals(&mut ints, Shape::C1)?;
    } else {
        s.v_intervals(&mut ints, Shape::C1)?;
    }
    let mut min_int: f64 = precision::infinite();
    for i in 1..=nb_int {
        min_int = std_min(min_int, ints[i + 1] - ints[i]);
    }
    let range: f64 = if is_u {
        s.last_u_parameter()? - s.first_u_parameter()?
    } else {
        s.last_v_parameter()? - s.first_v_parameter()?
    };
    Ok(Some(min_int / range))
}

/// `GetLocalStep(theSurf, theStep)`: 1e-4 on a Bezier or B-spline patch of degree above
/// 3 whose resolution is under 1e-10, or on a patch with a C1 interval under 0.2% of
/// its range; else the step.
pub(super) fn get_local_step(s: &Adaptor3dSurfaceHandle, step: f64) -> Result<f64, Failure> {
    let mut local_step: f64 = step;
    if s.u_continuity()? > Shape::C0 && s.v_continuity()? > Shape::C0 {
        let st: SurfaceType = s.get_type()?;
        if st == SurfaceType::BezierSurface || st == SurfaceType::BSplineSurface {
            let lim_res: f64 = 1.0e-10;
            let min_res: f64 = std_min(
                s.u_resolution(precision::confusion())?,
                s.v_resolution(precision::confusion())?,
            );
            let max_deg: i32 = std::cmp::max(s.u_degree()?, s.v_degree()?);
            if min_res < lim_res && max_deg > 3 {
                local_step = 0.0001;
            }
        }
    }
    for is_u in [true, false] {
        let c0 = if is_u {
            s.u_continuity()?
        } else {
            s.v_continuity()?
        } == Shape::C0;
        if c0 && min_interval(s, is_u)?.is_some_and(|m| m < 0.002) {
            local_step = 0.0001;
        }
    }
    Ok(std_min(step, local_step))
}

/// `IsSeamOrPole(theQSurf, theLine, IsReversed, theRefIndex, theTol3D, theDeltaMax)`:
/// the special point between the points `ref_index` and `ref_index + 1` of the line: the
/// apex of the cone or a pole of the sphere at the second point, else a jump of U (and
/// V on a torus) of at least `delta_max`.
pub(super) fn is_seam_or_pole(
    q_surf: &Adaptor3dSurfaceHandle,
    line: &IntSurfLineOn2S,
    is_reversed: bool,
    ref_index: i32,
    (tol_3d, delta_max): (f64, f64),
) -> Result<IntPatchSpecPntType, Failure> {
    if ref_index < 1 || ref_index >= line.nb_points() {
        return Ok(IntPatchSpecPntType::SPntNone);
    }
    let (pr, pn) = (line.value(ref_index)?, line.value(ref_index + 1)?);
    let p3d: Point3 = pn.value();
    let ((u_ref, v_ref), (u_next, v_next)) = if is_reversed {
        (pr.parameters_on_s2(), pn.parameters_on_s2())
    } else {
        (pr.parameters_on_s1(), pn.parameters_on_s1())
    };
    let t: SurfaceType = q_surf.get_type()?;
    if t == SurfaceType::Cone && q_surf.cone()?.apex().square_distance(&p3d) < tol_3d * tol_3d {
        return Ok(IntPatchSpecPntType::SPntPoleSeamU);
    }
    if t == SurfaceType::Sphere {
        let sphere = q_surf.sphere()?;
        let sq_tol: f64 = tol_3d * tol_3d;
        if elslib::value(0.0, FRAC_PI_2, &sphere).square_distance(&p3d) < sq_tol
            || elslib::value(0.0, -FRAC_PI_2, &sphere).square_distance(&p3d) < sq_tol
        {
            return Ok(IntPatchSpecPntType::SPntPoleSeamU);
        }
    }
    let delta_u: f64 = (u_ref - u_next).abs();
    if t != SurfaceType::Torus && delta_u < delta_max {
        return Ok(IntPatchSpecPntType::SPntNone);
    }
    Ok(match t {
        SurfaceType::Cylinder => IntPatchSpecPntType::SPntSeamU,
        SurfaceType::Torus => {
            let delta_v: f64 = (v_ref - v_next).abs();
            if delta_u >= delta_max && delta_v >= delta_max {
                IntPatchSpecPntType::SPntSeamUV
            } else if delta_u >= delta_max {
                IntPatchSpecPntType::SPntSeamU
            } else if delta_v >= delta_max {
                IntPatchSpecPntType::SPntSeamV
            } else {
                IntPatchSpecPntType::SPntNone
            }
        }
        SurfaceType::Sphere | SurfaceType::Cone => IntPatchSpecPntType::SPntPoleSeamU,
        _ => IntPatchSpecPntType::SPntNone,
    })
}

/// `x` within 1.5 PI of `ref` by turns of 2 PI.
fn near(mut x: f64, r: f64) -> f64 {
    while x < (r - 1.5 * PI) {
        x += PI + PI;
    }
    while x > (r + 1.5 * PI) {
        x -= PI + PI;
    }
    x
}

/// The periodic parameters of one surface near the reference ones: U on the cylinder,
/// cone and sphere, U and V on the torus.
fn reframe(t: SurfaceType, (u, v): (f64, f64), (up, vp): (f64, f64)) -> (f64, f64) {
    match t {
        SurfaceType::Torus => (near(u, up), near(v, vp)),
        SurfaceType::Cylinder | SurfaceType::Cone | SurfaceType::Sphere => (near(u, up), v),
        _ => (u, v),
    }
}

/// `Recadre(reversed, typeS1, typeS2, pt, iwline, Param, U1, V1, U2, V2)`: the
/// parameters of the vertex near those of the point `param` of the walking line.
pub(super) fn recadre(
    (type_s1, type_s2): (SurfaceType, SurfaceType),
    pt: &mut IntPatchPoint,
    iwline: &IntPatchTheIWLineOfTheIWalking,
    param: i32,
    [u1, v1, u2, v2]: [f64; 4],
) -> Result<(), Failure> {
    let (u1p, v1p, u2p, v2p) = iwline.line().value(param)?.parameters();
    let (u1, v1) = reframe(type_s1, (u1, v1), (u1p, v1p));
    let (u2, v2) = reframe(type_s2, (u2, v2), (u2p, v2p));
    pt.set_parameters(u1, v1, u2, v2);
    Ok(())
}
