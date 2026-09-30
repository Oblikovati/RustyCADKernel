// Rust translation of ModelingData/TKG2d/Geom2dEval/Geom2dEval_AHTBezierCurve.cxx and ModelingData/TKG3d/GeomEval/GeomEval_AHTBezierCurve.cxx (evalPolynomialNonRational, evalPolynomialRational, evalMixedAHT, EvalD1..EvalD3) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! AHT-Bezier evaluation of orders 0..3: Horner sums for a purely algebraic basis,
//! direct basis sums for a mixed basis, and the rational quotients.
use super::sums::{BasisDerivatives, BasisSums};
use super::{EvalCoord, EvalPole};

/// The number of basis functions (`basisDimension` of the AHT-Bezier curves and
/// surfaces): `k + 1`, plus 2 for a hyperbolic pair when alpha > 0 and 2 for a trigonometric pair when beta > 0;
/// `basis_dimension(1, 1.0, 1.0) == 6`.
pub fn basis_dimension(alg_degree: i32, alpha: f64, beta: f64) -> i32 {
    alg_degree + 1 + if alpha > 0.0 { 2 } else { 0 } + if beta > 0.0 { 2 } else { 0 }
}

/// The poles, the optional weights and the basis parameters of one curve, as slices
/// in storage order (the native code indexes `&Value(Lower())` pointers).
#[derive(Clone, Copy)]
pub struct AhtView<'a, P: EvalPole> {
    /// The poles in storage order.
    pub poles: &'a [P],
    /// The weights in storage order; `None` for a non-rational curve.
    pub weights: Option<&'a [f64]>,
    /// The algebraic degree k.
    pub alg_degree: usize,
    /// The hyperbolic frequency, 0 without the sinh/cosh pair.
    pub alpha: f64,
    /// The trigonometric frequency, 0 without the sin/cos pair.
    pub beta: f64,
}

/// `evalPolynomialNonRational<MAX>` / `evalPolynomialRational<MAX>`: Horner's scheme on
/// `sum P_k t^k`, with its derivatives up to MAX; `polynomial::<1>(&view, 0.5)`.
pub fn polynomial<const MAX: usize, P: EvalPole>(
    view: &AhtView<'_, P>,
    u: f64,
) -> BasisSums<P::Coord> {
    let degree: usize = view.alg_degree;
    let mut s = BasisSums::default();
    let weight = |k: usize| view.weights.map_or(1.0, |w| w[k]);
    s.n[0] = match view.weights {
        None => view.poles[degree].coord(),
        Some(_) => view.poles[degree].coord() * weight(degree),
    };
    s.w[0] = view.weights.map_or(0.0, |_| weight(degree));
    for k in (0..degree).rev() {
        horner_derivative_steps::<MAX, _>(&mut s, u, view.weights.is_some());
        s.n[0] = match view.weights {
            None => s.n[0] * u + view.poles[k].coord(),
            Some(_) => s.n[0] * u + view.poles[k].coord() * weight(k),
        };
        if view.weights.is_some() {
            s.w[0] = s.w[0] * u + weight(k);
        }
    }
    s
}

// `N3 = N3 u + 3 N2`, `N2 = N2 u + 2 N1`, `N1 = N1 u + N0` (and the W twins), highest first.
fn horner_derivative_steps<const MAX: usize, C: EvalCoord>(
    s: &mut BasisSums<C>,
    u: f64,
    rational: bool,
) {
    for d in (1..=MAX.min(3)).rev() {
        let factor: f64 = d as f64;
        s.n[d] = if d == 1 {
            s.n[1] * u + s.n[0]
        } else {
            s.n[d] * u + s.n[d - 1] * factor
        };
        if rational {
            s.w[d] = if d == 1 {
                s.w[1] * u + s.w[0]
            } else {
                s.w[d] * u + s.w[d - 1] * factor
            };
        }
    }
}

// The algebraic basis `t^k` and its derivatives 1..3 from the running powers.
fn power_derivatives(k: usize, powers: &[f64; 4]) -> BasisDerivatives {
    let kf: f64 = k as f64;
    let b1: f64 = if k >= 1 { kf * powers[1] } else { 0.0 };
    let b2: f64 = if k >= 2 {
        kf * (k - 1) as f64 * powers[2]
    } else {
        0.0
    };
    let b3: f64 = if k >= 3 {
        kf * (k - 1) as f64 * (k - 2) as f64 * powers[3]
    } else {
        0.0
    };
    [powers[0], b1, b2, b3]
}

/// `evalMixedAHT<MAX, rational>`: the algebraic part `1, t, ..., t^k`, then the
/// `sinh/cosh(alpha t)` pair when alpha > 0, then the `sin/cos(beta t)` pair when
/// beta > 0; `mixed::<2>(&view, 0.5)`.
pub fn mixed<const MAX: usize, P: EvalPole>(view: &AhtView<'_, P>, u: f64) -> BasisSums<P::Coord> {
    let mut s = BasisSums::default();
    // powers = [t^k, t^(k-1), t^(k-2), t^(k-3)], shifted after each term.
    let mut powers: [f64; 4] = [1.0, 0.0, 0.0, 0.0];
    for k in 0..=view.alg_degree {
        let weight: Option<f64> = view.weights.map(|w| w[k]);
        s.add_term::<MAX>(
            view.poles[k].coord(),
            weight,
            &power_derivatives(k, &powers),
        );
        powers = [powers[0] * u, powers[0], powers[1], powers[2]];
    }
    let mut index: usize = view.alg_degree + 1;
    if view.alpha > 0.0 {
        add_pair_at::<MAX, _>(&mut s, view, index, hyperbolic_pair(view.alpha, u));
        index += 2;
    }
    if view.beta > 0.0 {
        add_pair_at::<MAX, _>(&mut s, view, index, trigonometric_pair(view.beta, u));
    }
    s
}

fn add_pair_at<const MAX: usize, P: EvalPole>(
    s: &mut BasisSums<P::Coord>,
    view: &AhtView<'_, P>,
    index: usize,
    basis: [BasisDerivatives; 2],
) {
    let poles: [P::Coord; 2] = [view.poles[index].coord(), view.poles[index + 1].coord()];
    let weights: Option<[f64; 2]> = view.weights.map(|w| [w[index], w[index + 1]]);
    s.add_pair::<MAX>(poles, weights, [&basis[0], &basis[1]]);
}

// sinh(a t), cosh(a t) and their derivatives 1..3.
fn hyperbolic_pair(alpha: f64, u: f64) -> [BasisDerivatives; 2] {
    let (sinh, cosh) = ((alpha * u).sinh(), (alpha * u).cosh());
    let alpha2: f64 = alpha * alpha;
    let alpha3: f64 = alpha2 * alpha;
    [
        [sinh, alpha * cosh, alpha2 * sinh, alpha3 * cosh],
        [cosh, alpha * sinh, alpha2 * cosh, alpha3 * sinh],
    ]
}

// sin(b t), cos(b t) and their derivatives 1..3.
fn trigonometric_pair(beta: f64, u: f64) -> [BasisDerivatives; 2] {
    let (sin, cos) = ((beta * u).sin(), (beta * u).cos());
    let beta2: f64 = beta * beta;
    let beta3: f64 = beta2 * beta;
    [
        [sin, beta * cos, -beta2 * sin, -beta3 * cos],
        [cos, -beta * sin, -beta2 * cos, beta3 * sin],
    ]
}

/// The rational point and first derivative of EvalD1 and EvalD2:
/// `C = N0 / W0` and `C' = (N1 W0 - N0 W1) / W0^2`, through `1 / W0`;
/// `quotient_d1(&sums)`.
pub fn quotient_d1<C: EvalCoord>(s: &BasisSums<C>) -> (C, C) {
    let inv_w: f64 = 1.0 / s.w[0];
    let c: C = s.n[0] * inv_w;
    let d1: C = (s.n[1] * s.w[0] - s.n[0] * s.w[1]) * (inv_w * inv_w);
    (c, d1)
}

/// The rational second derivative from the point and first derivative;
/// `quotient_d2(&sums, c, d1)`.
pub fn quotient_d2<C: EvalCoord>(s: &BasisSums<C>, c: C, d1: C) -> C {
    let inv_w: f64 = 1.0 / s.w[0];
    (s.n[2] - d1 * (2.0 * s.w[1]) - c * s.w[2]) * inv_w
}

/// EvalD3's rational chain: unlike EvalD1 and EvalD2 it computes `C' = (N1 - C W1) / W0`,
/// so its first derivative can differ from theirs in the last bits;
/// `quotient_d3(&sums)`.
pub fn quotient_d3<C: EvalCoord>(s: &BasisSums<C>) -> [C; 4] {
    let inv_w: f64 = 1.0 / s.w[0];
    let c: C = s.n[0] * inv_w;
    let d1: C = (s.n[1] - c * s.w[1]) * inv_w;
    let d2: C = quotient_d2(s, c, d1);
    let d3: C = (s.n[3] - d2 * (3.0 * s.w[1]) - d1 * (3.0 * s.w[2]) - c * s.w[3]) * inv_w;
    [c, d1, d2, d3]
}
