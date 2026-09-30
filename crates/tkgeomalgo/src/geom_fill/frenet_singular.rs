// Rust translation of the private IsSingular, DoSingular and SingularD0..D2 members
// of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_Frenet.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The Frenet frame at a singular point: the first non-null derivative gives the
//! tangent and the next non-parallel one the binormal, oriented like the frame a
//! little away from the point.
use super::frame_math::{dderiv, fderiv};
use super::frenet::{GeomFillFrenet, set_normal_derivatives};
use super::sweep_failures::live;
use super::trihedron_law::{GeomFillTrihedronLaw, TrihedronD0, TrihedronD1, TrihedronD2};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::arrays::at;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::gp::Vector3;
use std::f64::consts::PI;

/// The derivative orders and orientations that `DoSingular` found.
struct SingularOrders {
    n: i32,
    k: i32,
    t_flag: i32,
    bn_flag: i32,
}

/// The highest derivative order `DoSingular` tries.
const MAX_N: i32 = 20;

/// The first order `i` from `from` with `|C^(i)(u)| > Confusion`, or `MAX_N + 1`.
fn first_non_null(
    curve: &dyn Adaptor3dCurve,
    u: f64,
    tangent: &mut Vector3,
) -> Result<i32, Failure> {
    for i in 1..=MAX_N {
        *tangent = curve.dn(u, i)?;
        if tangent.magnitude() > precision::confusion() {
            return Ok(i);
        }
    }
    Ok(MAX_N + 1)
}

/// The first order `i > n` with `|T ^ C^(i)(u)| > Confusion` (the next one when it is
/// larger), or `MAX_N + 1`.
fn first_binormal(
    curve: &dyn Adaptor3dCurve,
    u: f64,
    n: i32,
    tangent: Vector3,
    binormal: &mut Vector3,
) -> Result<i32, Failure> {
    for i in n + 1..=MAX_N {
        *binormal = tangent.crossed(curve.dn(u, i)?);
        let magn = binormal.magnitude();
        if magn > precision::confusion() {
            let next = tangent.crossed(curve.dn(u, i + 1)?);
            if next.magnitude() > magn {
                *binormal = next;
                return Ok(i + 1);
            }
            return Ok(i);
        }
    }
    Ok(MAX_N + 1)
}

impl GeomFillFrenet {
    /// `IsSingular(U, Index)`: the index of the singular neighbourhood holding `u`.
    pub(crate) fn is_singular(&self, u: f64) -> Option<i32> {
        let singular = self.singular.read();
        if !singular.is_sngl {
            return None;
        }
        (1..=singular.sngl.length()).find(|&i| (u - singular.sngl[i]).abs() < singular.sngl_len[i])
    }

    /// `DoSingular(U, Index, Tangent, BiNormal, n, k, TFlag, BNFlag, Delta)`: writes
    /// the singular tangent and binormal; `None` (native false) when no derivative
    /// qualifies, with `delta` set to the step when the binormal is missing.
    fn do_singular(
        &self,
        u: f64,
        index: i32,
        tangent: &mut Vector3,
        binormal: &mut Vector3,
        delta: &mut f64,
    ) -> Result<Option<SingularOrders>, Failure> {
        *delta = 0.0;
        let mut h = 2.0 * at(&self.singular.read().sngl_len, index)?;
        let (a, b) = self.get_interval()?;
        if u >= (a + b) / 2.0 {
            h = -h;
        }
        let trimmed = self.trimmed("GeomFill_Frenet::DoSingular")?;
        let curve = live(&trimmed, "GeomFill_Frenet::DoSingular", "myTrimmed")?;
        let n = first_non_null(curve, u, tangent)?;
        if n > MAX_N {
            return Ok(None);
        }
        tangent.normalize()?;
        let k = first_binormal(curve, u, n, *tangent, binormal)?;
        if k > MAX_N {
            *delta = h;
            return Ok(None);
        }
        binormal.normalize()?;
        let mut near = TrihedronD0::default();
        self.d0(u + h, &mut near)?;
        Ok(Some(SingularOrders {
            n,
            k,
            t_flag: if tangent.angle(near.tangent)? > PI / 2.0 {
                -1
            } else {
                1
            },
            bn_flag: if binormal.angle(near.binormal)? > PI / 2.0 {
                -1
            } else {
                1
            },
        }))
    }

    /// `SingularD0(Param, Index, Tangent, Normal, BiNormal, Delta)`.
    pub(crate) fn singular_d0(
        &self,
        param: f64,
        index: i32,
        frame: &mut TrihedronD0,
        delta: &mut f64,
    ) -> Result<bool, Failure> {
        let Some(orders) =
            self.do_singular(param, index, &mut frame.tangent, &mut frame.binormal, delta)?
        else {
            return Ok(false);
        };
        frame.tangent *= f64::from(orders.t_flag);
        frame.binormal *= f64::from(orders.bn_flag);
        frame.normal = frame.binormal;
        frame.normal.cross(frame.tangent);
        Ok(true)
    }

    /// `SingularD1(...)`: the derivatives of the singular frame.
    pub(crate) fn singular_d1(
        &self,
        param: f64,
        index: i32,
        frame: &mut TrihedronD1,
        delta: &mut f64,
    ) -> Result<bool, Failure> {
        let Some(o) =
            self.do_singular(param, index, &mut frame.tangent, &mut frame.binormal, delta)?
        else {
            return Ok(false);
        };
        let trimmed = self.trimmed("GeomFill_Frenet::SingularD1")?;
        let curve = live(&trimmed, "GeomFill_Frenet::SingularD1", "myTrimmed")?;
        let (f, df) = (curve.dn(param, o.n)?, curve.dn(param, o.n + 1)?);
        frame.d_tangent = fderiv(f, df);
        let dtmp = curve.dn(param, o.k)?;
        let f = frame.tangent.crossed(dtmp);
        let df = frame.d_tangent.crossed(dtmp) + frame.tangent.crossed(curve.dn(param, o.k + 1)?);
        frame.d_binormal = fderiv(f, df);
        if o.t_flag < 0 {
            frame.tangent = -frame.tangent;
            frame.d_tangent = -frame.d_tangent;
        }
        if o.bn_flag < 0 {
            frame.binormal = -frame.binormal;
            frame.d_binormal = -frame.d_binormal;
        }
        frame.normal = frame.binormal.crossed(frame.tangent);
        frame.d_normal =
            frame.d_binormal.crossed(frame.tangent) + frame.binormal.crossed(frame.d_tangent);
        Ok(true)
    }

    /// `SingularD2(...)`: the second derivatives of the singular frame.
    pub(crate) fn singular_d2(
        &self,
        param: f64,
        index: i32,
        frame: &mut TrihedronD2,
        delta: &mut f64,
    ) -> Result<bool, Failure> {
        let Some(o) =
            self.do_singular(param, index, &mut frame.tangent, &mut frame.binormal, delta)?
        else {
            return Ok(false);
        };
        let trimmed = self.trimmed("GeomFill_Frenet::SingularD2")?;
        let curve = live(&trimmed, "GeomFill_Frenet::SingularD2", "myTrimmed")?;
        let (f, df, d2f) = (
            curve.dn(param, o.n)?,
            curve.dn(param, o.n + 1)?,
            curve.dn(param, o.n + 2)?,
        );
        frame.d_tangent = fderiv(f, df);
        frame.d2_tangent = dderiv(f, df, d2f);
        let (dtmp1, dtmp2) = (curve.dn(param, o.k)?, curve.dn(param, o.k + 1)?);
        let f = frame.tangent.crossed(dtmp1);
        let df = frame.d_tangent.crossed(dtmp1) + frame.tangent.crossed(dtmp2);
        let d2f = frame.d2_tangent.crossed(dtmp1)
            + 2.0 * frame.d_tangent.crossed(dtmp2)
            + frame.tangent.crossed(curve.dn(param, o.k + 2)?);
        frame.d_binormal = fderiv(f, df);
        frame.d2_binormal = dderiv(f, df, d2f);
        flip_d2(frame, o.t_flag, o.bn_flag);
        frame.normal = frame.binormal.crossed(frame.tangent);
        set_normal_derivatives(frame);
        Ok(true)
    }
}

/// The orientation flips of `SingularD2`.
fn flip_d2(frame: &mut TrihedronD2, t_flag: i32, bn_flag: i32) {
    if t_flag < 0 {
        frame.tangent = -frame.tangent;
        frame.d_tangent = -frame.d_tangent;
        frame.d2_tangent = -frame.d2_tangent;
    }
    if bn_flag < 0 {
        frame.binormal = -frame.binormal;
        frame.d_binormal = -frame.d_binormal;
        frame.d2_binormal = -frame.d2_binormal;
    }
}
