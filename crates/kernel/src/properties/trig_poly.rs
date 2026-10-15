// SPDX-License-Identifier: GPL-2.0-only
//! Trigonometric polynomials: the profile family of spheres, tori and circular planar trims.
use super::univariate::{TermSum, Univariate};

/// `sum_k cosines[k] cos(k t) + sines[k] sin(k t)`; `sines[0]` is always zero.
/// Example: `TrigPoly::harmonic(1, 2.0, 0.0)` is `2 cos t`.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TrigPoly {
    cosines: Vec<f64>,
    sines: Vec<f64>,
}

impl TrigPoly {
    /// `cosine cos(k t) + sine sin(k t)`; `TrigPoly::harmonic(2, 0.0, 1.0)` is `sin 2t`.
    pub(crate) fn harmonic(k: usize, cosine: f64, sine: f64) -> Self {
        let mut poly = Self::zero(k + 1);
        poly.cosines[k] = cosine;
        if k > 0 {
            poly.sines[k] = sine;
        }
        poly
    }

    fn zero(length: usize) -> Self {
        Self {
            cosines: vec![0.0; length],
            sines: vec![0.0; length],
        }
    }

    fn degree(&self) -> usize {
        self.cosines.len() - 1
    }

    /// Product-to-sum of one harmonic pair into `target` at `|i-j|` and `i+j`.
    fn accumulate_pair(&self, other: &Self, (i, j): (usize, usize), target: &mut Self) {
        let (ca, sa, cb, sb) = (
            self.cosines[i],
            self.sines[i],
            other.cosines[j],
            other.sines[j],
        );
        let sum = i + j;
        target.cosines[sum] += 0.5 * (ca * cb - sa * sb);
        target.sines[sum] += 0.5 * (sa * cb + ca * sb);
        let (difference, sign) = if i >= j { (i - j, 1.0) } else { (j - i, -1.0) };
        target.cosines[difference] += 0.5 * (ca * cb + sa * sb);
        // sin(a)cos(b) - cos(a)sin(b) = sin(a - b); its sign follows the ordered difference.
        target.sines[difference] += 0.5 * sign * (sa * cb - ca * sb);
        target.sines[0] = 0.0;
    }

    fn antiderivative_terms(&self, t: f64) -> TermSum {
        let mut sum = TermSum::term(self.cosines[0] * t);
        for k in 1..self.cosines.len() {
            let kt = k as f64 * t;
            let scale = 1.0 / k as f64;
            sum = sum.plus(TermSum::term(self.cosines[k] * kt.sin() * scale));
            sum = sum.plus(TermSum::term(-self.sines[k] * kt.cos() * scale));
        }
        sum
    }
}

impl Univariate for TrigPoly {
    fn constant(value: f64) -> Self {
        Self::harmonic(0, value, 0.0)
    }

    fn add(&self, other: &Self) -> Self {
        let mut sum = Self::zero(self.degree().max(other.degree()) + 1);
        for part in [self, other] {
            for k in 0..part.cosines.len() {
                sum.cosines[k] += part.cosines[k];
                sum.sines[k] += part.sines[k];
            }
        }
        sum
    }

    fn mul(&self, other: &Self) -> Self {
        let mut product = Self::zero(self.degree() + other.degree() + 1);
        for i in 0..=self.degree() {
            for j in 0..=other.degree() {
                self.accumulate_pair(other, (i, j), &mut product);
            }
        }
        product
    }

    fn scale(&self, factor: f64) -> Self {
        Self {
            cosines: self.cosines.iter().map(|c| c * factor).collect(),
            sines: self.sines.iter().map(|s| s * factor).collect(),
        }
    }

    fn derivative(&self) -> Self {
        let mut derived = Self::zero(self.cosines.len());
        for k in 1..self.cosines.len() {
            derived.cosines[k] = k as f64 * self.sines[k];
            derived.sines[k] = -(k as f64) * self.cosines[k];
        }
        derived
    }

    fn eval(&self, t: f64) -> f64 {
        let terms = self.cosines.iter().zip(&self.sines).enumerate();
        terms.fold(0.0, |sum, (k, (c, s))| {
            let kt = k as f64 * t;
            sum + c * kt.cos() + s * kt.sin()
        })
    }

    fn integral(&self, from: f64, to: f64) -> TermSum {
        let upper = self.antiderivative_terms(to);
        upper.plus(self.antiderivative_terms(from).times(-1.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() <= 1e-14 * (1.0 + b.abs())
    }

    #[test]
    fn products_match_pointwise_products() {
        let a = TrigPoly::harmonic(1, 2.0, -1.0).add(&TrigPoly::constant(0.5));
        let b = TrigPoly::harmonic(2, 0.3, 0.7).add(&TrigPoly::harmonic(1, -1.0, 4.0));
        let product = a.mul(&b);
        for t in [-2.0, -0.3, 0.0, 0.9, 3.1] {
            assert!(close(product.eval(t), a.eval(t) * b.eval(t)), "t = {t}");
        }
    }

    #[test]
    fn integrals_and_derivatives_match_closed_forms() {
        // Integral of cos^2 over a period is pi; of sin t cos t over [0, pi/2] is 1/2.
        let cosine = TrigPoly::harmonic(1, 1.0, 0.0);
        assert!(close(cosine.mul(&cosine).integral(0.0, 2.0 * PI).value, PI));
        let sine = TrigPoly::harmonic(1, 0.0, 1.0);
        assert!(close(sine.mul(&cosine).integral(0.0, PI / 2.0).value, 0.5));
        assert!(close(sine.derivative().eval(0.4), 0.4f64.cos()));
        assert!(close(cosine.scale(3.0).eval(0.0), 3.0));
    }
}
