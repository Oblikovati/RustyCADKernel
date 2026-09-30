// Rust translation of the Section bodies of
// ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_QuasiAngularConvertor.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The coefficient algebra of `GeomFill_QuasiAngularConvertor::Section`: the
//! parameter `b` of the quasi-angular arc and its derivatives, the monomial terms
//! of the poles and weights, and the rotated frames. Each formula keeps the native
//! operation order, including where the D1 and D2 members write the same derivative
//! differently, and the `b + 11` of the native second weight derivative.
use super::arc_section::{ArcSectionD1, ArcSectionD2};
use super::convertor_basis::{arc_frame, math_vector};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Matrix3, Vector3, Xyz};
use rustycad_tkmath::math::{Matrix, Vector};
use std::f64::consts::PI;

/// `#define NullAngle 1.e-6`.
const NULL_ANGLE: f64 = 1.0e-6;

/// The powers `beta^1..beta^6` as the native code forms them.
struct BetaPowers {
    beta: f64,
    beta2: f64,
    beta3: f64,
    beta4: f64,
    beta5: f64,
    beta6: f64,
}

impl BetaPowers {
    fn new(beta: f64) -> Self {
        let beta2 = beta * beta;
        let beta3 = beta * beta2;
        let beta4 = beta2 * beta2;
        Self {
            beta,
            beta2,
            beta3,
            beta4,
            beta5: beta3 * beta2,
            beta6: beta3 * beta3,
        }
    }
}

/// The value terms `Vx(3,5,7)`, `Vy(2,4,6)`, `Vw(3,5,7)` of a quarter angle `beta`,
/// with the `b`, `c`, `b2`, `c2` they are made of.
pub(crate) struct QuasiTerms {
    /// `Vx(3)`, `Vx(5)`, `Vx(7)`.
    pub x: [f64; 3],
    /// `Vy(2)`, `Vy(4)`, `Vy(6)`.
    pub y: [f64; 3],
    /// `Vw(3)`, `Vw(5)`, `Vw(7)`.
    pub w: [f64; 3],
    b: f64,
    c: f64,
    b2: f64,
    c2: f64,
    powers: BetaPowers,
}

impl QuasiTerms {
    /// The terms of `beta` for the parameter `b`.
    pub fn value(beta: f64, b: f64) -> Self {
        let powers = BetaPowers::new(beta);
        let c = 1.0 / 3.0 + b;
        let (b2, c2) = (b * b, c * c);
        let p = &powers;
        Self {
            x: [
                p.beta2 * (2.0 * b - 1.0),
                p.beta4 * (b2 - 2.0 * c),
                -p.beta6 * c2,
            ],
            y: [2.0 * p.beta, p.beta3 * 2.0 * (c + b), 2.0 * p.beta5 * b * c],
            w: [
                p.beta2 * (1.0 + 2.0 * b),
                p.beta4 * (2.0 * c + b2),
                p.beta6 * c2,
            ],
            b,
            c,
            b2,
            c2,
            powers,
        }
    }
}

/// The full derivative vectors `DVx`, `DVy`, `DVw` (or their second derivatives),
/// zero where the native code leaves them at `Init(0)`.
pub(crate) struct DerivativeTerms {
    x: [f64; 7],
    y: [f64; 7],
    w: [f64; 7],
}

impl DerivativeTerms {
    /// `B * DVx`, `B * DVy`, `B * DVw`, in that order.
    pub fn products(&self, basis: &Matrix<'_>) -> Result<[Vector<'static>; 3], Failure> {
        let mut out = [
            Vector::with_bounds(1, 7)?,
            Vector::with_bounds(1, 7)?,
            Vector::with_bounds(1, 7)?,
        ];
        for (product, terms) in out.iter_mut().zip([&self.x, &self.y, &self.w]) {
            product.set_matrix_vector(basis, &math_vector(terms)?)?;
        }
        Ok(out)
    }
}

/// `b` of the D0 member.
pub(crate) fn value_b(beta: f64) -> f64 {
    let beta2 = beta * beta;
    if (PI / 2.0 - beta) > NULL_ANGLE {
        if beta.abs() < NULL_ANGLE {
            let cf = 2.0 / (3.0 * 5.0 * 7.0);
            return -(0.2 + cf * beta2) / (1.0 + 0.2 * beta2);
        }
        let tan_b = beta.tan();
        let mut b = -1.0e0 / beta2;
        b += beta / (3.0 * (tan_b - beta));
        return b;
    }
    -1.0 / beta2
}

/// `b` and its derivative `bpr` of the D1 member.
pub(crate) fn first_order_b(beta: f64, betaprim: f64) -> (f64, f64) {
    let beta2 = beta * beta;
    if beta.abs() < NULL_ANGLE {
        let cf = 2.0 / (3.0 * 5.0 * 7.0);
        let num = 0.2 + cf * beta2;
        let denom = 1.0 + 0.2 * beta2;
        let bpr = -2.0 * beta * betaprim * (cf * denom - 0.2 * num) / (denom * denom);
        return (-num / denom, bpr);
    }
    let beta3 = beta * beta2;
    let (mut b, mut bpr) = (-1.0 / beta2, (2.0 * betaprim) / beta3);
    if (PI / 2.0 - beta) > NULL_ANGLE {
        let tan_b = beta.tan();
        let dtan_b = betaprim * (1.0 + tan_b * tan_b);
        let b2 = tan_b - beta;
        b += beta / (3.0 * b2);
        bpr += (betaprim * tan_b - beta * dtan_b) / (3.0 * b2 * b2);
    }
    (b, bpr)
}

/// `b`, `bpr` and the second derivative `bsc` of the D2 member (whose small-angle
/// `cf` is `-2/21`, where D0 and D1 use `2/105`).
pub(crate) fn second_order_b(beta: f64, betaprim: f64, betasecn: f64) -> (f64, f64, f64) {
    let (beta2, betaprim2) = (beta * beta, betaprim * betaprim);
    if beta.abs() < NULL_ANGLE {
        let cf = -2.0 / 21.0;
        let num = 0.2 + cf * beta2;
        let denom = 1.0 + 0.2 * beta2;
        let aux = (cf * denom - 0.2 * num) / (denom * denom);
        let bpr = -2.0 * beta * betaprim * aux;
        let bsc = 2.0 * aux * (betaprim2 + beta * betasecn - 2.0 * beta * betaprim2);
        return (-num / denom, bpr, bsc);
    }
    let beta3 = beta * beta2;
    let mut b = -1.0 / beta2;
    let mut bpr = (2.0 * betaprim) / beta3;
    let mut bsc = (2.0 * betasecn - 6.0 * betaprim * (betaprim / beta)) / beta3;
    if (PI / 2.0 - beta) > NULL_ANGLE {
        let tan_b = beta.tan();
        let dtan_b = betaprim * (1.0 + tan_b * tan_b);
        let d2tan_b = betasecn * (1.0 + tan_b * tan_b) + 2.0 * betaprim * tan_b * dtan_b;
        let b2 = tan_b - beta;
        b += beta / (3.0 * b2);
        let aux = betaprim * tan_b - beta * dtan_b;
        bpr += aux / (3.0 * b2 * b2);
        let daux = betasecn * tan_b - beta * d2tan_b;
        bsc += (daux - 2.0 * aux * betaprim * tan_b * tan_b / b2) / (3.0 * b2 * b2);
    }
    (b, bpr, bsc)
}

/// `DVx` and `DVy`, written alike by the D1 and D2 members.
fn first_xy(t: &QuasiTerms, betaprim: f64, bpr: f64) -> ([f64; 7], [f64; 7]) {
    let p = &t.powers;
    let (b, c, b2, c2) = (t.b, t.c, t.b2, t.c2);
    let mut x = [0.0; 7];
    x[2] = 2.0 * (p.beta * betaprim * (2.0 * b - 1.0) + bpr * p.beta2);
    x[4] = 4.0 * p.beta3 * betaprim * (b2 - 2.0 * c) + 2.0 * p.beta4 * bpr * (b - 1.0);
    x[6] = -6.0 * p.beta5 * betaprim * c2 - 2.0 * p.beta6 * bpr * c;
    let mut y = [0.0; 7];
    y[1] = 2.0 * betaprim;
    y[3] = 6.0 * p.beta2 * betaprim * (b + c) + 4.0 * p.beta3 * bpr;
    y[5] = 10.0 * p.beta4 * betaprim * b * c + 2.0 * p.beta5 * bpr * (b + c);
    (x, y)
}

/// The first derivative terms of the D1 member (its own `DVw` expressions).
pub(crate) fn first_order_terms(t: &QuasiTerms, betaprim: f64, bpr: f64) -> DerivativeTerms {
    let (x, y) = first_xy(t, betaprim, bpr);
    let p = &t.powers;
    let (b, c, b2) = (t.b, t.c, t.b2);
    let mut w = [0.0; 7];
    w[2] = 2.0 * p.beta * (betaprim * (1.0 + 2.0 * b) + p.beta * bpr);
    w[4] = 2.0 * p.beta3 * (2.0 * betaprim * (2.0 * c + b2) + p.beta * bpr * (b + 1.0));
    w[6] = 2.0 * p.beta5 * c * (3.0 * betaprim * c + p.beta * bpr);
    DerivativeTerms { x, y, w }
}

/// The first derivative terms of the D2 member (its expanded `DVw` expressions).
pub(crate) fn first_order_terms_d2(t: &QuasiTerms, betaprim: f64, bpr: f64) -> DerivativeTerms {
    let (x, y) = first_xy(t, betaprim, bpr);
    let p = &t.powers;
    let (b, c, b2, c2) = (t.b, t.c, t.b2, t.c2);
    let mut w = [0.0; 7];
    w[2] = 2.0 * (p.beta * betaprim * (1.0 + 2.0 * b) + p.beta2 * bpr);
    w[4] = 4.0 * p.beta3 * betaprim * (2.0 * c + b2) + 2.0 * p.beta4 * bpr * (b + 1.0);
    w[6] = 6.0 * p.beta5 * betaprim * c2 + 2.0 * p.beta6 * bpr * c;
    DerivativeTerms { x, y, w }
}

/// The second derivative terms `D2Vx`, `D2Vy`, `D2Vw` of the D2 member;
/// `primes` is `[betaprim, betasecn]` and `bders` is `[bpr, bsc]`.
pub(crate) fn second_order_terms(
    t: &QuasiTerms,
    primes: [f64; 2],
    bders: [f64; 2],
) -> DerivativeTerms {
    let p = &t.powers;
    let [betaprim, betasecn] = primes;
    let [bpr, bsc] = bders;
    let (b, c, b2, c2) = (t.b, t.c, t.b2, t.c2);
    let (betaprim2, bpr2) = (betaprim * betaprim, bpr * bpr);
    let mut x = [0.0; 7];
    x[2] = 2.0
        * ((betaprim2 + p.beta * betasecn) * (2.0 * b - 1.0)
            + 8.0 * p.beta * betaprim * bpr
            + bsc * p.beta2);
    x[4] = 4.0 * (b2 - 2.0 * c) * (3.0 * p.beta2 * betaprim2 + p.beta3 * betasecn)
        + 16.0 * p.beta3 * betaprim * bpr * (b - 1.0)
        + 2.0 * p.beta4 * (bsc * (b - 1.0) + bpr2);
    x[6] = -6.0 * c2 * (5.0 * p.beta4 * betaprim2 + p.beta5 * betasecn)
        - 24.0 * p.beta5 * betaprim * bpr * c
        - 2.0 * p.beta6 * (bsc * c + bpr2);
    let mut y = [0.0; 7];
    y[1] = 2.0 * betasecn;
    y[3] = 6.0 * (b + c) * (2.0 * p.beta * betaprim2 + p.beta2 * betasecn)
        + 24.0 * p.beta2 * betaprim * bpr * (b + c)
        + 4.0 * p.beta3 * bsc;
    y[5] = 10.0 * b * c * (4.0 * p.beta3 * betaprim2 + p.beta4 * betasecn)
        + 40.0 * p.beta4 * betaprim * bpr * (b + c)
        + 2.0 * p.beta5 * (bsc * (b + c) + 2.0 * bpr2);
    DerivativeTerms {
        x,
        y,
        w: second_weight_terms(t, betaprim, betasecn, [bpr, bsc]),
    }
}

// `D2Vw`, with the native `(b + 11)` of `D2Vw(5)` kept as written.
fn second_weight_terms(t: &QuasiTerms, betaprim: f64, betasecn: f64, bders: [f64; 2]) -> [f64; 7] {
    let p = &t.powers;
    let [bpr, bsc] = bders;
    let (b, c, b2, c2) = (t.b, t.c, t.b2, t.c2);
    let (betaprim2, bpr2) = (betaprim * betaprim, bpr * bpr);
    let mut w = [0.0; 7];
    w[2] = 2.0
        * ((betaprim2 + p.beta * betasecn) * (2.0 * b + 1.0)
            + 8.0 * p.beta * betaprim * bpr
            + bsc * p.beta2);
    w[4] = 4.0 * (b2 + 2.0 * c) * (3.0 * p.beta2 * betaprim2 + p.beta3 * betasecn)
        + 16.0 * p.beta3 * betaprim * bpr * (b + 11.0)
        + 2.0 * p.beta4 * (bsc * (b + 1.0) + bpr2);
    w[6] = 6.0 * c2 * (5.0 * p.beta4 * betaprim2 + p.beta5 * betasecn)
        + 24.0 * p.beta5 * betaprim * bpr * c
        + 2.0 * p.beta6 * (bsc * c + bpr2);
    w
}

/// `D`, `D^2` of the cross matrix of `dir` and the rotation of half the angle.
fn rotation_parts(dir: Vector3, angle: f64) -> Result<(Matrix3, Matrix3, Matrix3), Failure> {
    let mut rot = Matrix3::default();
    rot.set_rotation(*dir.xyz(), angle / 2.0)?;
    let mut d = Matrix3::default();
    d.set_cross(*dir.xyz());
    Ok((rot, d, d.powered(2)?))
}

/// `M` and `MPrim` of the D1 member (from `Rot`, `RotPrim`).
pub(crate) fn frame_d1(arc: &ArcSectionD1) -> Result<(Matrix3, Matrix3), Failure> {
    let base = &arc.arc;
    let (rot, d, d_sq) = rotation_parts(base.dir, base.angle)?;
    let (sina, cosa) = ((base.angle / 2.0).sin(), (base.angle / 2.0).cos());
    let mut d_prim = Matrix3::default();
    d_prim.set_cross(*arc.d_dir.xyz());
    let mut rot_prim = d_sq * sina;
    rot_prim += d * cosa;
    rot_prim *= arc.d_angle / 2.0;
    rot_prim += d_prim * sina;
    rot_prim += ((d_prim * d) + (d * d_prim)) * (1.0 - cosa);
    let v1 = Vector3::between(base.center, base.first_pnt);
    let mut aux = rot * *(arc.d_first_pnt - arc.d_center).xyz();
    aux += rot_prim * *v1.xyz();
    let v1_prim = Vector3::from(aux);
    let mut rotated = *v1.xyz();
    rotated *= rot;
    let v1 = Vector3::from(rotated);
    let m = arc_frame(v1, base.dir ^ v1);
    let v2 = (arc.d_dir ^ v1) + (base.dir ^ v1_prim);
    Ok((m, arc_frame(v1_prim, v2)))
}

/// `RotPrim` and `RotSecn` of the D2 member.
fn rotation_derivatives(
    arc: &ArcSectionD2,
    d: Matrix3,
    d_sq: Matrix3,
) -> Result<(Matrix3, Matrix3), Failure> {
    let d1 = &arc.d1;
    let (sina, cosa) = ((d1.arc.angle / 2.0).sin(), (d1.arc.angle / 2.0).cos());
    let (mut d_prim, mut d_secn) = (Matrix3::default(), Matrix3::default());
    d_prim.set_cross(*d1.d_dir.xyz());
    d_secn.set_cross(*arc.d2_dir.xyz());
    let ddp = (d_prim * d) + (d * d_prim);
    let mut rot_prim = d_sq * sina;
    rot_prim += d * cosa;
    rot_prim *= d1.d_angle / 2.0;
    rot_prim += d_prim * sina;
    rot_prim += ddp * (1.0 - cosa);
    let mut rot_secn = d_sq * sina;
    rot_secn += d * cosa;
    rot_secn *= arc.d2_angle / 2.0;
    let mut maux = d_sq * cosa;
    maux -= d * sina;
    maux *= d1.d_angle / 2.0;
    maux += ddp * (2.0 * sina);
    maux += d_prim * (2.0 * cosa);
    maux *= d1.d_angle / 2.0;
    rot_secn += maux;
    let mut maux = (d_secn * d) + (d * d_secn);
    maux += d_prim.powered(2)? * 2.0;
    maux *= 1.0 - cosa;
    maux += d_secn * sina;
    rot_secn += maux;
    Ok((rot_prim, rot_secn))
}

/// `M`, `MPrim` and `MSecn` of the D2 member.
pub(crate) fn frame_d2(arc: &ArcSectionD2) -> Result<[Matrix3; 3], Failure> {
    let d1 = &arc.d1;
    let (rot, d, d_sq) = rotation_parts(d1.arc.dir, d1.arc.angle)?;
    let (rot_prim, rot_secn) = rotation_derivatives(arc, d, d_sq)?;
    let v1 = Vector3::between(d1.arc.center, d1.arc.first_pnt);
    let v1_prim = d1.d_first_pnt - d1.d_center;
    let mut auxyz: Xyz = rot * *(arc.d2_first_pnt - arc.d2_center).xyz();
    auxyz += 2.0 * (rot_prim * *v1_prim.xyz());
    auxyz += rot_secn * *v1.xyz();
    let v1_secn = Vector3::from(auxyz);
    let mut auxyz = rot * *v1_prim.xyz();
    auxyz += rot_prim * *v1.xyz();
    let v1_prim = Vector3::from(auxyz);
    let mut rotated = *v1.xyz();
    rotated *= rot;
    let v1 = Vector3::from(rotated);
    let m = arc_frame(v1, d1.arc.dir ^ v1);
    let m_prim = arc_frame(v1_prim, (d1.d_dir ^ v1) + (d1.arc.dir ^ v1_prim));
    let mut v2 = d1.d_dir ^ v1_prim;
    v2 *= 2.0;
    v2 += (arc.d2_dir ^ v1) + (d1.arc.dir ^ v1_secn);
    Ok([m, m_prim, arc_frame(v1_secn, v2)])
}
