// SPDX-License-Identifier: GPL-2.0-only
//! Parameter angles of conic frames: exact binary64 radians with rigorous,
//! refinable enclosures of their cosine, sine and of pi.
//!
//! A frame `(a, o, ra, rb)` places angle `theta` in the direction
//! `q(theta) = ra cos(theta) a + o rb sin(theta) a_perp` from the center. For a
//! nonzero rational `theta`, `cos`, `sin` and `tan` are transcendental
//! (Lindemann-Weierstrass), so `q(theta)` is never parallel to an algebraic
//! direction: comparisons with algebraic contacts always separate under
//! refinement, and only `theta = 0` is decided exactly.
use super::super::certificate_arithmetic::{ExactRatio, RatioInterval};
use super::rational_polynomial::ratio;
use num_bigint::BigInt;

/// Fraction bits of the first enclosure; each refinement adds 64.
const FIRST_BITS: u64 = 64;
/// Admitted angle magnitude; it bounds the Taylor series length.
pub(super) const MAX_ANGLE: f64 = 64.;

/// The frame of a conic's parameter angle.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct AngleFrame {
    pub axis: [f64; 2],
    pub counterclockwise: bool,
    pub radii: [f64; 2],
}
/// One exact angle with its current enclosure precision.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct FrameAngle {
    pub frame: AngleFrame,
    pub theta: f64,
    pub bits: u64,
}
impl FrameAngle {
    pub fn new(frame: AngleFrame, theta: f64) -> Self {
        Self {
            frame,
            theta,
            bits: FIRST_BITS,
        }
    }
    /// Add 64 fraction bits; linear growth keeps alternating refinement with a
    /// bisected root polynomial in the number of steps.
    pub fn refine(&mut self) {
        self.bits = (self.bits + 64).min(1 << 16);
    }
    /// Exact direction at `theta = 0`: `ra a`.
    pub fn exact_direction(&self) -> Option<[ExactRatio; 2]> {
        (self.theta == 0.).then(|| {
            self.frame
                .axis
                .map(|value| ratio(value).multiply(&ratio(self.frame.radii[0])))
        })
    }
    /// Rigorous enclosure of `q(theta)`.
    pub fn direction(&self) -> [RatioInterval; 2] {
        let [cosine, sine] = cos_sin(self.theta, self.bits);
        let [ax, ay] = self.frame.axis.map(ratio);
        let sense = ratio(if self.frame.counterclockwise { 1. } else { -1. });
        let along = cosine.multiply(&point(ratio(self.frame.radii[0])));
        let across = sine.multiply(&point(ratio(self.frame.radii[1]).multiply(&sense)));
        // a_perp = (-ay, ax).
        [
            along
                .multiply(&point(ax.clone()))
                .add(&across.multiply(&point(ay.negated()))),
            along.multiply(&point(ay)).add(&across.multiply(&point(ax))),
        ]
    }
    /// True when both angles name the same exact direction (same frame axis
    /// direction, same radii ratio and `o1 theta1 = o2 theta2`); any other pair of
    /// rational angles differs (Lindemann-Weierstrass).
    pub fn same_direction(&self, other: &Self) -> bool {
        let [ax, ay] = self.frame.axis;
        let [bx, by] = other.frame.axis;
        let parallel = ratio(ax).multiply(&ratio(by)) == ratio(ay).multiply(&ratio(bx));
        let pointing = ax * bx + ay * by > 0.;
        let shape = ratio(self.frame.radii[0]).multiply(&ratio(other.frame.radii[1]))
            == ratio(self.frame.radii[1]).multiply(&ratio(other.frame.radii[0]));
        let signed = |angle: &Self| {
            if angle.frame.counterclockwise {
                angle.theta
            } else {
                -angle.theta
            }
        };
        parallel && pointing && shape && signed(self) == signed(other)
    }
}
fn point(value: ExactRatio) -> RatioInterval {
    RatioInterval::point(value)
}
/// Enclosures of `cos(x)` and `sin(x)` for a binary64 `x` with `|x| <= MAX_ANGLE`.
///
/// With `x = p / q` and `N` terms, both partial sums share the integer denominator
/// `q^N N!`; the Lagrange remainder of either series is at most `|x|^N / N!`.
pub(super) fn cos_sin(x: f64, bits: u64) -> [RatioInterval; 2] {
    let value = ratio(x);
    let (p, q) = (value.numerator().clone(), value.denominator().clone());
    let count = term_count(&value, bits);
    let mut factorial = (1..=count).fold(BigInt::from(1), |total, k| total * BigInt::from(k));
    let denominator = num_pow(&q, count) * &factorial;
    let mut sums = [BigInt::from(0), BigInt::from(0)];
    let mut power = BigInt::from(1);
    for order in 0..count {
        // p^n q^(N-n) N!/n!, the n-th term over the common denominator.
        let term = &power * num_pow(&q, count - order) * &factorial;
        let negative = (order / 2) % 2 == 1;
        sums[order % 2] += if negative { -term } else { term };
        power *= &p;
        factorial /= BigInt::from(order + 1);
    }
    let remainder = remainder(&value, count);
    sums.map(|sum| widen_raw(&sum, &denominator, &remainder, bits))
}
// `sum/denominator +- remainder` rounded outward to `bits` fraction bits, with
// integer arithmetic only (no normalization of the large common denominator).
fn widen_raw(
    sum: &BigInt,
    denominator: &BigInt,
    remainder: &ExactRatio,
    bits: u64,
) -> RatioInterval {
    let scale = BigInt::from(1) << bits;
    let (rn, rd) = (remainder.numerator(), remainder.denominator());
    let common = denominator * rd;
    let center = sum * rd;
    let spread = rn * denominator;
    let lower = floor_div(&((&center - &spread) * &scale), &common);
    let upper = -floor_div(&(-((&center + &spread) * &scale)), &common);
    RatioInterval {
        lower: ExactRatio::new(lower, scale.clone()),
        upper: ExactRatio::new(upper, scale),
    }
}
fn num_pow(base: &BigInt, exponent: usize) -> BigInt {
    (0..exponent).fold(BigInt::from(1), |total, _| total * base)
}
// |x|^N / N!, exactly.
fn remainder(value: &ExactRatio, count: usize) -> ExactRatio {
    let magnitude = if value.sign() == std::cmp::Ordering::Less {
        value.negated()
    } else {
        value.clone()
    };
    let factorial = (1..=count).fold(BigInt::from(1), |total, k| total * BigInt::from(k));
    let numerator = num_pow(magnitude.numerator(), count);
    let denominator = num_pow(magnitude.denominator(), count) * factorial;
    ExactRatio::new(numerator, denominator)
}
// The smallest N >= 3 with |x|^N / N! <= 2^-(bits+2), found by an f64 estimate
// and confirmed exactly.
fn term_count(value: &ExactRatio, bits: u64) -> usize {
    let limit = ExactRatio::new(BigInt::from(1), BigInt::from(1) << (bits + 2));
    let magnitude = value_f64(value).abs().max(f64::MIN_POSITIVE);
    let mut count = 3usize;
    let mut log_term = 3. * magnitude.ln() - (1..=3).map(|k| (k as f64).ln()).sum::<f64>();
    let target = -((bits + 2) as f64) * std::f64::consts::LN_2 - 8.;
    while log_term > target {
        count += 1;
        log_term += magnitude.ln() - (count as f64).ln();
    }
    while remainder(value, count) > limit {
        count += 4;
    }
    count
}
fn value_f64(value: &ExactRatio) -> f64 {
    super::report::outward_ratio(value)[1]
}
fn floor_div(numerator: &BigInt, denominator: &BigInt) -> BigInt {
    let quotient = numerator / denominator;
    if (numerator % denominator).sign() == num_bigint::Sign::Minus {
        quotient - 1
    } else {
        quotient
    }
}
/// Enclosure of pi by Machin's formula `16 atan(1/5) - 4 atan(1/239)`.
pub(super) fn pi(bits: u64) -> RatioInterval {
    let first = arctangent_inverse(5, bits + 8);
    let second = arctangent_inverse(239, bits + 8);
    let sixteen = point(ratio(16.));
    let four = point(ratio(-4.));
    let flipped = RatioInterval {
        lower: second.upper.clone(),
        upper: second.lower.clone(),
    };
    first.multiply(&sixteen).add(&flipped.multiply(&four))
}
// atan(1/n) = sum (-1)^k / ((2k+1) n^(2k+1)); alternating with decreasing terms.
fn arctangent_inverse(n: u32, bits: u64) -> RatioInterval {
    let limit = ExactRatio::new(BigInt::from(1), BigInt::from(1) << bits);
    let base = BigInt::from(n);
    let mut sum = ratio(0.);
    let mut power = base.clone();
    let mut index = 0u64;
    loop {
        let term = ExactRatio::new(BigInt::from(1), &power * BigInt::from(2 * index + 1));
        let next = ExactRatio::new(
            BigInt::from(1),
            &power * &base * &base * BigInt::from(2 * index + 3),
        );
        sum = if index.is_multiple_of(2) {
            sum.add(&term)
        } else {
            sum.subtract(&term)
        };
        if next <= limit {
            return RatioInterval {
                lower: sum.subtract(&next),
                upper: sum.add(&next),
            };
        }
        power = power * &base * &base;
        index += 1;
    }
}
