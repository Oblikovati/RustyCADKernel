//! `BlendFunc_ConstRad::ComputeValues`: the obligatory passage of every computation. It
//! positions on the guide and on both surfaces, then computes the equations and their
//! derivatives up to `Order`, keeping intermediate results for the other members.
use super::super::guide::{GuideScratch, position_on_guide};
use super::super::linalg::{copy_vector, vget};
use super::super::normal_terms::{EPS, SurfaceJet};
use super::super::package::compute_d_normal;
use super::super::package::compute_normal;
use super::{ConstRad, SurfacePoint};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkmath::gp::{Point2, Vector3};
use rustycad_tkmath::math::Vector;
use std::cell::RefCell;

/// The native `static thread_local` scratch of `ComputeValues`: the third derivatives
/// of both surfaces and the guide values.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct ConstRadScratch {
    pub guide: GuideScratch,
    pub d3: [[Vector3; 4]; 2],
}

thread_local! {
    static SCRATCH: RefCell<ConstRadScratch> = RefCell::new(ConstRadScratch::default());
}

/// Runs `body` on the thread scratch and stores what it wrote, also on failure (the
/// native statics keep every value written before an exception).
pub(super) fn with_scratch<R>(body: impl FnOnce(&mut ConstRadScratch) -> R) -> R {
    let mut scratch = SCRATCH.with(|cell| *cell.borrow());
    let result = body(&mut scratch);
    SCRATCH.with(|cell| *cell.borrow_mut() = scratch);
    result
}

impl ConstRad {
    /// `ComputeValues(X, Order, ByParam, Param)`: false for an order above 2.
    pub(super) fn compute_values(
        &mut self,
        x: &Vector<'_>,
        order: i32,
        by_param: bool,
        param: f64,
    ) -> Result<bool, Failure> {
        with_scratch(|scratch| self.compute_with(scratch, x, order, (by_param, param)))
    }

    fn compute_with(
        &mut self,
        scratch: &mut ConstRadScratch,
        x: &Vector<'_>,
        order: i32,
        by: (bool, f64),
    ) -> Result<bool, Failure> {
        let (by_param, t) = (by.0, if by.0 { by.1 } else { self.param });
        let x_ok = self.x_cached(x, order)?;
        let t_ok = t == self.tval && (order <= self.t_order || !by_param);
        if x_ok && t_ok {
            return Ok(true);
        }
        if !t_ok && !self.position_guide(scratch, t, order, by_param)? {
            return Ok(false);
        }
        if !x_ok && !self.position_surfaces(scratch, x, order)? {
            return Ok(false);
        }
        self.equations_order0(&scratch.guide)?;
        if order >= 1 {
            self.equations_order1(&scratch.guide, by_param)?;
        }
        if order == 2 {
            self.equations_order2(scratch, by_param)?;
        }
        Ok(true)
    }

    /// Whether `x` and `order` are already computed (`myX_OK`).
    fn x_cached(&self, x: &Vector<'_>, order: i32) -> Result<bool, Failure> {
        let mut ok = order <= self.x_order;
        let mut ii = 1;
        while ii <= x.length() && ok {
            ok = vget(x, ii)? == vget(&self.xval, ii)?;
            ii += 1;
        }
        Ok(ok)
    }

    /// The processing of `t`.
    fn position_guide(
        &mut self,
        scratch: &mut ConstRadScratch,
        t: f64,
        order: i32,
        by_param: bool,
    ) -> Result<bool, Failure> {
        self.tval = t;
        self.t_order = if by_param { order } else { 0 };
        position_on_guide(
            &self.tcurv,
            t,
            self.t_order,
            &mut scratch.guide,
            &mut self.plane,
        )
    }

    /// The processing of `X`: both surfaces, then the degenerate normals.
    fn position_surfaces(
        &mut self,
        scratch: &mut ConstRadScratch,
        x: &Vector<'_>,
        order: i32,
    ) -> Result<bool, Failure> {
        copy_vector(&mut self.xval, x)?;
        self.x_order = order;
        if !(0..=2).contains(&order) {
            return Ok(false);
        }
        let uv1 = Point2::new(vget(x, 1)?, vget(x, 2)?);
        let uv2 = Point2::new(vget(x, 3)?, vget(x, 4)?);
        position_surface(&self.surf1, uv1, order, &mut self.s1, &mut scratch.d3[0])?;
        position_surface(&self.surf2, uv2, order, &mut self.s2, &mut scratch.d3[1])?;
        repair_normal(&self.surf1, uv1, order, &mut self.s1)?;
        repair_normal(&self.surf2, uv2, order, &mut self.s2)?;
        Ok(true)
    }
}

/// The surface positioning of one surface to `order` (0, 1 or 2); the third
/// derivatives go to `d3` (`d3u`, `d3v`, `d3uuv`, `d3uvv`).
fn position_surface(
    surface: &Handle<dyn Adaptor3dSurface>,
    uv: Point2,
    order: i32,
    point: &mut SurfacePoint,
    d3: &mut [Vector3; 4],
) -> Result<(), Failure> {
    let live = super::super::package::live_surface(surface)?;
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
    if order >= 1 {
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
