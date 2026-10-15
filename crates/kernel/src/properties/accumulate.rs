// SPDX-License-Identifier: GPL-2.0-only
//! The eight surface integrals every face contributes, with their error bounds.
//! Index layout: area, area moment (x, y, z), normal flux, flux-weighted moment (x, y, z),
//! all relative to one reference point so that a body's faces add up consistently.
use super::measured::{IntegrationMethod, Measured, MeasuredPoint};
use super::univariate::TermSum;

/// Number of integrated quantities per face.
pub(crate) const QUANTITIES: usize = 8;
/// Index of the area integral.
pub(crate) const AREA: usize = 0;
/// First index of the area moment `integral (P - Q) dA`.
pub(crate) const AREA_MOMENT: usize = 1;
/// Index of the flux `integral (P - Q) . N du dv` (three times the signed volume).
pub(crate) const FLUX: usize = 4;
/// First index of `integral ((P - Q) . N) (P - Q) du dv` (four times the volume moment).
pub(crate) const FLUX_MOMENT: usize = 5;

/// Relative rounding allowance for one closed-form term chain: 64 unit roundoffs covers the
/// products, the libm sine/cosine and the summation of each term; a floating-point model
/// bound, not interval arithmetic.
const ROUNDING: f64 = 64.0 * f64::EPSILON;

/// Integrals with absolute error bounds and the weakest method used to form them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Moments {
    pub(crate) value: [f64; QUANTITIES],
    pub(crate) error: [f64; QUANTITIES],
    pub(crate) method: IntegrationMethod,
}

impl Default for Moments {
    fn default() -> Self {
        Self {
            value: [0.0; QUANTITIES],
            error: [0.0; QUANTITIES],
            method: IntegrationMethod::ClosedForm,
        }
    }
}

impl Moments {
    /// Adds exact term sums times `sign`; the error grows by the rounding allowance.
    /// Example: `moments.add_closed_form(&terms, -1.0)` for a reversed edge.
    pub(crate) fn add_closed_form(&mut self, terms: &[TermSum; QUANTITIES], sign: f64) {
        for (k, term) in terms.iter().enumerate() {
            self.value[k] += sign * term.value;
            self.error[k] += ROUNDING * term.magnitude;
        }
    }

    /// Adds `other` times `sign`, keeping the weaker method; `body.add(&face, -1.0)`.
    pub(crate) fn add(&mut self, other: &Moments, sign: f64) {
        for k in 0..QUANTITIES {
            self.value[k] += sign * other.value[k];
            self.error[k] += other.error[k];
        }
        self.method = self.method.max(other.method);
    }

    /// One integral as a measured value scaled by `factor`; `moments.measured(FLUX, 1.0 / 3.0)`.
    pub(crate) fn measured(&self, index: usize, factor: f64) -> Measured {
        Measured {
            value: self.value[index] * factor,
            error_bound: self.error[index] * factor.abs(),
            method: self.method,
        }
    }

    /// Adds a face: area quantities as they are, flux quantities times `orientation`
    /// (`+1` forward, `-1` reversed, `0` internal); `body.add_face(&face, -1.0)`.
    pub(crate) fn add_face(&mut self, face: &Moments, orientation: f64) {
        for k in 0..QUANTITIES {
            let sign = if k < FLUX { 1.0 } else { orientation };
            self.value[k] += sign * face.value[k];
            self.error[k] += sign.abs() * face.error[k];
        }
        self.method = self.method.max(face.method);
    }

    /// `reference + ratio * moment / mass` with the propagated bound;
    /// `moments.centroid(q, (AREA, AREA_MOMENT), 1.0)`.
    pub(crate) fn centroid(
        &self,
        reference: [f64; 3],
        (mass, moment): (usize, usize),
        ratio: f64,
    ) -> MeasuredPoint {
        let (m, dm) = (self.value[mass], self.error[mass]);
        let mut point = reference;
        let mut squared = 0.0;
        for (axis, coordinate) in point.iter_mut().enumerate() {
            let (s, ds) = (self.value[moment + axis], self.error[moment + axis]);
            *coordinate += ratio * s / m;
            let bound =
                quotient_bound((s, ds), (m, dm), ratio) + 4.0 * f64::EPSILON * coordinate.abs();
            squared += bound * bound;
        }
        let error_bound = if dm < m.abs() {
            squared.sqrt()
        } else {
            f64::INFINITY
        };
        MeasuredPoint {
            point,
            error_bound,
            method: self.method,
        }
    }
}

/// `|ratio| |s/m - s'/m'| <= |ratio| (ds + |s/m| dm) / (|m| - dm)` for `|s'-s| <= ds`,
/// `|m'-m| <= dm < |m|`; the caller adds the rounding of the quotient (4 ulps).
fn quotient_bound((s, ds): (f64, f64), (m, dm): (f64, f64), ratio: f64) -> f64 {
    ratio.abs() * (ds + (s / m).abs() * dm) / (m.abs() - dm)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centroid_bound_covers_perturbed_quotients() {
        let mut moments = Moments::default();
        moments.value[AREA] = 2.0;
        moments.error[AREA] = 1e-3;
        moments.value[AREA_MOMENT] = 4.0;
        moments.error[AREA_MOMENT] = 1e-3;
        let centroid = moments.centroid([1.0, 0.0, 0.0], (AREA, AREA_MOMENT), 1.0);
        assert_eq!(centroid.point, [3.0, 0.0, 0.0]);
        let worst = (4.0 + 1e-3) / (2.0 - 1e-3) - 2.0;
        assert!(centroid.error_bound >= worst);
        moments.error[AREA] = 3.0;
        let unbounded = moments.centroid([0.0; 3], (AREA, AREA_MOMENT), 1.0);
        assert!(unbounded.error_bound.is_infinite());
    }

    #[test]
    fn closed_form_terms_carry_rounding_and_sign() {
        let mut moments = Moments::default();
        let mut terms = [TermSum::default(); QUANTITIES];
        terms[FLUX] = TermSum::term(3.0);
        moments.add_closed_form(&terms, -1.0);
        assert_eq!(moments.value[FLUX], -3.0);
        assert!(moments.error[FLUX] > 0.0 && moments.error[FLUX] < 1e-12);
        assert_eq!(moments.measured(FLUX, 1.0 / 3.0).value, -1.0);
        let mut body = Moments::default();
        terms[AREA] = TermSum::term(2.0);
        let mut face = Moments::default();
        face.add_closed_form(&terms, 1.0);
        body.add_face(&face, -1.0);
        assert_eq!((body.value[AREA], body.value[FLUX]), (2.0, -3.0));
        body.add_face(&face, 0.0);
        assert_eq!((body.value[AREA], body.value[FLUX]), (4.0, -3.0));
    }
}
