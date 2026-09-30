// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_SpecialPoints.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntPatch_SpecialPoints`: the points of an intersection line that the walking misses
//! (a pole of a sphere, the apex of a cone, the crossing of a seam), docs/geomalgo-design.md
//! section 17.
mod pole;
mod seam;

use super::IntPatchSpecPntType;
use super::point::IntPatchPoint;
use crate::int_surf::{IntSurfPntOn2S, set_period};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkgeombase::extrema::GenLocateExtPS;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Point3, Transform3};
use rustycad_tkmath::math::{FunctionSetRoot, Vector};
use seam::{FuncPreciseSeam, is_point_on_surface};
use std::f64::consts::{FRAC_PI_2, PI};

/// `IntPatch_SpecialPoints`: static members only. `theQSurf` is the quadric that may
/// hold the special point, `thePSurf` the other surface; `is_reversed` says the quadric
/// is the second surface of the points.
///
/// ```ignore
/// let mut added = IntSurfPntOn2S::new();
/// let found = IntPatchSpecialPoints::add_cross_uv_iso_point(&q, &p, &ref_pt, 1e-7, &mut added, false)?;
/// ```
pub struct IntPatchSpecialPoints;

/// The periods `[U1, V1, U2, V2]` of the points: the quadric is S1 unless reversed.
fn periods_of(
    q_surf: &Adaptor3dSurfaceHandle,
    p_surf: &Adaptor3dSurfaceHandle,
    is_reversed: bool,
) -> Result<[f64; 4], Failure> {
    if is_reversed {
        return set_period(&**p_surf, &**q_surf);
    }
    set_period(&**q_surf, &**p_surf)
}

/// `SetValue(0.5 * (P0 + PQuad), ...)` with the quadric parameters first unless reversed.
fn set_middle(
    point: &mut IntSurfPntOn2S,
    (p0, p_quad): (Point3, Point3),
    (u0, v0): (f64, f64),
    (u_quad, v_quad): (f64, f64),
    is_reversed: bool,
) {
    let mid = Point3::from(0.5 * (p0.xyz() + p_quad.xyz()));
    if is_reversed {
        point.set_value_all(mid, u0, v0, u_quad, v_quad);
    } else {
        point.set_value_all(mid, u_quad, v_quad, u0, v0);
    }
}

impl IntPatchSpecialPoints {
    /// `AddCrossUVIsoPoint(theQSurf, thePSurf, theRefPt, theTol3d, theAddedPoint,
    /// theIsReversed)`: projects the quadric point `(0, 0)` on the other surface from the
    /// parameters of `ref_pt` there; when nearer than `tol_3d`, writes the middle point
    /// adjusted to the periods of `ref_pt` into `added_point` and answers true.
    /// `IntPatchSpecialPoints::add_cross_uv_iso_point(&q, &p, &r, 1e-7, &mut out, false)?`.
    pub fn add_cross_uv_iso_point(
        q_surf: &Adaptor3dSurfaceHandle,
        p_surf: &Adaptor3dSurfaceHandle,
        ref_pt: &IntSurfPntOn2S,
        tol_3d: f64,
        added_point: &mut IntSurfPntOn2S,
        is_reversed: bool,
    ) -> Result<bool, Failure> {
        let periods: [f64; 4] = periods_of(q_surf, p_surf, is_reversed)?;
        // Not quadric point
        let (u0, v0) = if is_reversed {
            ref_pt.parameters_on_s1()
        } else {
            ref_pt.parameters_on_s2()
        };
        // Quadric point
        let (u_quad, v_quad) = (0.0, 0.0);
        let p_quad: Point3 = q_surf.d0(u_quad, v_quad)?;
        let conf: f64 = precision::p_confusion();
        let mut extr = GenLocateExtPS::new(&**p_surf, conf, conf);
        extr.perform(p_quad, u0, v0, false)?;
        if !extr.is_done() || extr.square_distance()? > tol_3d * tol_3d {
            return Ok(false);
        }
        let found = extr.point()?;
        let p0: Point3 = *found.value();
        let uv0 = found.parameter();
        set_middle(
            added_point,
            (p0, p_quad),
            uv0,
            (u_quad, v_quad),
            is_reversed,
        );
        Self::adjust_point_and_vertex(ref_pt, &periods, added_point, None);
        Ok(true)
    }

    /// `AddPointOnUorVIso(theQSurf, thePSurf, theRefPt, theIsU, theIsoParameter,
    /// theToler, theInitPoint, theInfBound, theSupBound, theAddedPoint, theIsReversed)`:
    /// solves (math_FunctionSetRoot, 100 iterations) for the point of the other surface
    /// on the quadric iso `U = iso` (`theIsU`) or `V = iso`; the unknowns are the
    /// parameters on the other surface and the free quadric parameter. Writes the
    /// middle point adjusted to `ref_pt` and answers true when the solver is done.
    /// `IntPatchSpecialPoints::add_point_on_u_or_v_iso(&q, &p, &r, true, 0.0, [&tol, &init, &inf, &sup], &mut out, false)?`.
    pub fn add_point_on_u_or_v_iso(
        q_surf: &Adaptor3dSurfaceHandle,
        p_surf: &Adaptor3dSurfaceHandle,
        ref_pt: &IntSurfPntOn2S,
        (is_u, iso_parameter): (bool, f64),
        [toler, init_point, inf_bound, sup_bound]: [&Vector<'_>; 4],
        added_point: &mut IntSurfPntOn2S,
        is_reversed: bool,
    ) -> Result<bool, Failure> {
        let periods: [f64; 4] = periods_of(q_surf, p_surf, is_reversed)?;
        let mut f = FuncPreciseSeam::new(&**q_surf, &**p_surf, is_u, iso_parameter);
        let mut srf = FunctionSetRoot::new(&f, toler, 100)?;
        srf.perform_bounded(&mut f, init_point, inf_bound, sup_bound, false)?;
        if !srf.is_done() {
            return Ok(false);
        }
        let mut roots = Vector::with_bounds(init_point.lower(), init_point.upper())?;
        srf.copy_root(&mut roots)?;
        // On parametric (the native code reads the indices 1 to 3 whatever the bounds)
        let (u0, v0) = (*roots.value(1)?, *roots.value(2)?);
        // On quadric
        let root3: f64 = *roots.value(3)?;
        let u_quad: f64 = if is_u { 0.0 } else { root3 };
        let v_quad: f64 = if is_u { root3 } else { 0.0 };
        let p_quad: Point3 = q_surf.value(u_quad, v_quad)?;
        let p0: Point3 = p_surf.value(u0, v0)?;
        set_middle(
            added_point,
            (p0, p_quad),
            (u0, v0),
            (u_quad, v_quad),
            is_reversed,
        );
        Self::adjust_point_and_vertex(ref_pt, &periods, added_point, None);
        Ok(true)
    }

    /// The pole of the quadric, `(Uquad, Vquad)` with `Uquad = 0`: Standard_TypeMismatch
    /// unless the quadric is a sphere (V of the sign of `v_quad`) or a cone (its apex).
    fn pole_parameters(q_surf: &Adaptor3dSurfaceHandle, v_quad: f64) -> Result<f64, Failure> {
        match q_surf.get_type()? {
            SurfaceType::Sphere => Ok(FRAC_PI_2.copysign(v_quad)),
            SurfaceType::Cone => {
                let co = q_surf.cone()?;
                Ok(-co.ref_radius() / co.semi_angle().sin())
            }
            _ => Err(Failure::with_message(
                "Standard_TypeMismatch",
                "IntPatch_SpecialPoints::AddSingularPole(),Unsupported quadric with Pole",
            )),
        }
    }

    /// `AddSingularPole(theQSurf, thePSurf, thePtIso, theVertex, theAddedPoint,
    /// theIsReversed, theIsReqRefCheck)`: when the pole of the quadric (a sphere or a
    /// cone, Standard_TypeMismatch otherwise) lies on the other surface within the
    /// vertex tolerance (and, with `is_req_ref_check`, near the vertex), writes it with
    /// its quadric U parameter (`ProcessSphere`/`ProcessCone`) into `added_point`,
    /// updates the vertex when it is the same point or when the iso was chosen, and
    /// answers true.
    /// `IntPatchSpecialPoints::add_singular_pole(&q, &p, &iso, &mut vtx, &mut out, false, false)?`.
    pub fn add_singular_pole(
        q_surf: &Adaptor3dSurfaceHandle,
        p_surf: &Adaptor3dSurfaceHandle,
        pt_iso: &IntSurfPntOn2S,
        vertex: &mut IntPatchPoint,
        added_point: &mut IntSurfPntOn2S,
        is_reversed: bool,
        is_req_ref_check: bool,
    ) -> Result<bool, Failure> {
        let (a, b, c, d) = vertex.parameters();
        let ((u0, v0), v_quad) = if is_reversed {
            ((a, b), d)
        } else {
            ((c, d), b)
        };
        let u_quad: f64 = 0.0;
        let v_quad: f64 = Self::pole_parameters(q_surf, v_quad)?;
        let p_quad: Point3 = q_surf.d0(u_quad, v_quad)?;
        let tol: f64 = vertex.tolerance();
        if is_req_ref_check && p_quad.square_distance(&vertex.value()) >= tol * tol {
            return Ok(false);
        }
        let Some((p0, u0, v0)) = is_point_on_surface(&**p_surf, p_quad, tol, (u0, v0))? else {
            return Ok(false);
        };
        // Pole is an intersection point (lies in the quadric and the parametric surface)
        set_middle(
            added_point,
            (p0, p_quad),
            (u0, v0),
            (u_quad, v_quad),
            is_reversed,
        );
        let is_same: bool = added_point.is_same(&vertex.pnt_on_2s(), precision::confusion(), -1.0);
        let pole = PoleSolve { u0, v0, v_quad };
        let (u_quad, is_iso) = Self::pole_u_parameter(q_surf, p_surf, pt_iso, &pole, is_reversed)?;
        set_middle(
            added_point,
            (p0, p_quad),
            (u0, v0),
            (u_quad, v_quad),
            is_reversed,
        );
        if is_same || is_iso {
            vertex.set_pnt_on_2s(added_point);
            return Ok(true);
        }
        let periods: [f64; 4] = periods_of(q_surf, p_surf, is_reversed)?;
        Self::adjust_point_and_vertex(&vertex.pnt_on_2s(), &periods, added_point, None);
        Ok(true)
    }

    /// The U parameter of the quadric in the pole: the derivatives of the other surface
    /// in the frame of the sphere or cone go to `ProcessSphere` or `ProcessCone`.
    fn pole_u_parameter(
        q_surf: &Adaptor3dSurfaceHandle,
        p_surf: &Adaptor3dSurfaceHandle,
        pt_iso: &IntSurfPntOn2S,
        pole: &PoleSolve,
        is_reversed: bool,
    ) -> Result<(f64, bool), Failure> {
        let d1 = p_surf.d1(pole.u0, pole.v0)?;
        let (mut vec_du, mut vec_dv) = (d1.d1u, d1.d1v);
        let is_sphere: bool = q_surf.get_type()? == SurfaceType::Sphere;
        // Transforms the parametric surface in the coordinate system of the quadric.
        let position = if is_sphere {
            q_surf.sphere()?.position()
        } else {
            q_surf.cone()?.position()
        };
        let tr = Transform3::transformation_to(position);
        vec_du.transform(tr);
        vec_dv.transform(tr);
        if is_sphere {
            return Self::process_sphere(pt_iso, vec_du, vec_dv, is_reversed, pole.v_quad);
        }
        Self::process_cone(pt_iso, vec_du, vec_dv, &q_surf.cone()?, is_reversed, 0.0)
    }

    /// `ContinueAfterSpecialPoint(theQSurf, thePSurf, theRefPt, theSPType, theTol2D,
    /// theNewPoint, theIsReversed)`: when the walking line broke in a special point,
    /// recomputes the quadric U parameter of the apex of a cone (the other generatrix may
    /// be tangent), then adjusts the new point to the reference one with the period of
    /// the quadric (PI/2 through a pole, 2*PI otherwise). False for no special point or
    /// when the new point is the reference point.
    /// `IntPatchSpecialPoints::continue_after_special_point(&q, &p, &r, t, 1e-9, &mut np, false)?`.
    pub fn continue_after_special_point(
        q_surf: &Adaptor3dSurfaceHandle,
        p_surf: &Adaptor3dSurfaceHandle,
        ref_pt: &IntSurfPntOn2S,
        sp_type: IntPatchSpecPntType,
        tol_2d: f64,
        new_point: &mut IntSurfPntOn2S,
        is_reversed: bool,
    ) -> Result<bool, Failure> {
        if sp_type == IntPatchSpecPntType::SPntNone {
            return Ok(false);
        }
        if new_point.is_same(ref_pt, precision::confusion(), tol_2d) {
            return Ok(false);
        }
        if sp_type == IntPatchSpecPntType::SPntPole && q_surf.get_type()? == SurfaceType::Cone {
            Self::recompute_apex_u(q_surf, p_surf, ref_pt, new_point, is_reversed)?;
        }
        let period: f64 = if sp_type == IntPatchSpecPntType::SPntPole {
            FRAC_PI_2
        } else {
            2.0 * PI
        };
        let periods = Self::continue_periods(q_surf, p_surf, period, is_reversed)?;
        Self::adjust_point_and_vertex(ref_pt, &periods, new_point, None);
        Ok(true)
    }

    /// Condition b) of `ContinueAfterSpecialPoint`: the U parameter of the cone apex from
    /// the generatrix nearest to the reference point.
    fn recompute_apex_u(
        q_surf: &Adaptor3dSurfaceHandle,
        p_surf: &Adaptor3dSurfaceHandle,
        ref_pt: &IntSurfPntOn2S,
        new_point: &mut IntSurfPntOn2S,
        is_reversed: bool,
    ) -> Result<(), Failure> {
        let (a, b, c, d) = new_point.parameters();
        let ((u0, v0), (u_quad, v_quad)) = if is_reversed {
            ((a, b), (c, d))
        } else {
            ((c, d), (a, b))
        };
        let d1 = p_surf.d1(u0, v0)?;
        let (mut vec_du, mut vec_dv) = (d1.d1u, d1.d1v);
        let cone = q_surf.cone()?;
        let tr = Transform3::transformation_to(cone.position());
        vec_du.transform(tr);
        vec_dv.transform(tr);
        let (u_quad, _) = Self::process_cone(ref_pt, vec_du, vec_dv, &cone, is_reversed, u_quad)?;
        new_point.set_parameters_on(!is_reversed, u_quad, v_quad);
        Ok(())
    }

    /// The periods of `ContinueAfterSpecialPoint`: the quadric ones replaced by `period`.
    fn continue_periods(
        q_surf: &Adaptor3dSurfaceHandle,
        p_surf: &Adaptor3dSurfaceHandle,
        period: f64,
        is_reversed: bool,
    ) -> Result<[f64; 4], Failure> {
        let up: f64 = if p_surf.is_u_periodic()? {
            p_surf.u_period()?
        } else {
            0.0
        };
        let uq: f64 = if q_surf.is_u_periodic()? { period } else { 0.0 };
        let vp: f64 = if p_surf.is_v_periodic()? {
            p_surf.v_period()?
        } else {
            0.0
        };
        let vq: f64 = if q_surf.is_v_periodic()? { period } else { 0.0 };
        if is_reversed {
            return Ok([up, vp, uq, vq]);
        }
        Ok([uq, vq, up, vp])
    }

    /// `AdjustPointAndVertex(theRefPoint, theArrPeriods, theNewPoint, theVertex)`:
    /// shifts each periodic parameter of `new_point` by whole periods until it is within
    /// half a period of the reference one; writes the parameters into the vertex too.
    /// `IntPatchSpecialPoints::adjust_point_and_vertex(&r, &[2.0 * PI, 0.0, 0.0, 0.0], &mut p, None)`.
    pub fn adjust_point_and_vertex(
        ref_point: &IntSurfPntOn2S,
        arr_periods: &[f64; 4],
        new_point: &mut IntSurfPntOn2S,
        vertex: Option<&mut IntPatchPoint>,
    ) {
        let (a, b, c, d) = new_point.parameters();
        let mut par: [f64; 4] = [a, b, c, d];
        for i in 0..4 {
            if arr_periods[i] == 0.0 {
                continue;
            }
            let ref_par = if i < 2 {
                ref_point.parameters_on_s1()
            } else {
                ref_point.parameters_on_s2()
            };
            let ref_value: f64 = [ref_par.0, ref_par.1][i % 2];
            par[i] = shift_by_periods(par[i], ref_value, arr_periods[i]);
        }
        if let Some(vertex) = vertex {
            vertex.set_parameters(par[0], par[1], par[2], par[3]);
        }
        new_point.set_parameters(par[0], par[1], par[2], par[3]);
    }
}

/// The inner loop of `AdjustPointAndVertex`.
fn shift_by_periods(mut par: f64, ref_value: f64, period: f64) -> f64 {
    let half_period: f64 = 0.5 * period;
    let mut delta: f64 = ref_value - par;
    let incr: f64 = period.copysign(delta);
    while delta > half_period || delta < -half_period {
        par += incr;
        delta = ref_value - par;
    }
    par
}

/// The pole and the parameters found by `AddSingularPole`.
struct PoleSolve {
    u0: f64,
    v0: f64,
    v_quad: f64,
}
