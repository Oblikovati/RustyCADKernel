// Rust translation of ModelingData/TKG2d/Geom2dEval/Geom2dEval_TBezierCurve.cxx and ModelingData/TKG3d/GeomEval/GeomEval_TBezierCurve.cxx (evalCurveBasis, evalBasis, evalBasisDeriv, EvalD0..EvalDN) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! T-Bezier evaluation: the basis sums of orders 0..3, the rational quotients, and
//! the order-N derivative (direct for non-rational curves, Leibniz recurrence for
//! rational ones).
use super::sums::{BasisDerivatives, BasisSums};
use super::{EvalCoord, EvalPole, pow_int, trig_shift};

/// The poles, the optional weights and alpha of one T-Bezier curve, as slices in
/// storage order (the native code indexes `&Value(Lower())` pointers).
#[derive(Clone, Copy)]
pub struct TBezierView<'a, P: EvalPole> {
    /// The poles in storage order.
    pub poles: &'a [P],
    /// The weights in storage order; `None` for a non-rational curve.
    pub weights: Option<&'a [f64]>,
    /// The frequency parameter.
    pub alpha: f64,
}

// The sin/cos pair of frequency k * alpha, advanced by the angle-addition recurrence.
struct TrigRecurrence {
    sin1: f64,
    cos1: f64,
    sin_k: f64,
    cos_k: f64,
}
impl TrigRecurrence {
    fn new(theta: f64) -> Self {
        let (sin1, cos1) = (theta.sin(), theta.cos());
        Self {
            sin1,
            cos1,
            sin_k: sin1,
            cos_k: cos1,
        }
    }
    // Only called while k < order, as natively, so the last pair is never advanced.
    fn advance(&mut self) {
        let sin_next: f64 = self.sin_k * self.cos1 + self.cos_k * self.sin1;
        let cos_next: f64 = self.cos_k * self.cos1 - self.sin_k * self.sin1;
        (self.sin_k, self.cos_k) = (sin_next, cos_next);
    }
}

// Derivatives 0..3 of sin(f t) and cos(f t) at the current pair.
fn pair_derivatives(freq: f64, sin: f64, cos: f64) -> [BasisDerivatives; 2] {
    let freq2: f64 = freq * freq;
    let freq3: f64 = freq2 * freq;
    [
        [sin, freq * cos, -freq2 * sin, -freq3 * cos],
        [cos, -freq * sin, -freq2 * cos, freq3 * sin],
    ]
}

// The constant basis function 1: `N0 += P0` or `N0 += P0 * w0; W0 += w0`.
fn add_constant_pole<P: EvalPole>(sums: &mut BasisSums<P::Coord>, view: &TBezierView<'_, P>) {
    let pole0: P::Coord = view.poles[0].coord();
    match view.weights {
        None => sums.n[0] += pole0,
        Some(weights) => {
            sums.n[0] += pole0 * weights[0];
            sums.w[0] += weights[0];
        }
    }
}

/// `evalCurveBasis<MAX>`: the sums of orders `0..=MAX` at `u`;
/// `curve_basis::<1>(&view, 0.5).n[1]`.
pub fn curve_basis<const MAX: usize, P: EvalPole>(
    view: &TBezierView<'_, P>,
    u: f64,
) -> BasisSums<P::Coord> {
    let mut sums = BasisSums::default();
    add_constant_pole(&mut sums, view);
    let order: usize = (view.poles.len() - 1) / 2;
    let mut trig = TrigRecurrence::new(view.alpha * u);
    let mut freq: f64 = view.alpha;
    for k in 1..=order {
        let index: usize = 2 * k - 1;
        let [b_sin, b_cos] = pair_derivatives(freq, trig.sin_k, trig.cos_k);
        let poles: [P::Coord; 2] = [view.poles[index].coord(), view.poles[index + 1].coord()];
        let weights: Option<[f64; 2]> = view.weights.map(|w| [w[index], w[index + 1]]);
        sums.add_pair::<MAX>(poles, weights, [&b_sin, &b_cos]);
        if k < order {
            trig.advance();
        }
        freq += view.alpha;
    }
    sums
}

/// The rational point `N0 / W0`; `rational_point(&sums)`.
pub fn rational_point<C: EvalCoord>(sums: &BasisSums<C>) -> C {
    sums.n[0] / sums.w[0]
}

/// The rational derivatives 1..=`count` by the quotient rule, divided by W0 as natively.
///
/// The native `2.0 * W1 * CD1` is `(2.0 * W1) * CD1`, written `CD1 * (2.0 * W1)`;
/// `rational_derivatives(&sums, c, 2)`.
pub fn rational_derivatives<C: EvalCoord>(sums: &BasisSums<C>, c: C, count: usize) -> [C; 3] {
    let (n, w) = (&sums.n, &sums.w);
    let mut d = [C::default(); 3];
    d[0] = (n[1] - c * w[1]) / w[0];
    if count >= 2 {
        d[1] = (n[2] - d[0] * (2.0 * w[1]) - c * w[2]) / w[0];
    }
    if count >= 3 {
        d[2] = (n[3] - d[0] * (3.0 * w[2]) - d[1] * (3.0 * w[1]) - c * w[3]) / w[0];
    }
    d
}

/// `EvalDN` of a non-rational curve: the poles weighted by the N-th derivative of
/// each sin/cos pair; `non_rational_dn(&view, 0.5, 4)`.
pub fn non_rational_dn<P: EvalPole>(view: &TBezierView<'_, P>, u: f64, n: i32) -> P::Coord {
    let mut sum = P::Coord::default();
    let order: usize = (view.poles.len() - 1) / 2;
    let mut trig = TrigRecurrence::new(view.alpha * u);
    let mut freq: f64 = view.alpha;
    for k in 1..=order {
        let index: usize = 2 * k - 1;
        let freq_pow: f64 = pow_int(freq, n);
        let (sin_deriv, cos_deriv) = trig_shift(trig.sin_k, trig.cos_k, n);
        sum += view.poles[index].coord() * (freq_pow * sin_deriv);
        sum += view.poles[index + 1].coord() * (freq_pow * cos_deriv);
        if k < order {
            trig.advance();
        }
        freq += view.alpha;
    }
    sum
}

/// `evalBasisDeriv` (with `evalBasis` as order 0): the value of every basis function's
/// `order`-th derivative at `u`; `basis_derivative(&view, 0.5, 0)[0] == 1.0`.
fn basis_derivative<P: EvalPole>(view: &TBezierView<'_, P>, u: f64, order: i32) -> Vec<f64> {
    let mut basis: Vec<f64> = vec![if order == 0 { 1.0 } else { 0.0 }; view.poles.len()];
    let trig_order: usize = (view.poles.len() - 1) / 2;
    let mut trig = TrigRecurrence::new(view.alpha * u);
    let mut freq: f64 = view.alpha;
    for k in 1..=trig_order {
        let (sin_value, cos_value) = if order == 0 {
            (trig.sin_k, trig.cos_k)
        } else {
            let freq_pow: f64 = pow_int(freq, order);
            let (sin_deriv, cos_deriv) = trig_shift(trig.sin_k, trig.cos_k, order & 3);
            (freq_pow * sin_deriv, freq_pow * cos_deriv)
        };
        (basis[2 * k - 1], basis[2 * k]) = (sin_value, cos_value);
        freq += view.alpha;
        if k < trig_order {
            trig.advance();
        }
    }
    basis
}

/// `EvalDN` of a rational curve above order 3: `C^(d) = (Cw^(d) - sum_k C(d,k) w^(k)
/// C^(d-k)) / w`, from the basis derivatives of every order up to N;
/// `rational_dn(&view, weights, 0.5, 5)`.
pub fn rational_dn<P: EvalPole>(
    view: &TBezierView<'_, P>,
    weights: &[f64],
    u: f64,
    n: i32,
) -> P::Coord {
    let count: usize = n as usize + 1;
    let mut w_derivs: Vec<f64> = Vec::with_capacity(count);
    let mut cw_derivs: Vec<P::Coord> = Vec::with_capacity(count);
    for d in 0..=n {
        let basis: Vec<f64> = basis_derivative(view, u, d);
        let (mut wd, mut cwd) = (0.0_f64, P::Coord::default());
        for ((&wi, &bd), pole) in weights.iter().zip(&basis).zip(view.poles) {
            // `(wi * bd) * pole`, written with the scalar on the right (same rounding).
            cwd += pole.coord() * (wi * bd);
            wd += wi * bd;
        }
        w_derivs.push(wd);
        cw_derivs.push(cwd);
    }
    let mut c_derivs: Vec<P::Coord> = vec![cw_derivs[0] / w_derivs[0]];
    for d in 1..count {
        let (mut sum, mut binom) = (P::Coord::default(), 1.0_f64);
        for k in 1..=d {
            binom *= (d - k + 1) as f64 / k as f64;
            sum += c_derivs[d - k] * (binom * w_derivs[k]);
        }
        c_derivs.push((cw_derivs[d] - sum) / w_derivs[0]);
    }
    c_derivs[n as usize]
}
