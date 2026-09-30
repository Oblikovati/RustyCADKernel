// Rust translation of ModelingAlgorithms/TKGeomAlgo/Plate/Plate_GlobalTranslationConstraint,
// Plate_LineConstraint, Plate_PlaneConstraint, Plate_SampledCurveConstraint and
// Plate_D1/D2/D3 (.hxx/.lxx/.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The constraints built on the linear combinations, and the derivative records
//! `Plate_D1`, `Plate_D2`, `Plate_D3` of the G1/G2/G3 constraints.
use super::{PlateLinearScalarConstraint, PlateLinearXYZConstraint, PlatePinpointConstraint};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Line3, Plane3, Xy, Xyz};

/// First derivatives `(Du, Dv)` of a surface (`Plate_D1`); `PlateD1::new(du, dv).du()`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlateD1 {
    pub(super) du: Xyz,
    pub(super) dv: Xyz,
}

impl PlateD1 {
    /// The derivatives `du`, `dv` (`Plate_D1(du, dv)`); `PlateD1::new(du, dv)`.
    pub const fn new(du: Xyz, dv: Xyz) -> Self {
        Self { du, dv }
    }

    /// `Du` (`DU`); `d1.du()`.
    pub const fn du(&self) -> Xyz {
        self.du
    }

    /// `Dv` (`DV`); `d1.dv()`.
    pub const fn dv(&self) -> Xyz {
        self.dv
    }
}

/// Second derivatives `(Duu, Duv, Dvv)` of a surface (`Plate_D2`), read by the G2
/// constraints only, as natively. `PlateD2::new(duu, duv, dvv)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlateD2 {
    pub(super) duu: Xyz,
    pub(super) duv: Xyz,
    pub(super) dvv: Xyz,
}

impl PlateD2 {
    /// The derivatives (`Plate_D2(duu, duv, dvv)`); `PlateD2::new(duu, duv, dvv)`.
    pub const fn new(duu: Xyz, duv: Xyz, dvv: Xyz) -> Self {
        Self { duu, duv, dvv }
    }
}

/// Third derivatives `(Duuu, Duuv, Duvv, Dvvv)` of a surface (`Plate_D3`), read by the
/// G3 constraints only. `PlateD3::new(duuu, duuv, duvv, dvvv)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlateD3 {
    pub(super) duuu: Xyz,
    pub(super) duuv: Xyz,
    pub(super) duvv: Xyz,
    pub(super) dvvv: Xyz,
}

impl PlateD3 {
    /// The derivatives (`Plate_D3(duuu, duuv, duvv, dvvv)`); `PlateD3::new(a, b, c, d)`.
    pub const fn new(duuu: Xyz, duuv: Xyz, duvv: Xyz, dvvv: Xyz) -> Self {
        Self {
            duuu,
            duuv,
            duvv,
            dvvv,
        }
    }
}

/// Equal (unknown) translations at a set of points (`Plate_GlobalTranslationConstraint`):
/// the differences between each point and the first are null.
///
/// `PlateGlobalTranslationConstraint::new(&points)?.lxyzc()`.
#[derive(Clone)]
pub struct PlateGlobalTranslationConstraint {
    my_lxyzc: PlateLinearXYZConstraint,
}

impl PlateGlobalTranslationConstraint {
    /// The constraint over the points of `s_of_xy`
    /// (`Plate_GlobalTranslationConstraint(SOfXY)`); an empty sequence gives invalid
    /// matrix bounds (Standard_RangeError). `PlateGlobalTranslationConstraint::new(&points)?`.
    pub fn new(s_of_xy: &Sequence<Xy>) -> Result<Self, Failure> {
        let n: i32 = s_of_xy.length();
        let my_lxyzc = PlateLinearXYZConstraint::with_size(n - 1, n)?;
        for (i, xy) in (1..).zip(s_of_xy.iter()) {
            my_lxyzc.set_ppc(
                i,
                PlatePinpointConstraint::new(*xy, Xyz::new(0.0, 0.0, 0.0), 0, 0),
            )?;
        }
        for i in 1..n {
            my_lxyzc.set_coeff(i, 1, -1.0)?;
            for j in 2..=n {
                my_lxyzc.set_coeff(i, j, if j == i + 1 { 1.0 } else { 0.0 })?;
            }
        }
        Ok(Self { my_lxyzc })
    }

    /// The linear constraint (`LXYZC`); `c.lxyzc()`.
    pub fn lxyzc(&self) -> &PlateLinearXYZConstraint {
        &self.my_lxyzc
    }
}

/// A point constrained to a line (`Plate_LineConstraint`): two scalar equations on
/// directions orthogonal to the line.
///
/// `PlateLineConstraint::new(Xy::new(0.5, 0.5), &line, 0, 0)?.lsc()`.
#[derive(Clone)]
pub struct PlateLineConstraint {
    my_lsc: PlateLinearScalarConstraint,
}

impl PlateLineConstraint {
    /// The constraint of `D^(iu,iv)` at `point2d` on `lin`
    /// (`Plate_LineConstraint(point2d, lin, iu, iv)`; the native defaults of `iu`, `iv`
    /// are 0). The first normal is `X ^ dir` or `Y ^ dir`, whichever is longer.
    /// `PlateLineConstraint::new(uv, &line, 0, 0)?`.
    pub fn new(point2d: Xy, lin: &Line3, iu: i32, iv: i32) -> Result<Self, Failure> {
        let my_lsc = PlateLinearScalarConstraint::with_size(2, 1)?;
        let point: Xyz = lin.location().xyz();
        my_lsc.set_ppc(1, PlatePinpointConstraint::new(point2d, point, iu, iv))?;
        let dir: Xyz = lin.direction().xyz();
        let mut d1: Xyz = Xyz::new(1.0, 0.0, 0.0) ^ dir;
        let mut d2: Xyz = Xyz::new(0.0, 1.0, 0.0) ^ dir;
        if d2.square_modulus() > d1.square_modulus() {
            d1 = d2;
        }
        d1.normalize()?;
        d2 = dir ^ d1;
        d2.normalize()?;
        my_lsc.set_coeff(1, 1, d1)?;
        my_lsc.set_coeff(2, 1, d2)?;
        Ok(Self { my_lsc })
    }

    /// The scalar constraint (`LSC`); `c.lsc()`.
    pub fn lsc(&self) -> &PlateLinearScalarConstraint {
        &self.my_lsc
    }
}

/// A point constrained to a plane (`Plate_PlaneConstraint`): one scalar equation on
/// the plane normal.
///
/// `PlatePlaneConstraint::new(Xy::new(0.5, 0.5), &plane, 0, 0)?.lsc()`.
#[derive(Clone)]
pub struct PlatePlaneConstraint {
    my_lsc: PlateLinearScalarConstraint,
}

impl PlatePlaneConstraint {
    /// The constraint of `D^(iu,iv)` at `point2d` on `pln`
    /// (`Plate_PlaneConstraint(point2d, pln, iu, iv)`). `PlatePlaneConstraint::new(uv, &plane, 0, 0)?`.
    pub fn new(point2d: Xy, pln: &Plane3, iu: i32, iv: i32) -> Result<Self, Failure> {
        let my_lsc = PlateLinearScalarConstraint::with_size(1, 1)?;
        let point: Xyz = pln.location().xyz();
        my_lsc.set_ppc(1, PlatePinpointConstraint::new(point2d, point, iu, iv))?;
        let mut dir: Xyz = pln.axis().direction().xyz();
        dir.normalize()?;
        my_lsc.set_coeff(1, 1, dir)?;
        Ok(Self { my_lsc })
    }

    /// The scalar constraint (`LSC`); `c.lsc()`.
    pub fn lsc(&self) -> &PlateLinearScalarConstraint {
        &self.my_lsc
    }
}

/// The static `B0`: the hat function `max(0, 1 - |t|)`.
fn b0(t: f64) -> f64 {
    let mut s: f64 = t;
    if s < 0.0 {
        s = -s;
    }
    s = 1.0 - s;
    if s < 0.0 {
        s = 0.0;
    }
    s
}

/// A curve sampled by pinpoint constraints and smoothed onto `n` equations with hat
/// functions (`Plate_SampledCurveConstraint`).
///
/// `PlateSampledCurveConstraint::new(&samples, 3)?.lxyzc()`.
#[derive(Clone)]
pub struct PlateSampledCurveConstraint {
    my_lxyzc: PlateLinearXYZConstraint,
}

impl PlateSampledCurveConstraint {
    /// `n` hat-weighted averages of the `m` samples of `soppc`
    /// (`Plate_SampledCurveConstraint(SOPPC, n)`); Standard_DimensionMismatch when
    /// `n > m`, raised after the arrays are built, as natively.
    /// `PlateSampledCurveConstraint::new(&samples, 3)?`.
    pub fn new(soppc: &Sequence<PlatePinpointConstraint>, n: i32) -> Result<Self, Failure> {
        let m: i32 = soppc.length();
        let my_lxyzc = PlateLinearXYZConstraint::with_size(n, m)?;
        if n > m {
            return Err(Failure::with_message(
                "Standard_DimensionMismatch",
                &format!(
                    "Plate_SampledCurveConstraint: n = {n} equations for {m} samples; expected n <= m"
                ),
            ));
        }
        for (index, ppc) in (1..).zip(soppc.iter()) {
            my_lxyzc.set_ppc(index, *ppc)?;
        }
        let ratio: f64 = f64::from(n + 1) / f64::from(m + 1);
        for i in 1..=n {
            for j in 1..=m {
                my_lxyzc.set_coeff(i, j, b0(ratio * f64::from(j) - f64::from(i)))?;
            }
        }
        Ok(Self { my_lxyzc })
    }

    /// The linear constraint (`LXYZC`); `c.lxyzc()`.
    pub fn lxyzc(&self) -> &PlateLinearXYZConstraint {
        &self.my_lxyzc
    }
}
