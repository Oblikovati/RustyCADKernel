// Rust translation of ModelingData/TKG3d/Geom/Geom_BSplineSurface_1.cxx (EvalD0..EvalDN, LocalD0..LocalDN, IsCNu/IsCNv) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Global evaluation, evaluation between given knots and continuity queries.
use super::state::{Direction, SurfaceState};
use crate::geom::errors;
use crate::geom::{GeomSurfaceD1, GeomSurfaceD2, GeomSurfaceD3, check_dn_orders};
use crate::geom_eval::rep_utils;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::spline_data::spline_is_cn;
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::bsplslib::{self, SurfaceD2, SurfaceD3, SurfaceSpan};
use rustycad_tkmath::gp::{Point3, Vector3};

/// The knot interval of a `LocalD*` evaluation: `(FromK1, ToK2)` in U and in V.
pub type LocalKnots = ((i32, i32), (i32, i32));

// `LocateParameter(Deg, Knots, &Mults, U, Periodic, Index, NewU)` then `FlatIndex`:
// the flat span of EvalD1/EvalD2 and the parameter located from the raw one.
fn located(direction: &Direction, u: f64, new_u: f64) -> Result<(i32, f64), Failure> {
    let (mut index, mut value): (i32, f64) = (0, new_u);
    bsplclib::locate_parameter_for_eval(
        direction.deg,
        &direction.knots,
        Some(&direction.mults),
        u,
        direction.periodic,
        &mut index,
        &mut value,
    )?;
    let flat: i32 =
        bsplclib::flat_index(direction.deg, index, &direction.mults, direction.periodic)?;
    Ok((flat, value))
}

// The span located between FromK1 and ToK2 of the flat knots, as LocalD* do.
fn located_between(
    direction: &Direction,
    u: f64,
    range: (i32, i32),
) -> Result<(i32, f64), Failure> {
    let (mut index, mut value): (i32, f64) = (0, u);
    bsplclib::locate_parameter_flat_knots(
        direction.deg,
        &direction.flat_knots,
        u,
        direction.periodic,
        range.0,
        range.1,
        &mut index,
        &mut value,
    )?;
    let flat: i32 =
        bsplclib::flat_index(direction.deg, index, &direction.mults, direction.periodic)?;
    Ok((flat, value))
}

fn d2_result(r: SurfaceD2) -> GeomSurfaceD2 {
    GeomSurfaceD2 {
        point: r.p,
        d1u: r.vu,
        d1v: r.vv,
        d2u: r.vuu,
        d2v: r.vvv,
        d2uv: r.vuv,
    }
}

/// The native field order of `ResD3` from a BSplSLib D3 result.
pub(super) fn d3_result(r: SurfaceD3) -> GeomSurfaceD3 {
    GeomSurfaceD3 {
        point: r.p,
        d1u: r.vu,
        d1v: r.vv,
        d2u: r.vuu,
        d2v: r.vvv,
        d2uv: r.vuv,
        d3u: r.vuuu,
        d3v: r.vvvv,
        d3uuv: r.vuuv,
        d3uvv: r.vuvv,
    }
}

impl SurfaceState {
    /// The evaluation span on the flat knots with the given parameters and indices.
    pub(super) fn span(&self, u: f64, v: f64, index: (i32, i32)) -> SurfaceSpan<'_, 'static> {
        SurfaceSpan {
            u,
            v,
            u_index: index.0,
            v_index: index.1,
            u_knots: &self.u.flat_knots,
            v_knots: &self.v.flat_knots,
            u_mults: None,
            v_mults: None,
            u_degree: self.u.deg,
            v_degree: self.v.deg,
            u_rat: self.u_rational,
            v_rat: self.v_rational,
            u_per: self.u.periodic,
            v_per: self.v.periodic,
        }
    }
    // The span of EvalD1/EvalD2. Quirk kept: the normalized parameters are computed,
    // then overwritten, because LocateParameter receives the raw `U` and `V`.
    fn located_span(&self, u: f64, v: f64) -> Result<SurfaceSpan<'_, 'static>, Failure> {
        let (new_u, new_v) = self.periodic_normalization(u, v)?;
        let (u_index, u_value) = located(&self.u, u, new_u)?;
        let (v_index, v_value) = located(&self.v, v, new_v)?;
        Ok(self.span(u_value, v_value, (u_index, v_index)))
    }
    pub(super) fn eval_d0(&self, u: f64, v: f64) -> Result<Point3, Failure> {
        if let Some(point) = rep_utils::try_eval_surface_d0(&self.eval_rep, u, v)? {
            return Ok(point);
        }
        let (new_u, new_v) = self.periodic_normalization(u, v)?;
        bsplslib::d0(
            &self.span(new_u, new_v, (0, 0)),
            &self.poles,
            self.weights_view(),
        )
    }
    pub(super) fn eval_d1(&self, u: f64, v: f64) -> Result<GeomSurfaceD1, Failure> {
        if let Some(result) = rep_utils::try_eval_surface_d1(&self.eval_rep, u, v)? {
            return Ok(result);
        }
        let span: SurfaceSpan<'_, 'static> = self.located_span(u, v)?;
        let (point, d1u, d1v) = bsplslib::d1(&span, &self.poles, self.weights_view())?;
        Ok(GeomSurfaceD1 { point, d1u, d1v })
    }
    pub(super) fn eval_d2(&self, u: f64, v: f64) -> Result<GeomSurfaceD2, Failure> {
        if let Some(result) = rep_utils::try_eval_surface_d2(&self.eval_rep, u, v)? {
            return Ok(result);
        }
        let span: SurfaceSpan<'_, 'static> = self.located_span(u, v)?;
        Ok(d2_result(bsplslib::d2(
            &span,
            &self.poles,
            self.weights_view(),
        )?))
    }
    /// `EvalD3` works on the raw parameters, without the periodic normalization.
    pub(super) fn eval_d3(&self, u: f64, v: f64) -> Result<GeomSurfaceD3, Failure> {
        if let Some(result) = rep_utils::try_eval_surface_d3(&self.eval_rep, u, v)? {
            return Ok(result);
        }
        Ok(d3_result(bsplslib::d3(
            &self.span(u, v, (0, 0)),
            &self.poles,
            self.weights_view(),
        )?))
    }
    /// `EvalDN` works on the raw parameters, without the periodic normalization.
    pub(super) fn eval_dn(&self, u: f64, v: f64, nu: i32, nv: i32) -> Result<Vector3, Failure> {
        check_dn_orders("Geom_BSplineSurface", nu, nv)?;
        if let Some(vector) = rep_utils::try_eval_surface_dn(&self.eval_rep, u, v, nu, nv)? {
            return Ok(vector);
        }
        bsplslib::dn(
            &self.span(u, v, (0, 0)),
            nu,
            nv,
            &self.poles,
            self.weights_view(),
        )
    }
    // The span of LocalD0..LocalDN between the given knots; Standard_DomainError for
    // equal bounds in either direction.
    fn local_span(
        &self,
        u: f64,
        v: f64,
        knots: LocalKnots,
        member: &str,
    ) -> Result<SurfaceSpan<'_, 'static>, Failure> {
        let (u_range, v_range) = knots;
        if u_range.0 == u_range.1 || v_range.0 == v_range.1 {
            return Err(errors::domain(&format!(
                "Geom_BSplineSurface::{member}: FromUK1 = {}, ToUK2 = {}, FromVK1 = {}, ToVK2 = {}; \
                 expected distinct knots in each direction",
                u_range.0, u_range.1, v_range.0, v_range.1
            )));
        }
        let (u_index, u_value) = located_between(&self.u, u, u_range)?;
        let (v_index, v_value) = located_between(&self.v, v, v_range)?;
        Ok(self.span(u_value, v_value, (u_index, v_index)))
    }
    pub(super) fn local_d0(&self, u: f64, v: f64, knots: LocalKnots) -> Result<Point3, Failure> {
        let span = self.local_span(u, v, knots, "LocalD0")?;
        bsplslib::d0(&span, &self.poles, self.weights_view())
    }
    pub(super) fn local_d1(
        &self,
        u: f64,
        v: f64,
        knots: LocalKnots,
    ) -> Result<GeomSurfaceD1, Failure> {
        let span = self.local_span(u, v, knots, "LocalD1")?;
        let (point, d1u, d1v) = bsplslib::d1(&span, &self.poles, self.weights_view())?;
        Ok(GeomSurfaceD1 { point, d1u, d1v })
    }
    pub(super) fn local_d2(
        &self,
        u: f64,
        v: f64,
        knots: LocalKnots,
    ) -> Result<GeomSurfaceD2, Failure> {
        let span = self.local_span(u, v, knots, "LocalD2")?;
        Ok(d2_result(bsplslib::d2(
            &span,
            &self.poles,
            self.weights_view(),
        )?))
    }
    pub(super) fn local_d3(
        &self,
        u: f64,
        v: f64,
        knots: LocalKnots,
    ) -> Result<GeomSurfaceD3, Failure> {
        let span = self.local_span(u, v, knots, "LocalD3")?;
        Ok(d3_result(bsplslib::d3(
            &span,
            &self.poles,
            self.weights_view(),
        )?))
    }
    pub(super) fn local_dn(
        &self,
        u: f64,
        v: f64,
        knots: LocalKnots,
        orders: (i32, i32),
    ) -> Result<Vector3, Failure> {
        let span = self.local_span(u, v, knots, "LocalDN")?;
        bsplslib::dn(&span, orders.0, orders.1, &self.poles, self.weights_view())
    }
    /// `IsCNu` (`u`) or `IsCNv`: Standard_RangeError for `n < 0`.
    pub(super) fn is_cn(&self, u: bool, n: i32) -> Result<bool, Failure> {
        if n < 0 {
            return Err(errors::range(&format!(
                "Geom_BSplineSurface::IsCN{}: N = {n}; expected N >= 0",
                if u { "u" } else { "v" }
            )));
        }
        let direction: &Direction = self.dir(u);
        spline_is_cn(direction.smooth, n, direction.deg, &direction.mults)
    }
}
