// Rust translation of IsTangentExtCheck, SQDistPointSurface, DistanceMinimizeByGradient,
// DistanceMinimizeByExtrema and HandleSingleSingularPoint of
// ModelingAlgorithms/TKGeomAlgo/IntWalk/IntWalk_PWalking.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The local searches of the boundary points: the tangency check of the start point,
//! the gradient and Newton minimizations of the distance between the surfaces, and
//! the singular point on a border.
use super::IntWalkPWalking;
use crate::int_imp::IntImpConstIsoparametric as Iso;
use crate::int_walk::IntWalkTheInt2S;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, real};
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg3d::adaptor3d::h_surface_tool::{self as tool, Adaptor3dSurfaceHandle};
use rustycad_tkgeombase::extrema::GenLocateExtPS;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Point3, Vector3};
use rustycad_tkmath::math::FunctionSetRoot;

/// `SQDistPointSurface`: the squared distance from the point to the surface by the
/// local extremum from `(u0, v0)`, RealLast when it fails.
fn sq_dist_point_surface(
    pnt: Point3,
    surf: &Adaptor3dSurfaceHandle,
    (u0, v0): (f64, f64),
) -> Result<f64, Failure> {
    let mut ext_ps =
        GenLocateExtPS::new(&**surf, precision::p_confusion(), precision::p_confusion());
    ext_ps.perform(pnt, u0, v0, false)?;
    if !ext_ps.is_done() {
        return Ok(real::real_last());
    }
    ext_ps.square_distance()
}

/// `IsTangentExtCheck`: whether the surfaces are tangent at the start point: their
/// normals are parallel (or null) and the four neighbours one step away on each
/// surface lie on the other surface within twice the tolerance.
pub(super) fn is_tangent_ext_check(
    surf1: &Adaptor3dSurfaceHandle,
    surf2: &Adaptor3dSurfaceHandle,
    [u10, v10, u20, v20]: [f64; 4],
    toler: f64,
    arr_step: [f64; 4],
) -> Result<bool, Failure> {
    let d1 = tool::d1(surf1, u10, v10)?;
    let d2 = tool::d1(surf2, u20, v20)?;
    let (n1, n2): (Vector3, Vector3) = (d1.d1u.crossed(d1.d1v), d2.d1u.crossed(d2.d1v));
    let (dp, sq1, sq2) = (n1.dot(n2), n1.square_magnitude(), n2.square_magnitude());
    if sq1 < real::real_small() || sq2 < real::real_small() {
        return Ok(true);
    }
    if dp * dp < 0.9998 * sq1 * sq2 {
        return Ok(false); // cos(N1, N2) < 0.9999
    }
    let sq_toler: f64 = 4.0 * toler * toler;
    let neighbours =
        |u: f64, v: f64, su: f64, sv: f64| [(u + su, v), (u - su, v), (u, v + sv), (u, v - sv)];
    for (u, v) in neighbours(u10, v10, arr_step[0], arr_step[1]) {
        if sq_dist_point_surface(tool::value(surf1, u, v)?, surf2, (u20, v20))? > sq_toler {
            return Ok(false);
        }
    }
    for (u, v) in neighbours(u20, v20, arr_step[2], arr_step[3]) {
        if sq_dist_point_surface(tool::value(surf2, u, v)?, surf1, (u10, v10))? > sq_toler {
            return Ok(false);
        }
    }
    Ok(true)
}

/// The gradient step of one parameter: at least its Epsilon (limited from below,
/// to avoid overflows near 0), in the direction of the gradient.
fn gradient_step(value: f64, add: f64, min_add: f64) -> f64 {
    value - std_max(add.abs(), min_add).copysign(add)
}

/// The gradient of the squared distance at the parameters `init`.
fn distance_gradient(
    surf1: &Adaptor3dSurfaceHandle,
    surf2: &Adaptor3dSurfaceHandle,
    init: &[f64; 4],
) -> Result<(f64, [f64; 4]), Failure> {
    let d1 = tool::d1(surf1, init[0], init[1])?;
    let d2 = tool::d1(surf2, init[2], init[3])?;
    let p12: Vector3 = Vector3::between(d1.point, d2.point);
    let grad: [f64; 4] = [
        -p12.dot(d1.d1u),
        -p12.dot(d1.d1v),
        p12.dot(d2.d1u),
        p12.dot(d2.d1v),
    ];
    Ok((d1.point.square_distance(&d2.point), grad))
}

impl IntWalkPWalking {
    /// `DistanceMinimizeByGradient(theASurf1, theASurf2, theInit, theStep0)`: descends
    /// the squared distance between the surfaces from `init` (u1 v1 u2 v2) on Bezier
    /// and B-spline surfaces only (true at once otherwise); the steps grow by 20% on
    /// success and restart after 60 failures. Returns whether the distance fell under
    /// 1e-14. `walk.distance_minimize_by_gradient(&s1, &s2, &mut init, None)?`.
    pub fn distance_minimize_by_gradient(
        &self,
        surf1: &Adaptor3dSurfaceHandle,
        surf2: &Adaptor3dSurfaceHandle,
        init: &mut [f64; 4],
        step0: Option<[f64; 4]>,
    ) -> Result<bool, Failure> {
        let tol_nul: f64 = 1.0 / precision::infinite();
        let min_add: [f64; 4] = init.map(|value| std_max(real::epsilon(value), tol_nul));
        let is_spline = |s: &Adaptor3dSurfaceHandle| -> Result<bool, Failure> {
            Ok(matches!(
                s.get_type()?,
                SurfaceType::BezierSurface | SurfaceType::BSplineSurface
            ))
        };
        if !is_spline(surf1)? || !is_spline(surf2)? {
            return Ok(true);
        }
        let (mut sq_dist_prev, mut grad) = distance_gradient(surf1, surf2, init)?;
        let initial_steps: [f64; 4] = step0.unwrap_or([1.0e-6; 4]);
        let mut steps: [f64; 4] = initial_steps;
        let (mut status, mut nb_iter) = (false, 60);
        loop {
            let par: [f64; 4] =
                [0, 1, 2, 3].map(|k| gradient_step(init[k], grad[k] * steps[k], min_add[k]));
            let p1 = tool::d1(surf1, par[0], par[1])?.point;
            let p2 = tool::d1(surf2, par[2], par[3])?.point;
            let sq_dist: f64 = p1.square_distance(&p2);
            if sq_dist < sq_dist_prev {
                sq_dist_prev = sq_dist;
                *init = par;
                status = sq_dist_prev < 1.0e-14;
                steps = steps.map(|step| step * 1.2);
                continue;
            }
            nb_iter -= 1;
            if nb_iter < 0 {
                return Ok(status);
            }
            grad = distance_gradient(surf1, surf2, init)?.1;
            steps = initial_steps;
        }
    }
    /// `DistanceMinimizeByExtrema(theASurf, theP0, theU0, theV0, theStep0)`: Newton
    /// iterations (at most 10) of the projection of `p0` on the surface from `(u0, v0)`,
    /// kept while the distance falls; returns whether it fell under 1e-14.
    /// `walk.distance_minimize_by_extrema(&s, p, &mut u, &mut v, None)?`.
    pub fn distance_minimize_by_extrema(
        &self,
        surf: &Adaptor3dSurfaceHandle,
        p0: Point3,
        (u0, v0): (&mut f64, &mut f64),
        step0: Option<[f64; 2]>,
    ) -> Result<bool, Failure> {
        let tol: f64 = 1.0e-14;
        let mut sq_dist_prev: f64 = real::real_last();
        let (mut u, mut v) = (*u0, *v0);
        let step: [f64; 2] = step0.unwrap_or([1.0, 1.0]);
        let mut nb_iter: i32 = 10;
        loop {
            let d2 = tool::d2(surf, u, v)?;
            let vec: Vector3 = Vector3::between(p0, d2.point);
            let sq_dist: f64 = vec.square_magnitude();
            if sq_dist >= sq_dist_prev {
                break;
            }
            sq_dist_prev = sq_dist;
            (*u0, *v0) = (u, v);
            nb_iter -= 1;
            if sq_dist_prev < tol {
                break;
            }
            let (f1, f2) = (d2.d1u.dot(vec), d2.d1v.dot(vec));
            // The mixed term reads D2U by D1V, as natively.
            let df1u: f64 = d2.d2u.dot(vec) + d2.d1u.dot(d2.d1u);
            let df1v: f64 = d2.d2u.dot(d2.d1v);
            let df2u: f64 = df1v;
            let df2v: f64 = d2.d2v.dot(vec) + d2.d1v.dot(d2.d1v);
            let det: f64 = df1u * df2v - df1v * df2u;
            u -= step[0] * (df2v * f1 - df1v * f2) / det;
            v += step[1] * (df2u * f1 - df1u * f2) / det;
            if nb_iter <= 0 {
                break;
            }
        }
        Ok(sq_dist_prev < tol)
    }
    /// `HandleSingleSingularPoint(theASurf1, theASurf2, the3DTol, thePnt)`: for each
    /// parameter on a border, the intersection point on that isoparametric (with the
    /// tolerance `tol3d`), snapped to the borders within `tol3d`; true when it is in
    /// the domain (`pnt` then holds it).
    pub fn handle_single_singular_point(
        &self,
        surf1: &Adaptor3dSurfaceHandle,
        surf2: &Adaptor3dSurfaceHandle,
        tol3d: f64,
        pnt: &mut [f64; 4],
    ) -> Result<bool, Failure> {
        let (low, upp) = domain_borders(surf1, surf2)?;
        let locked: [Iso; 4] = [
            Iso::UIsoparametricOnCaro1,
            Iso::VIsoparametricOnCaro1,
            Iso::UIsoparametricOnCaro2,
            Iso::VIsoparametricOnCaro2,
        ];
        let mut int = IntWalkTheInt2S::new(surf1, surf2, tol3d)?;
        let mut rsnld = FunctionSetRoot::without_tolerance(int.function(), 100)?;
        for i in 0..4 {
            if !((pnt[i] - low[i]).abs() < precision::p_confusion()
                || (pnt[i] - upp[i]).abs() < precision::p_confusion())
            {
                continue;
            }
            let mut param: Array1<'static, f64> = one_based(4)?;
            (param[1], param[2], param[3], param[4]) = (pnt[0], pnt[1], pnt[2], pnt[3]);
            int.perform_iso(&param, &mut rsnld, locked[i])?;
            if !int.is_done() || int.is_empty()? {
                continue;
            }
            let (a, b, c, d) = int.point()?.parameters();
            let mut pars: [f64; 4] = [a, b, c, d];
            for (k, surf) in [surf1, surf2].into_iter().enumerate() {
                snap_to_borders(surf, (2 * k, 2 * k + 1), &mut pars, (&low, &upp), tol3d)?;
            }
            *pnt = pars;
            // A NaN product stays in the domain, as the native `> 0` test.
            let in_domain: bool = (0..4).all(|j| {
                let product: f64 = (pnt[j] - low[j] + precision::p_confusion())
                    * (pnt[j] - upp[j] - precision::p_confusion());
                product <= 0.0 || product.is_nan()
            });
            if in_domain {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

/// The parameter bounds of both surfaces (adaptor members), in the order u1 v1 u2 v2.
pub(super) fn domain_borders(
    surf1: &Adaptor3dSurfaceHandle,
    surf2: &Adaptor3dSurfaceHandle,
) -> Result<([f64; 4], [f64; 4]), Failure> {
    let low: [f64; 4] = [
        surf1.first_u_parameter()?,
        surf1.first_v_parameter()?,
        surf2.first_u_parameter()?,
        surf2.first_v_parameter()?,
    ];
    let upp: [f64; 4] = [
        surf1.last_u_parameter()?,
        surf1.last_v_parameter()?,
        surf2.last_u_parameter()?,
        surf2.last_v_parameter()?,
    ];
    Ok((low, upp))
}

/// The parameters `(iu, iv)` of one surface snapped to a border when the local
/// resolution (`tol3d` over the derivative) reaches it and the border point is within
/// `tol3d` of the point.
fn snap_to_borders(
    surf: &Adaptor3dSurfaceHandle,
    (iu, iv): (usize, usize),
    pars: &mut [f64; 4],
    (low, upp): (&[f64; 4], &[f64; 4]),
    tol3d: f64,
) -> Result<(), Failure> {
    let tol2: f64 = tol3d * tol3d;
    let d1 = surf.d1(pars[iu], pars[iv])?;
    let p_int: Point3 = d1.point;
    let near = |uv: (f64, f64)| -> Result<bool, Failure> {
        Ok(p_int.square_distance(&surf.value(uv.0, uv.1)?) < tol2)
    };
    let modu: f64 = d1.d1u.magnitude();
    if modu > precision::confusion() {
        let tol_u: f64 = tol3d / modu;
        if (low[iu] - pars[iu]).abs() < tol_u {
            if near((low[iu], pars[iv]))? {
                pars[iu] = low[iu];
            }
        } else if (upp[iu] - pars[iu]).abs() < tol_u && near((upp[iu], pars[iv]))? {
            pars[iu] = upp[iu];
        }
    }
    let modv: f64 = d1.d1v.magnitude();
    if modv > precision::confusion() {
        let tol_v: f64 = tol3d / modv;
        if (low[iv] - pars[iv]).abs() < tol_v {
            if near((pars[iu], low[iv]))? {
                pars[iv] = low[iv];
            }
        } else if (upp[iv] - pars[iv]).abs() < tol_v && near((pars[iu], upp[iv]))? {
            pars[iv] = upp[iv];
        }
    }
    Ok(())
}
