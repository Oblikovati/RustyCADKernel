// Rust translation of ModelingAlgorithms/TKTopAlgo/Bisector/Bisector_BisecCC.cxx (Values and Extension) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The derivative of the bisector of two curves and its extensions by tangency.
use super::state::{CcState, OnCurves};
use crate::bisector::PointOnBis;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{self, Point2, Vector2};

/// The outputs of `Values`: the point and the three derivatives, of which only the
/// first is ever computed (the second and third stay null).
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct CcValues {
    pub p: Point2,
    pub v1: Vector2,
    pub v2: Vector2,
    pub v3: Vector2,
}

// The scalar products of the first derivative.
struct Products {
    tu_tu: f64,
    tv_tv: f64,
    tu_tv: f64,
    tu_pu_pv: f64,
    tv_pu_pv: f64,
    tuu_pu_pv: f64,
    tu_tuu: f64,
    tvv_pu_pv: f64,
    tv_tvv: f64,
}

impl Products {
    fn new(tu: Vector2, tuu: Vector2, tv: Vector2, tvv: Vector2, pu_pv: Vector2) -> Self {
        Self {
            tu_tu: tu.dot(tu),
            tv_tv: tv.dot(tv),
            tu_tv: tu.dot(tv),
            tu_pu_pv: tu.dot(pu_pv),
            tv_pu_pv: tv.dot(pu_pv),
            tuu_pu_pv: tuu.dot(pu_pv),
            tu_tuu: tu.dot(tuu),
            tvv_pu_pv: tvv.dot(pu_pv),
            tv_tvv: tv.dot(tvv),
        }
    }
    // `dH/du` and `dH/dv` of H(u,v) = (Tu.P1P2)^2 |Tv|^2 - (Tv.P1P2)^2 |Tu|^2.
    fn dh(&self) -> (f64, f64) {
        let dhdu: f64 = 2.
            * (self.tu_pu_pv * (self.tuu_pu_pv - self.tu_tu) * self.tv_tv
                + self.tv_pu_pv * self.tu_tv * self.tu_tu
                - self.tu_tuu * self.tv_pu_pv * self.tv_pu_pv);
        let dhdv: f64 = 2.
            * (self.tu_pu_pv * self.tu_tv * self.tv_tv
                + self.tv_tvv * self.tu_pu_pv * self.tu_pu_pv
                - self.tv_pu_pv * (self.tvv_pu_pv + self.tv_tv) * self.tu_tu);
        (dhdu, dhdv)
    }
}

impl CcState {
    /// `Values(U, N, P, V1, V2, V3)`: the point and, for `n >= 1`, the first
    /// derivative `dB/du = dF/du + dF/dv (-dH/du / dH/dv)`, the tangent bisecting the
    /// tangents of the curves.
    pub(crate) fn values(&self, u: f64, n: i32) -> Result<CcValues, Failure> {
        let mut out = CcValues::default();
        let mut on_curves = OnCurves::default();
        let poly = &self.my_polygon;
        if poly.length() <= 1 {
            // The native code computes the extension here and goes on.
            let (p, t) = self.extension(u, &mut on_curves)?;
            (out.p, out.v1) = (p, t);
        }
        if u < poly.first().param_on_bis() || u > poly.last().param_on_bis() {
            let (p, t) = self.extension(u, &mut on_curves)?;
            (out.p, out.v1) = (p, t);
            return Ok(out);
        }
        out.p = self.value_and_dist(u, &mut on_curves)?;
        if n == 0 {
            return Ok(out);
        }
        out.v1 = self.first_derivative(on_curves.u1, on_curves.u2)?;
        Ok(out)
    }

    // The first derivative at the parameters `u0` on C1 and `v0` on C2.
    fn first_derivative(&self, u0: f64, v0: f64) -> Result<Vector2, Failure> {
        let (p1, tu, tuu) = self.curve1.d2(u0)?;
        let nor = Vector2::new(-tu.y(), tu.x());
        let nu = Vector2::new(-tuu.y(), tuu.x());
        let (p2, tv, tvv) = self.curve2.d2(v0)?;
        let pu_pv = Vector2::new(p2.x() - p1.x(), p2.y() - p1.y());
        let (dhdu, dhdv) = Products::new(tu, tuu, tv, tvv, pu_pv).dh();
        let nor_pu_pv: f64 = nor.dot(pu_pv);
        let nu_pu_pv: f64 = nu.dot(pu_pv);
        let nor_tv: f64 = nor.dot(tv);
        let a: f64 = 0.5 * pu_pv.square_magnitude();
        let b: f64 = -nor_pu_pv;
        let bb: f64 = b * b;
        let (dadu, dbdu, dadv, dbdv) = (-tu.dot(pu_pv), -nu_pu_pv, tv.dot(pu_pv), -nor_tv);
        if bb < gp::resolution() {
            let v1: Vector2 = tu.normalized()? + tv.normalized()?;
            return Ok(0.5 * tu.square_magnitude() * v1);
        }
        let dfdu: Vector2 = tu - (dadu / b - dbdu * a / bb) * nor - (a / b) * nu;
        let dfdv: Vector2 = (-dadv / b + dbdv * a / bb) * nor;
        if dhdv.abs() > gp::resolution() {
            return Ok(dfdu + dfdv * (-dhdu / dhdv));
        }
        Ok(tu)
    }

    /// `Extension(U, U1, U2, Dist, T)`: the point on the extensions (the segments to
    /// the start or end points, or the tangent half lines), its unit tangent, the
    /// parameters on the curves and the distance (not squared) to the first curve.
    pub(crate) fn extension(
        &self,
        u: f64,
        out: &mut OnCurves,
    ) -> Result<(Point2, Vector2), Failure> {
        let poly = &self.my_polygon;
        let (p, p1, du, tang) = if poly.length() == 0 {
            self.empty_polygon_segment(u, out)?
        } else if u < poly.first().param_on_bis() {
            self.start_extension(u, poly.first(), out)?
        } else if u > poly.last().param_on_bis() {
            self.end_extension(u, poly.last(), out)?
        } else {
            (Point2::default(), Point2::default(), 0., Vector2::default())
        };
        let t: Vector2 = tang.normalized()?;
        let pbis = Point2::new(p.x() + du * t.x(), p.y() + du * t.y());
        out.dist = p1.distance(pbis);
        Ok((pbis, t))
    }

    // An empty polygon: the segment from the start point to the end point.
    fn empty_polygon_segment(
        &self,
        u: f64,
        out: &mut OnCurves,
    ) -> Result<(Point2, Point2, f64, Vector2), Failure> {
        let du: f64 = u - self.start_intervals.first()?;
        let (p, p1) = (self.point_start, self.point_end);
        out.u1 = self.curve1.last_parameter();
        out.u2 = if self.sign1 == self.sign2 {
            self.curve2.first_parameter()
        } else {
            self.curve2.last_parameter()
        };
        Ok((p, p1, du, Vector2::new(p1.x() - p.x(), p1.y() - p.y())))
    }

    // Before the polygon: the segment from the start point, or the tangent.
    fn start_extension(
        &self,
        u: f64,
        pref: PointOnBis,
        out: &mut OnCurves,
    ) -> Result<(Point2, Point2, f64, Vector2), Failure> {
        let (p, du) = (pref.point(), u - pref.param_on_bis());
        if !self.extension_start {
            return self.tangent_extension(pref, p, du, out);
        }
        let p1: Point2 = self.point_start;
        out.u1 = self.curve1.first_parameter();
        out.u2 = if self.sign1 == self.sign2 {
            self.curve2.last_parameter()
        } else {
            self.curve2.first_parameter()
        };
        Ok((p, p1, du, Vector2::new(p.x() - p1.x(), p.y() - p1.y())))
    }

    // After the polygon: the segment to the end point, or the tangent.
    fn end_extension(
        &self,
        u: f64,
        pref: PointOnBis,
        out: &mut OnCurves,
    ) -> Result<(Point2, Point2, f64, Vector2), Failure> {
        let (p, du) = (pref.point(), u - pref.param_on_bis());
        if !self.extension_end {
            return self.tangent_extension(pref, p, du, out);
        }
        let p1: Point2 = self.point_end;
        out.u1 = self.curve1.last_parameter();
        out.u2 = if self.sign1 == self.sign2 {
            self.curve2.last_parameter()
        } else {
            self.curve2.first_parameter()
        };
        Ok((p, p1, du, Vector2::new(p1.x() - p.x(), p1.y() - p.y())))
    }

    // A curve without extension is extended by tangency at the polygon point.
    fn tangent_extension(
        &self,
        pref: PointOnBis,
        p: Point2,
        du: f64,
        out: &mut OnCurves,
    ) -> Result<(Point2, Point2, f64, Vector2), Failure> {
        out.u1 = pref.param_on_c1();
        out.u2 = pref.param_on_c2();
        let p2: Point2 = self.curve2.value(out.u2)?;
        let (p1, t1) = self.curve1.d1(out.u1)?;
        let mut tang = Vector2::new(2. * p.x() - p1.x() - p2.x(), 2. * p.y() - p1.y() - p2.y());
        if tang.magnitude() < precision::confusion() {
            tang = t1;
        }
        if t1.dot(tang) < 0. {
            tang = -tang;
        }
        Ok((p, p1, du, tang))
    }
}
