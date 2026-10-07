//! The common steps of `ComputeValues`, the obligatory passage of every computation: the
//! cache test, the processing of `t` and of `X`, then the equations and their derivatives
//! up to `Order`.
use super::super::guide::GuideScratch;
use super::super::linalg::{copy_vector, vget};
use super::super::normal_terms::{EPS, SurfaceJet};
use super::super::package::{compute_d_normal, compute_normal, live_surface};
use super::{Radii, SurfacePoint, TwoSurfaceCore};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkmath::gp::{Point2, Vector3};
use rustycad_tkmath::math::Vector;

/// The native `static thread_local` scratch of `ComputeValues`: the guide values and the
/// third derivatives of both surfaces. Each class keeps its own, shared by its objects.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct SurfaceScratch {
    pub guide: GuideScratch,
    pub d3: [[Vector3; 4]; 2],
}

/// What the cache test of `ComputeValues` decided.
pub(crate) struct CacheState {
    /// The guide parameter of the computation (`Param`, or the field `param`).
    pub t: f64,
    /// Whether `X` is already processed to the order (`myX_OK`).
    pub x_ok: bool,
    /// Whether `t` is already processed (`t_OK`).
    pub t_ok: bool,
}

impl TwoSurfaceCore {
    /// The cache test of `ComputeValues(X, Order, ByParam, Param)`.
    pub(crate) fn cache_state(
        &self,
        x: &Vector<'_>,
        order: i32,
        by_param: bool,
        param: f64,
    ) -> Result<CacheState, Failure> {
        let t = if by_param { param } else { self.param };
        let mut x_ok = order <= self.x_order;
        let mut ii = 1;
        while ii <= x.length() && x_ok {
            x_ok = vget(x, ii)? == vget(&self.xval, ii)?;
            ii += 1;
        }
        let t_ok = t == self.tval && (order <= self.t_order || !by_param);
        Ok(CacheState { t, x_ok, t_ok })
    }

    /// Records the processing of `t` and returns the order to position on the guide.
    pub(crate) fn begin_guide(&mut self, t: f64, order: i32, by_param: bool) -> i32 {
        self.tval = t;
        self.t_order = if by_param { order } else { 0 };
        self.t_order
    }

    /// The processing of `X`: both surfaces, then the degenerate normals; false for an
    /// order above 2. `dns_at_order2` is false for BlendFunc_EvolRad, whose third-order
    /// positioning does not update the normal derivatives (they keep older values).
    pub(crate) fn position_surfaces(
        &mut self,
        d3: &mut [[Vector3; 4]; 2],
        x: &Vector<'_>,
        order: i32,
        dns_at_order2: bool,
    ) -> Result<bool, Failure> {
        copy_vector(&mut self.xval, x)?;
        self.x_order = order;
        if !(0..=2).contains(&order) {
            return Ok(false);
        }
        let uv1 = Point2::new(vget(x, 1)?, vget(x, 2)?);
        let uv2 = Point2::new(vget(x, 3)?, vget(x, 4)?);
        let with_dns = order == 1 || (order == 2 && dns_at_order2);
        position_surface(&self.surf1, uv1, order, &mut self.s1, &mut d3[0], with_dns)?;
        position_surface(&self.surf2, uv2, order, &mut self.s2, &mut d3[1], with_dns)?;
        repair_normal(&self.surf1, uv1, order, &mut self.s1)?;
        repair_normal(&self.surf2, uv2, order, &mut self.s2)?;
        Ok(true)
    }

    /// The equations and their derivatives up to `order` with the radii `radii`.
    pub(crate) fn equations(
        &mut self,
        scratch: &SurfaceScratch,
        order: i32,
        by_param: bool,
        radii: Radii,
    ) -> Result<(), Failure> {
        self.equations_order0(&scratch.guide, radii)?;
        if order >= 1 {
            self.equations_order1(&scratch.guide, by_param, radii)?;
        }
        if order == 2 {
            self.equations_order2(scratch, by_param, radii)?;
        }
        Ok(())
    }
}

/// The surface positioning of one surface to `order` (0, 1 or 2); the third derivatives
/// go to `d3` (`d3u`, `d3v`, `d3uuv`, `d3uvv`), and the normal derivatives are updated
/// when `with_dns`.
fn position_surface(
    surface: &Handle<dyn Adaptor3dSurface>,
    uv: Point2,
    order: i32,
    point: &mut SurfacePoint,
    d3: &mut [Vector3; 4],
    with_dns: bool,
) -> Result<(), Failure> {
    let live = live_surface(surface)?;
    let (u, v) = (uv.x(), uv.y());
    if order == 0 {
        let d1 = live.d1(u, v)?;
        (point.pts, point.jet.d1u, point.jet.d1v) = (d1.point, d1.d1u, d1.d1v);
    } else if order == 1 {
        let d2 = live.d2(u, v)?;
        (point.pts, point.jet.d1u, point.jet.d1v) = (d2.point, d2.d1u, d2.d1v);
        (point.jet.d2u, point.jet.d2v, point.jet.d2uv) = (d2.d2u, d2.d2v, d2.d2uv);
    } else {
        let d = live.d3(u, v)?;
        (point.pts, point.jet.d1u, point.jet.d1v) = (d.point, d.d1u, d.d1v);
        (point.jet.d2u, point.jet.d2v, point.jet.d2uv) = (d.d2u, d.d2v, d.d2uv);
        // The third derivatives live only in the thread scratch, as natively.
        *d3 = [d.d3u, d.d3v, d.d3uuv, d.d3uvv];
    }
    point.nsurf = point.jet.d1u.crossed(point.jet.d1v);
    if with_dns {
        let jet: SurfaceJet = point.jet;
        (point.dnsu, point.dnsv) = (jet.dns_u(), jet.dns_v());
    }
    Ok(())
}

/// The case of degenerated surfaces: the normal (and its derivatives) from higher
/// derivatives; an undefined normal keeps the computed values.
fn repair_normal(
    surface: &Handle<dyn Adaptor3dSurface>,
    uv: Point2,
    order: i32,
    point: &mut SurfacePoint,
) -> Result<(), Failure> {
    if point.nsurf.magnitude() >= EPS {
        return Ok(());
    }
    if order == 0 {
        if let Some(normal) = compute_normal(surface, uv)? {
            point.nsurf = normal;
        }
        return Ok(());
    }
    if let Some([normal, dnu, dnv]) = compute_d_normal(surface, uv)? {
        (point.nsurf, point.dnsu, point.dnsv) = (normal, dnu, dnv);
    }
    Ok(())
}
