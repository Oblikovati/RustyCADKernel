//! OCCT's process-wide sampled range and contour grid decisions.
use super::ContapHContTool;
use rustycad_tkernel::standard::{
    Failure,
    real::{real_first, real_last},
};
use rustycad_tkg3d::adaptor3d::{Adaptor3dSurface, h_surface_tool::Adaptor3dSurfaceHandle};
use rustycad_tkmath::geom_abs::SurfaceType;
use std::sync::{PoisonError, RwLock};
// Native file statics are observable across surfaces; retain them for compatibility.
static SAMPLED_RANGE: RwLock<[f64; 4]> = RwLock::new([0.0; 4]);
fn sampled_range() -> [f64; 4] {
    *SAMPLED_RANGE.read().unwrap_or_else(PoisonError::into_inner)
}
fn store_bound(index: usize, value: f64) {
    SAMPLED_RANGE
        .write()
        .unwrap_or_else(PoisonError::into_inner)[index] = value;
}
fn live_surface(surface: &Adaptor3dSurfaceHandle) -> Result<&dyn Adaptor3dSurface, Failure> {
    surface.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            "Contap_HContTool: null surface; expected live Adaptor3d_Surface",
        )
    })
}
fn capture_bounds(surface: &Adaptor3dSurfaceHandle) -> Result<[f64; 4], Failure> {
    let surface = live_surface(surface)?;
    // Store each result immediately: native failures retain already-written statics.
    store_bound(0, surface.first_u_parameter()?);
    store_bound(2, surface.last_u_parameter()?);
    store_bound(1, surface.first_v_parameter()?);
    store_bound(3, surface.last_v_parameter()?);
    let [u0, v0, u1, v1] = sampled_range();
    let (u0, u1) = finite_range(u0, u1);
    let (v0, v1) = finite_range(v0, v1);
    let bounds = [u0, v0, u1, v1];
    *SAMPLED_RANGE
        .write()
        .unwrap_or_else(PoisonError::into_inner) = bounds;
    Ok(bounds)
}
/// A range made finite for sampling: an infinite side is 2e5 from the other one, both
/// infinite sides give `[-1e5, 1e5]`.
fn finite_range(mut inf: f64, mut sup: f64) -> (f64, f64) {
    if sup < inf {
        (inf, sup) = (sup, inf);
    }
    if inf == real_first() && sup == real_last() {
        return (-1.0e5, 1.0e5);
    }
    if inf == real_first() {
        inf = sup - 2.0e5;
    } else if sup == real_last() {
        sup = inf + 2.0e5;
    }
    (inf, sup)
}

/// The B-spline sample count: knots times degree, at least 2.
fn spline_samples(nb_knots: i32, degree: i32) -> i32 {
    let nbs = nb_knots.wrapping_mul(degree);
    if nbs < 2 { 2 } else { nbs }
}

impl ContapHContTool {
    /// NbSamplesU(S, u1, u2): the U sample count by surface type (the range is not
    /// used); `ContapHContTool::nb_samples_u(&s, 0.0, 1.0)?`.
    pub fn nb_samples_u(s: &Adaptor3dSurfaceHandle, _u1: f64, _u2: f64) -> Result<i32, Failure> {
        Ok(match live_surface(s)?.get_type()? {
            SurfaceType::Plane => 2,
            SurfaceType::BezierSurface => 3 + s.nb_u_poles()?,
            SurfaceType::BSplineSurface => spline_samples(s.nb_u_knots()?, s.u_degree()?),
            SurfaceType::Torus => 20,
            _ => 10,
        })
    }

    /// NbSamplesV(S, v1, v2): the V sample count by surface type (the range is not
    /// used); `ContapHContTool::nb_samples_v(&s, 0.0, 1.0)?`.
    pub fn nb_samples_v(s: &Adaptor3dSurfaceHandle, _v1: f64, _v2: f64) -> Result<i32, Failure> {
        Ok(match live_surface(s)?.get_type()? {
            SurfaceType::Plane => 2,
            SurfaceType::BezierSurface => 3 + s.nb_v_poles()?,
            SurfaceType::BSplineSurface => spline_samples(s.nb_v_knots()?, s.v_degree()?),
            SurfaceType::Cylinder
            | SurfaceType::Cone
            | SurfaceType::Sphere
            | SurfaceType::Torus
            | SurfaceType::SurfaceOfRevolution
            | SurfaceType::SurfaceOfExtrusion => 15,
            _ => 10,
        })
    }

    /// NbSamplePoints(S): keeps the parametric range of `s` made finite (in the
    /// process-wide statics) and returns the number of sample points: 5, or for a
    /// B-spline the product of the thirds of the sample counts when it is larger;
    /// `ContapHContTool::nb_sample_points(&s)?`.
    pub fn nb_sample_points(s: &Adaptor3dSurfaceHandle) -> Result<i32, Failure> {
        let [uinf, vinf, usup, vsup] = capture_bounds(s)?;
        if s.get_type()? != SurfaceType::BSplineSurface {
            return Ok(5);
        }
        let m = (Self::nb_samples_u(s, uinf, usup)? / 3)
            .wrapping_mul(Self::nb_samples_v(s, vinf, vsup)? / 3);
        Ok(if m > 5 { m } else { 5 })
    }

    /// SamplePoint(S, Index, U, V): the sample point `index` of the range kept by the
    /// last `NbSamplePoints` (of any surface): a grid point of a B-spline with more
    /// than 5 cells, else one of five fixed points of the range;
    /// `ContapHContTool::sample_point(&s, 1)?`.
    pub fn sample_point(s: &Adaptor3dSurfaceHandle, index: i32) -> Result<(f64, f64), Failure> {
        let kind = live_surface(s)?.get_type()?;
        let [uinf, vinf, usup, vsup] = sampled_range();
        if kind == SurfaceType::BSplineSurface
            && let Some(point) = spline_grid_point(s, index, [uinf, vinf, usup, vsup])?
        {
            return Ok(point);
        }
        Ok(match index {
            1 => (0.75 * uinf + 0.25 * usup, 0.75 * vinf + 0.25 * vsup),
            2 => (0.75 * uinf + 0.25 * usup, 0.25 * vinf + 0.75 * vsup),
            3 => (0.25 * uinf + 0.75 * usup, 0.75 * vinf + 0.25 * vsup),
            4 => (0.25 * uinf + 0.75 * usup, 0.25 * vinf + 0.75 * vsup),
            _ => (0.5 * (uinf + usup), 0.5 * (vinf + vsup)),
        })
    }
}

fn spline_grid_point(
    surface: &Adaptor3dSurfaceHandle,
    index: i32,
    bounds: [f64; 4],
) -> Result<Option<(f64, f64)>, Failure> {
    let [uinf, vinf, usup, vsup] = bounds;
    let nu = ContapHContTool::nb_samples_u(surface, uinf, usup)? / 3;
    let nv = ContapHContTool::nb_samples_v(surface, vinf, vsup)? / 3;
    if nu.wrapping_mul(nv) <= 5 {
        return Ok(None);
    }
    let iu = index.wrapping_sub(1) / nu;
    let iv = index.wrapping_sub(1) - iu * nu;
    let u = uinf + ((usup - uinf) / f64::from(nu + 1)) * f64::from(iu + 1);
    let v = vinf + ((vsup - vinf) / f64::from(nv + 2)) * f64::from(iv + 1);
    Ok(Some((u, v)))
}
