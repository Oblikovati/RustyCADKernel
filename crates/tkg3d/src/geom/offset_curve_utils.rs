// Rust translation of ModelingData/TKG3d/Geom/Geom_OffsetCurveUtils.pxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom_OffsetCurveUtils`: offset point and derivatives from basis derivatives.
//!
//! `P(u) = p(u) + Offset * N / |N|` with `N = p'(u) ^ V`, `V` the reference
//! direction. The formulas and their evaluation order are the native ones,
//! including the unstable fallback used when `|N|` is tiny. The `calculate_*`
//! functions return false where the native ones do (degenerate normal).
use super::basis_eval::BasisCurveEvaluator;
use super::curve::{GeomCurveD1, GeomCurveD2, GeomCurveD3};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkmath::gp::{Direction3, Point3, Vector3, Xyz, resolution};

// theValue.ChangeCoord().Add(Ndir)
fn shift(value: &mut Point3, n_dir: Xyz) {
    value.set_xyz(value.xyz() + n_dir);
}

/// Offset point: `value` enters as the basis point (`CalculateD0`, default tolerance
/// gp::Resolution()); `calculate_d0(&mut p, d1, dir, 2.0, resolution())`.
pub fn calculate_d0(
    value: &mut Point3,
    d1: Vector3,
    dir: Xyz,
    offset: f64,
    tolerance: f64,
) -> bool {
    let mut n_dir: Xyz = d1.xyz().crossed(dir);
    let r: f64 = n_dir.modulus();
    if r <= tolerance {
        return false;
    }
    n_dir *= offset / r;
    shift(value, n_dir);
    true
}

/// Offset point and first derivative; `value` and `d1` enter as the basis ones
/// (`CalculateD1`); `calculate_d1(&mut p, &mut v1, v2, dir, 2.0, resolution())`.
pub fn calculate_d1(
    value: &mut Point3,
    d1: &mut Vector3,
    d2: Vector3,
    dir: Xyz,
    offset: f64,
    tolerance: f64,
) -> bool {
    let mut n_dir: Xyz = d1.xyz().crossed(dir);
    let mut dn_dir: Xyz = d2.xyz().crossed(dir);
    let r2: f64 = n_dir.square_modulus();
    let r: f64 = r2.sqrt();
    let r3: f64 = r * r2;
    let dr: f64 = n_dir.dot(dn_dir);
    if r3 <= tolerance {
        if r2 <= tolerance {
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
    shift(value, n_dir);
    *d1 += Vector3::from(dn_dir);
    true
}

// The normal direction and its first derivatives with the scalar products that
// the D2 and D3 formulas share.
struct NormalTerms {
    n: Xyz,
    dn: Xyz,
    d2n: Xyz,
    r2: f64,
    r: f64,
    dr: f64,
    d2r: f64,
}
impl NormalTerms {
    fn new(derivatives: [Vector3; 3], dir: Xyz) -> Self {
        let [d1, d2, d3] = derivatives;
        let n: Xyz = d1.xyz().crossed(dir);
        let dn: Xyz = d2.xyz().crossed(dir);
        let d2n: Xyz = d3.xyz().crossed(dir);
        let r2: f64 = n.square_modulus();
        Self {
            n,
            dn,
            d2n,
            r2,
            r: r2.sqrt(),
            dr: n.dot(dn),
            d2r: n.dot(d2n) + dn.dot(dn),
        }
    }
}

/// Offset point and two derivatives; `value`, `d1`, `d2` enter as the basis ones
/// (`CalculateD2`); `calculate_d2(&mut p, [&mut v1, &mut v2], v3, dir, 2.0, false, resolution())`.
pub fn calculate_d2(
    value: &mut Point3,
    derivatives: [&mut Vector3; 2],
    d3: Vector3,
    dir: Xyz,
    offset: f64,
    is_dir_change: bool,
    tolerance: f64,
) -> bool {
    let [d1, d2] = derivatives;
    let t = NormalTerms::new([*d1, *d2, d3], dir);
    let (r, r2, dr, d2r) = (t.r, t.r2, t.dr, t.d2r);
    let (mut n_dir, mut dn_dir, mut d2n_dir) = (t.n, t.dn, t.d2n);
    let r3: f64 = r2 * r;
    let r4: f64 = r2 * r2;
    let r5: f64 = r3 * r2;
    if r5 <= tolerance {
        if r4 <= tolerance {
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
    shift(value, n_dir);
    *d1 += Vector3::from(dn_dir);
    if is_dir_change {
        d2.reverse();
    }
    *d2 += Vector3::from(d2n_dir);
    true
}

/// Offset point and three derivatives; `value`, `d1`, `d2`, `d3` enter as the basis
/// ones (`CalculateD3`). Quirk kept: in the unstable branch the Ndir term of D2 is
/// subtracted, where CalculateD2 adds it; and unlike Geom2d the stable branch divides
/// D2Ndir and D3Ndir by R first and multiplies by the offset last;
/// `calculate_d3(&mut p, [&mut v1, &mut v2, &mut v3], v4, dir, 2.0, false, resolution())`.
pub fn calculate_d3(
    value: &mut Point3,
    derivatives: [&mut Vector3; 3],
    d4: Vector3,
    dir: Xyz,
    offset: f64,
    is_dir_change: bool,
    tolerance: f64,
) -> bool {
    let [d1, d2, d3] = derivatives;
    let t = NormalTerms::new([*d1, *d2, *d3], dir);
    let mut n_dir: Xyz = t.n;
    let d3n_dir: Xyz = d4.xyz().crossed(dir);
    let d3r: f64 = t.n.dot(d3n_dir) + 3.0 * t.dn.dot(t.d2n);
    let Some((dn_dir, d2n_dir, d3n_dir)) = third_order_terms(&t, d3n_dir, d3r, offset, tolerance)
    else {
        return false;
    };
    n_dir *= offset / t.r;
    shift(value, n_dir);
    *d1 += Vector3::from(dn_dir);
    *d2 += Vector3::from(d2n_dir);
    if is_dir_change {
        d3.reverse();
    }
    *d3 += Vector3::from(d3n_dir);
    true
}

// The DNdir, D2Ndir and D3Ndir offsets of CalculateD3, `None` when R6 is null.
fn third_order_terms(
    t: &NormalTerms,
    mut d3n_dir: Xyz,
    d3r: f64,
    offset: f64,
    tolerance: f64,
) -> Option<(Xyz, Xyz, Xyz)> {
    let (n_dir, mut dn_dir, mut d2n_dir) = (t.n, t.dn, t.d2n);
    let (r, r2, dr, d2r) = (t.r, t.r2, t.dr, t.d2r);
    let r3: f64 = r2 * r;
    let mut r4: f64 = r2 * r2;
    let r5: f64 = r3 * r2;
    let r6: f64 = r3 * r3;
    let r7: f64 = r5 * r2;
    if r7 <= tolerance {
        if r6 <= tolerance {
            return None;
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
        d3n_dir /= r;
        d3n_dir -= d2n_dir * (3.0 * dr / r3);
        d3n_dir -= dn_dir * (3.0 * ((d2r / r3) + (dr * dr) / r5));
        d3n_dir +=
            n_dir * (6.0 * dr * dr / r5 + 6.0 * dr * d2r / r5 - 15.0 * dr * dr * dr / r7 - d3r);
        d3n_dir *= offset;
        d2n_dir /= r;
        d2n_dir -= dn_dir * (2.0 * dr / r3);
        d2n_dir -= n_dir * ((3.0 * dr * dr / r5) - (d2r / r3));
        d2n_dir *= offset;
        dn_dir *= offset / r;
        dn_dir -= n_dir * (offset * dr / r3);
    }
    Some((dn_dir, d2n_dir, d3n_dir))
}

/// Replace near-null derivatives at a singular point by the first non-null higher
/// derivative, oriented by a chord of Taylor step `max(range * 1e-3, 1e-7)`;
/// `derivatives` holds D1..D4 and only D1..D`max_derivative` are written. Returns
/// whether the direction changed (`AdjustDerivative`, whose native bool is always true);
/// `adjust_derivative(&*basis, 3, u, [&mut d1, &mut d2, &mut d3, &mut d4])?`.
pub fn adjust_derivative<C: BasisCurveEvaluator + ?Sized>(
    curve: &C,
    max_derivative: i32,
    u: f64,
    derivatives: [&mut Vector3; 4],
) -> Result<bool, Failure> {
    const MIN_STEP: f64 = 1e-7;
    const MAX_DERIV_ORDER: i32 = 3;
    const DIVISION_FACTOR: f64 = 1.0e-3;
    let (inf, sup) = (
        curve.basis_first_parameter()?,
        curve.basis_last_parameter()?,
    );
    // RealLast() and RealFirst() are +/-DBL_MAX.
    let du: f64 = if sup >= f64::MAX || inf <= -f64::MAX {
        0.0
    } else {
        sup - inf
    };
    let delta: f64 = std_max(du * DIVISION_FACTOR, MIN_STEP);
    // Derivative is approximated by Taylor-series.
    let mut index: i32 = 1;
    let mut v: Vector3;
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
    let p1: Point3 = curve.basis_d0(std_min(u, other))?;
    let p2: Point3 = curve.basis_d0(std_max(u, other))?;
    let is_direction_change: bool = v.dot(Vector3::between(p1, p2)) < 0.0;
    let sign: f64 = if is_direction_change { -1.0 } else { 1.0 };
    let [d1, rest @ ..] = derivatives;
    *d1 = v * sign;
    for (i, target) in (1..max_derivative).zip(rest) {
        *target = curve.basis_dn(u, index + i)? * sign;
    }
    Ok(is_direction_change)
}

/// Offset point from the basis D1 (`EvaluateD0`); `None` for a degenerate normal;
/// `evaluate_d0(u, &*basis, dir, 2.0)?`.
pub fn evaluate_d0<C: BasisCurveEvaluator + ?Sized>(
    u: f64,
    basis: &C,
    dir: Direction3,
    offset: f64,
) -> Result<Option<Point3>, Failure> {
    let result: GeomCurveD1 = basis.basis_d1(u)?;
    let mut value: Point3 = result.point;
    let computed: bool = calculate_d0(&mut value, result.d1, dir.xyz(), offset, resolution());
    Ok(computed.then_some(value))
}

/// Offset point and D1 from the basis D2 (`EvaluateD1`); `evaluate_d1(u, &*basis, dir, 2.0)?`.
pub fn evaluate_d1<C: BasisCurveEvaluator + ?Sized>(
    u: f64,
    basis: &C,
    dir: Direction3,
    offset: f64,
) -> Result<Option<GeomCurveD1>, Failure> {
    let result: GeomCurveD2 = basis.basis_d2(u)?;
    let (mut point, mut d1) = (result.point, result.d1);
    let computed: bool = calculate_d1(
        &mut point,
        &mut d1,
        result.d2,
        dir.xyz(),
        offset,
        resolution(),
    );
    Ok(computed.then_some(GeomCurveD1 { point, d1 }))
}

/// Offset point, D1 and D2 from the basis D3, adjusting a singular D1
/// (`EvaluateD2`); `evaluate_d2(u, &*basis, dir, 2.0)?`.
pub fn evaluate_d2<C: BasisCurveEvaluator + ?Sized>(
    u: f64,
    basis: &C,
    dir: Direction3,
    offset: f64,
) -> Result<Option<GeomCurveD2>, Failure> {
    let result: GeomCurveD3 = basis.basis_d3(u)?;
    let (mut point, mut d1, mut d2, mut d3) = (result.point, result.d1, result.d2, result.d3);
    let mut is_direction_change: bool = false;
    if d1.square_magnitude() <= resolution() {
        let mut dummy_d4 = Vector3::default();
        let derivatives = [&mut d1, &mut d2, &mut d3, &mut dummy_d4];
        is_direction_change = adjust_derivative(basis, 3, u, derivatives)?;
    }
    let (derivatives, xyz) = ([&mut d1, &mut d2], dir.xyz());
    let computed: bool = calculate_d2(
        &mut point,
        derivatives,
        d3,
        xyz,
        offset,
        is_direction_change,
        resolution(),
    );
    Ok(computed.then_some(GeomCurveD2 { point, d1, d2 }))
}

/// Offset point and three derivatives from the basis D3 and D4, adjusting a singular
/// D1 (`EvaluateD3`); `evaluate_d3(u, &*basis, dir, 2.0)?`.
pub fn evaluate_d3<C: BasisCurveEvaluator + ?Sized>(
    u: f64,
    basis: &C,
    dir: Direction3,
    offset: f64,
) -> Result<Option<GeomCurveD3>, Failure> {
    let result: GeomCurveD3 = basis.basis_d3(u)?;
    let (mut point, mut d1, mut d2, mut d3) = (result.point, result.d1, result.d2, result.d3);
    let mut d4: Vector3 = basis.basis_dn(u, 4)?;
    let mut is_direction_change: bool = false;
    if d1.square_magnitude() <= resolution() {
        let derivatives = [&mut d1, &mut d2, &mut d3, &mut d4];
        is_direction_change = adjust_derivative(basis, 4, u, derivatives)?;
    }
    let (derivatives, xyz) = ([&mut d1, &mut d2, &mut d3], dir.xyz());
    let computed: bool = calculate_d3(
        &mut point,
        derivatives,
        d4,
        xyz,
        offset,
        is_direction_change,
        resolution(),
    );
    Ok(computed.then_some(GeomCurveD3 { point, d1, d2, d3 }))
}

/// Offset derivative of order `n`: D1..D3 through the functions above, the basis
/// derivative beyond (`EvaluateDN`); `evaluate_dn(u, &*basis, dir, 2.0, 2)?`.
///
/// Quirk kept for `n == 3`: see [`evaluate_d3_aliased`].
pub fn evaluate_dn<C: BasisCurveEvaluator + ?Sized>(
    u: f64,
    basis: &C,
    dir: Direction3,
    offset: f64,
    n: i32,
) -> Result<Option<Vector3>, Failure> {
    Ok(match n {
        1 => evaluate_d1(u, basis, dir, offset)?.map(|result| result.d1),
        2 => evaluate_d2(u, basis, dir, offset)?.map(|result| result.d2),
        3 => evaluate_d3_aliased(u, basis, dir, offset)?,
        _ => Some(basis.basis_dn(u, n)?),
    })
}

/// The third derivative as native `EvaluateDN` computes it: it passes one dummy vector
/// as both `theD1` and `theD2` of `EvaluateD3`, so the basis D2 overwrites D1. The
/// singularity test, AdjustDerivative and the normal of CalculateD3 then all read the
/// basis second derivative where D1 belongs (a line offset raises there). Only the D3
/// output is kept, which the aliased writes do not reach; the same quirk as Geom2d;
/// `evaluate_d3_aliased(u, &*basis, dir, 2.0)?`.
pub fn evaluate_d3_aliased<C: BasisCurveEvaluator + ?Sized>(
    u: f64,
    basis: &C,
    dir: Direction3,
    offset: f64,
) -> Result<Option<Vector3>, Failure> {
    let result: GeomCurveD3 = basis.basis_d3(u)?;
    let (mut point, mut dummy, mut d3) = (result.point, result.d2, result.d3);
    let mut d4: Vector3 = basis.basis_dn(u, 4)?;
    let mut is_direction_change: bool = false;
    if dummy.square_magnitude() <= resolution() {
        // AdjustDerivative writes theD1 first and then theD2, the same vector: the
        // tangent it finds is overwritten by the next derivative.
        let mut overwritten = Vector3::default();
        let slots = [&mut overwritten, &mut dummy, &mut d3, &mut d4];
        is_direction_change = adjust_derivative(basis, 4, u, slots)?;
    }
    // CalculateD3 reads theD1 and theD2 before writing either, so two copies of the
    // dummy reproduce the aliased reads.
    let (mut first, mut second) = (dummy, dummy);
    let derivatives = [&mut first, &mut second, &mut d3];
    let xyz = dir.xyz();
    let computed: bool = calculate_d3(
        &mut point,
        derivatives,
        d4,
        xyz,
        offset,
        is_direction_change,
        resolution(),
    );
    Ok(computed.then_some(d3))
}
