// Copyright (c) 2025 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of ModelingData/TKG2d/Geom2d/Geom2d_OffsetCurveUtils.pxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2d_OffsetCurveUtils`: offset point and derivatives from basis derivatives.
//!
//! `P(u) = p(u) + Offset * N / |N|` with `N = (p'(u).Y, -p'(u).X)`. The formulas and
//! their evaluation order are the native ones, including the unstable fallback used
//! when `|N|` is tiny. The `calculate_*` functions return `Ok(false)` where the
//! native ones return false (degenerate normal).
use super::curve::{Geom2dCurve, Geom2dCurveD1, Geom2dCurveD2, Geom2dCurveD3};
use super::native::{std_max, std_min};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Direction2, Point2, Vector2, Xy, resolution};

/// Offset point: `value` enters as the basis point (`CalculateD0`);
/// `calculate_d0(&mut p, d1, 2.0)?`.
pub fn calculate_d0(value: &mut Point2, d1: Vector2, offset: f64) -> Result<bool, Failure> {
    if d1.square_magnitude() <= resolution() {
        return Ok(false);
    }
    let normal: Direction2 = Direction2::new(d1.y(), -d1.x())?;
    *value.change_coord() += normal.xy() * offset;
    Ok(true)
}

/// Offset point and first derivative; `value` and `d1` enter as the basis ones (`CalculateD1`);
/// `calculate_d1(&mut p, &mut v1, v2, 2.0)`.
pub fn calculate_d1(value: &mut Point2, d1: &mut Vector2, d2: Vector2, offset: f64) -> bool {
    let mut n_dir = Xy::new(d1.y(), -d1.x());
    let mut dn_dir = Xy::new(d2.y(), -d2.x());
    let r2: f64 = n_dir.square_modulus();
    let r: f64 = r2.sqrt();
    let r3: f64 = r * r2;
    let dr: f64 = n_dir.dot(dn_dir);
    if r3 <= resolution() {
        if r2 <= resolution() {
            return false;
        }
        // Another computation, whose stability is not very good.
        dn_dir *= r;
        dn_dir -= n_dir * (dr / r);
        dn_dir *= offset / r2;
    } else {
        // Same computation as IICURV in EUCLID-IS because the stability is better.
        dn_dir *= offset / r;
        dn_dir -= n_dir * (offset * dr / r3);
    }
    n_dir *= offset / r;
    *value.change_coord() += n_dir;
    *d1 += Vector2::from(dn_dir);
    true
}

// The normal direction and its derivatives from the basis derivatives, with the
// scalar products that the D2 and D3 formulas share.
struct NormalTerms {
    n: Xy,
    dn: Xy,
    d2n: Xy,
    r2: f64,
    r: f64,
    dr: f64,
    d2r: f64,
}
impl NormalTerms {
    fn new(d1: Vector2, d2: Vector2, d3: Vector2) -> Self {
        let n = Xy::new(d1.y(), -d1.x());
        let dn = Xy::new(d2.y(), -d2.x());
        let d2n = Xy::new(d3.y(), -d3.x());
        let r2: f64 = n.square_modulus();
        let dr: f64 = n.dot(dn);
        let d2r: f64 = n.dot(d2n) + dn.dot(dn);
        Self {
            n,
            dn,
            d2n,
            r2,
            r: r2.sqrt(),
            dr,
            d2r,
        }
    }
}

/// Offset point and two derivatives; `value`, `d1`, `d2` enter as the basis ones
/// (`CalculateD2`); `calculate_d2(&mut p, &mut v1, &mut v2, v3, false, 2.0)`.
pub fn calculate_d2(
    value: &mut Point2,
    d1: &mut Vector2,
    d2: &mut Vector2,
    d3: Vector2,
    is_dir_change: bool,
    offset: f64,
) -> bool {
    let t = NormalTerms::new(*d1, *d2, d3);
    let (r, r2, dr, d2r) = (t.r, t.r2, t.dr, t.d2r);
    let (mut n_dir, mut dn_dir, mut d2n_dir) = (t.n, t.dn, t.d2n);
    let r3: f64 = r2 * r;
    let r4: f64 = r2 * r2;
    let r5: f64 = r3 * r2;
    if r5 <= resolution() {
        if r4 <= resolution() {
            return false;
        }
        // Another computation, whose stability is not very good dixit ISG.
        d2n_dir -= dn_dir * (2.0 * dr / r2);
        d2n_dir += n_dir * (((3.0 * dr * dr) / r4) - (d2r / r2));
        d2n_dir *= offset / r;
        dn_dir *= r;
        dn_dir -= n_dir * (dr / r);
        dn_dir *= offset / r2;
    } else {
        // Same computation as IICURV in EUCLID-IS because the stability is better.
        d2n_dir *= offset / r;
        d2n_dir -= dn_dir * (2.0 * offset * dr / r3);
        d2n_dir += n_dir * (offset * (((3.0 * dr * dr) / r5) - (d2r / r3)));
        dn_dir *= offset / r;
        dn_dir -= n_dir * (offset * dr / r3);
    }
    n_dir *= offset / r;
    *value.change_coord() += n_dir;
    *d1 += Vector2::from(dn_dir);
    if is_dir_change {
        d2.reverse();
    }
    *d2 += Vector2::from(d2n_dir);
    true
}

/// Offset point and three derivatives; `value`, `d1`, `d2`, `d3` enter as the basis
/// ones (`CalculateD3`). Quirk kept: in the unstable branch the Ndir term of D2 is
/// subtracted, where CalculateD2 adds it;
/// `calculate_d3(&mut p, &mut v1, &mut v2, &mut v3, v4, false, 2.0)`.
pub fn calculate_d3(
    value: &mut Point2,
    derivatives: [&mut Vector2; 3],
    d4: Vector2,
    is_dir_change: bool,
    offset: f64,
) -> bool {
    let [d1, d2, d3] = derivatives;
    let t = NormalTerms::new(*d1, *d2, *d3);
    let (mut n_dir, mut dn_dir, mut d2n_dir) = (t.n, t.dn, t.d2n);
    let mut d3n_dir = Xy::new(d4.y(), -d4.x());
    let (r, r2, dr, d2r) = (t.r, t.r2, t.dr, t.d2r);
    let r3: f64 = r2 * r;
    let mut r4: f64 = r2 * r2;
    let r5: f64 = r3 * r2;
    let r6: f64 = r3 * r3;
    let r7: f64 = r5 * r2;
    let d3r: f64 = n_dir.dot(d3n_dir) + 3.0 * dn_dir.dot(d2n_dir);
    if r7 <= resolution() {
        if r6 <= resolution() {
            return false;
        }
        // Another computation, whose stability is not very good dixit ISG.
        d3n_dir -= d2n_dir * (3.0 * dr / r2);
        d3n_dir -= dn_dir * (3.0 * ((d2r / r2) + (dr * dr / r4)));
        d3n_dir +=
            n_dir * (6.0 * dr * dr / r4 + 6.0 * dr * d2r / r4 - 15.0 * dr * dr * dr / r6 - d3r);
        d3n_dir *= offset / r;
        r4 = r2 * r2;
        d2n_dir -= dn_dir * (2.0 * dr / r2);
        d2n_dir -= n_dir * ((3.0 * dr * dr / r4) - (d2r / r2));
        d2n_dir *= offset / r;
        dn_dir *= r;
        dn_dir -= n_dir * (dr / r);
        dn_dir *= offset / r2;
    } else {
        // Same computation as IICURV in EUCLID-IS because the stability is better.
        d3n_dir *= offset / r;
        d3n_dir -= d2n_dir * (3.0 * offset * dr / r3);
        d3n_dir -= dn_dir * ((3.0 * offset) * ((d2r / r3) + (dr * dr) / r5));
        d3n_dir += n_dir
            * (offset
                * (6.0 * dr * dr / r5 + 6.0 * dr * d2r / r5 - 15.0 * dr * dr * dr / r7 - d3r));
        d2n_dir *= offset / r;
        d2n_dir -= dn_dir * (2.0 * offset * dr / r3);
        d2n_dir -= n_dir * (offset * (((3.0 * dr * dr) / r5) - (d2r / r3)));
        dn_dir *= offset / r;
        dn_dir -= n_dir * (offset * dr / r3);
    }
    n_dir *= offset / r;
    *value.change_coord() += n_dir;
    *d1 += Vector2::from(dn_dir);
    *d2 += Vector2::from(d2n_dir);
    if is_dir_change {
        d3.reverse();
    }
    *d3 += Vector2::from(d3n_dir);
    true
}

/// The basis-curve members that the offset evaluation needs (the native
/// `CurveType`/`BasisCurveType` template parameters); implemented for every Geom2d curve.
pub trait OffsetBasisCurve {
    /// `FirstParameter`.
    fn basis_first_parameter(&self) -> f64;
    /// `LastParameter`.
    fn basis_last_parameter(&self) -> f64;
    /// `EvalD0`.
    fn basis_d0(&self, u: f64) -> Result<Point2, Failure>;
    /// `EvalD1`.
    fn basis_d1(&self, u: f64) -> Result<Geom2dCurveD1, Failure>;
    /// `EvalD2`.
    fn basis_d2(&self, u: f64) -> Result<Geom2dCurveD2, Failure>;
    /// `EvalD3`.
    fn basis_d3(&self, u: f64) -> Result<Geom2dCurveD3, Failure>;
    /// `EvalDN`.
    fn basis_dn(&self, u: f64, n: i32) -> Result<Vector2, Failure>;
}
impl<C: Geom2dCurve + ?Sized> OffsetBasisCurve for C {
    fn basis_first_parameter(&self) -> f64 {
        self.first_parameter()
    }
    fn basis_last_parameter(&self) -> f64 {
        self.last_parameter()
    }
    fn basis_d0(&self, u: f64) -> Result<Point2, Failure> {
        self.eval_d0(u)
    }
    fn basis_d1(&self, u: f64) -> Result<Geom2dCurveD1, Failure> {
        self.eval_d1(u)
    }
    fn basis_d2(&self, u: f64) -> Result<Geom2dCurveD2, Failure> {
        self.eval_d2(u)
    }
    fn basis_d3(&self, u: f64) -> Result<Geom2dCurveD3, Failure> {
        self.eval_d3(u)
    }
    fn basis_dn(&self, u: f64, n: i32) -> Result<Vector2, Failure> {
        self.eval_dn(u, n)
    }
}

/// Replace near-null derivatives at a singular point by the first non-null higher
/// derivative, oriented by a chord of Taylor step `max(range * 1e-3, 1e-7)`;
/// `derivatives` holds D1..D4 and only D1..D`max_derivative` are written. Returns
/// whether the direction changed (`AdjustDerivative`, whose native bool is always true);
/// `adjust_derivative(&*basis, 3, u, [&mut d1, &mut d2, &mut d3, &mut d4])?`.
pub fn adjust_derivative<C: OffsetBasisCurve + ?Sized>(
    curve: &C,
    max_derivative: i32,
    u: f64,
    derivatives: [&mut Vector2; 4],
) -> Result<bool, Failure> {
    const MIN_STEP: f64 = 1e-7;
    const MAX_DERIV_ORDER: i32 = 3;
    const DIVISION_FACTOR: f64 = 1.0e-3;
    let (inf, sup) = (curve.basis_first_parameter(), curve.basis_last_parameter());
    let du: f64 = if sup >= f64::MAX || inf <= -f64::MAX {
        0.0
    } else {
        sup - inf
    };
    let delta: f64 = std_max(du * DIVISION_FACTOR, MIN_STEP);
    // Derivative is approximated by Taylor-series.
    let mut index: i32 = 1;
    let mut v: Vector2;
    loop {
        index += 1;
        v = curve.basis_dn(u, index)?;
        if !(v.square_magnitude() <= resolution() && index < MAX_DERIV_ORDER) {
            break;
        }
    }
    let other: f64 = if u - inf < delta {
        u + delta
    } else {
        u - delta
    };
    let p1: Point2 = curve.basis_d0(std_min(u, other))?;
    let p2: Point2 = curve.basis_d0(std_max(u, other))?;
    let is_direction_change: bool = v.dot(Vector2::between(p1, p2)) < 0.0;
    let sign: f64 = if is_direction_change { -1.0 } else { 1.0 };
    let [d1, rest @ ..] = derivatives;
    *d1 = v * sign;
    for (i, target) in (1..max_derivative).zip(rest) {
        *target = curve.basis_dn(u, index + i)? * sign;
    }
    Ok(is_direction_change)
}

/// Offset point from the basis D1 (`EvaluateD0`); `evaluate_d0(u, &*basis, 2.0)?`.
pub fn evaluate_d0<C: OffsetBasisCurve + ?Sized>(
    u: f64,
    basis: &C,
    offset: f64,
) -> Result<Option<Point2>, Failure> {
    let result: Geom2dCurveD1 = basis.basis_d1(u)?;
    let mut value: Point2 = result.point;
    Ok(calculate_d0(&mut value, result.d1, offset)?.then_some(value))
}

/// Offset point and D1 from the basis D2 (`EvaluateD1`); `evaluate_d1(u, &*basis, 2.0)?`.
pub fn evaluate_d1<C: OffsetBasisCurve + ?Sized>(
    u: f64,
    basis: &C,
    offset: f64,
) -> Result<Option<Geom2dCurveD1>, Failure> {
    let result: Geom2dCurveD2 = basis.basis_d2(u)?;
    let (mut point, mut d1) = (result.point, result.d1);
    Ok(calculate_d1(&mut point, &mut d1, result.d2, offset).then_some(Geom2dCurveD1 { point, d1 }))
}

/// Offset point, D1 and D2 from the basis D3, adjusting a singular D1
/// (`EvaluateD2`); `evaluate_d2(u, &*basis, 2.0)?`.
pub fn evaluate_d2<C: OffsetBasisCurve + ?Sized>(
    u: f64,
    basis: &C,
    offset: f64,
) -> Result<Option<Geom2dCurveD2>, Failure> {
    let result: Geom2dCurveD3 = basis.basis_d3(u)?;
    let (mut point, mut d1, mut d2, mut d3) = (result.point, result.d1, result.d2, result.d3);
    let mut is_direction_change: bool = false;
    if d1.square_magnitude() <= resolution() {
        let mut dummy_d4 = Vector2::default();
        is_direction_change =
            adjust_derivative(basis, 3, u, [&mut d1, &mut d2, &mut d3, &mut dummy_d4])?;
    }
    let computed: bool = calculate_d2(
        &mut point,
        &mut d1,
        &mut d2,
        d3,
        is_direction_change,
        offset,
    );
    Ok(computed.then_some(Geom2dCurveD2 { point, d1, d2 }))
}

/// Offset point and three derivatives from the basis D3 and D4, adjusting a singular
/// D1 (`EvaluateD3`); `evaluate_d3(u, &*basis, 2.0)?`.
pub fn evaluate_d3<C: OffsetBasisCurve + ?Sized>(
    u: f64,
    basis: &C,
    offset: f64,
) -> Result<Option<Geom2dCurveD3>, Failure> {
    let result: Geom2dCurveD3 = basis.basis_d3(u)?;
    let (mut point, mut d1, mut d2, mut d3) = (result.point, result.d1, result.d2, result.d3);
    let mut d4: Vector2 = basis.basis_dn(u, 4)?;
    let mut is_direction_change: bool = false;
    if d1.square_magnitude() <= resolution() {
        is_direction_change = adjust_derivative(basis, 4, u, [&mut d1, &mut d2, &mut d3, &mut d4])?;
    }
    let derivatives = [&mut d1, &mut d2, &mut d3];
    let computed: bool = calculate_d3(&mut point, derivatives, d4, is_direction_change, offset);
    Ok(computed.then_some(Geom2dCurveD3 { point, d1, d2, d3 }))
}

/// Offset derivative of order `n`: D1..D3 through the functions above, the basis
/// derivative beyond (`EvaluateDN`); `evaluate_dn(u, &*basis, 2.0, 2)?`.
pub fn evaluate_dn<C: OffsetBasisCurve + ?Sized>(
    u: f64,
    basis: &C,
    offset: f64,
    n: i32,
) -> Result<Option<Vector2>, Failure> {
    Ok(match n {
        1 => evaluate_d1(u, basis, offset)?.map(|result| result.d1),
        2 => evaluate_d2(u, basis, offset)?.map(|result| result.d2),
        3 => evaluate_d3(u, basis, offset)?.map(|result| result.d3),
        _ => Some(basis.basis_dn(u, n)?),
    })
}
