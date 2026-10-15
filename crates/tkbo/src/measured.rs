// SPDX-License-Identifier: GPL-2.0-only
//! Martensite ADR 0081: one measured-value type for every public measure.
//!
//! Modelling results (`profile_extrusion`, ADR 0068) and body properties
//! (`rustycad_kernel::properties`, ADR 0073) report the same [`Measured`] value: the number,
//! an absolute error bound and the [`IntegrationMethod`] that produced it. The method says
//! how the value was computed; [`Evidence`] (derived from the method) says what the bound
//! means: a proven enclosure, an estimate, or nothing. The type lives here because the
//! modelling seam is in this crate and the kernel facade re-exports it.

/// How a value was computed, from strongest to weakest evidence.
/// Combining values keeps the weakest method (`max` of the ordering).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IntegrationMethod {
    /// Exact or interval arithmetic on the exact input: the true value lies within the bound
    /// (certified rational integrals of line-bounded regions, ADR 0066 part A).
    CertifiedEnclosure,
    /// A proven enclosure for the kernel's Bézier representation of the input, plus an
    /// estimate of that representation's rounding (arc split points and weights, spline knot
    /// insertion); the representation term is not proven.
    RepresentationEnclosure,
    /// Exact antiderivatives in binary64; the bound covers floating-point rounding only
    /// (a forward error model, not interval arithmetic).
    ClosedForm,
    /// Adaptive Gauss-Kronrod; the bound is the rule's error estimate plus rounding, not a
    /// proven enclosure.
    AdaptiveQuadrature,
    /// The ported `BRepGProp` fixed-order Gauss integration, which reports no error: the
    /// bound is infinite. (Its adaptive variant is not used: natively and in the port it
    /// reports a relative error of 7.4e-17 for an elliptic prism whose volume it misses by
    /// 4e-7.)
    NativeUnbounded,
}

/// What an error bound means.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Evidence {
    /// `|true value - value| <= error_bound` is proven.
    Proven,
    /// The bound is a model or estimate; it is usually but not provably an enclosure.
    Estimated,
    /// No bound is known (`error_bound` is infinite).
    Unbounded,
}

impl IntegrationMethod {
    /// What a bound produced by this method means; `method.evidence()`.
    ///
    /// ```
    /// use rustycad_tkbo::measured::{Evidence, IntegrationMethod};
    /// assert_eq!(IntegrationMethod::CertifiedEnclosure.evidence(), Evidence::Proven);
    /// assert_eq!(IntegrationMethod::ClosedForm.evidence(), Evidence::Estimated);
    /// ```
    pub fn evidence(self) -> Evidence {
        match self {
            Self::CertifiedEnclosure => Evidence::Proven,
            Self::RepresentationEnclosure | Self::ClosedForm | Self::AdaptiveQuadrature => {
                Evidence::Estimated
            }
            Self::NativeUnbounded => Evidence::Unbounded,
        }
    }
}

/// A scalar, its absolute error bound and the method behind the bound.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Measured {
    /// The computed value.
    pub value: f64,
    /// Absolute bound on `|true value - value|` under the method's error model.
    pub error_bound: f64,
    /// How the value was computed; `method.evidence()` says what the bound means.
    pub method: IntegrationMethod,
}

impl Measured {
    /// Whether `value` lies within `self.value +- self.error_bound`.
    ///
    /// ```
    /// use rustycad_tkbo::measured::{IntegrationMethod, Measured};
    /// let area = Measured { value: 2.0, error_bound: 1e-12, method: IntegrationMethod::CertifiedEnclosure };
    /// assert!(area.encloses(2.0) && !area.encloses(2.1));
    /// ```
    pub fn encloses(&self, value: f64) -> bool {
        (self.value - value).abs() <= self.error_bound
    }

    /// What the bound means; `measured.evidence()`.
    pub fn evidence(&self) -> Evidence {
        self.method.evidence()
    }

    /// The value times `factor` with its bound, plus one rounding of the product; the method
    /// is kept. Example: `area.scaled(height)` is a prism volume.
    pub fn scaled(&self, factor: f64) -> Measured {
        let value = self.value * factor;
        let error_bound = self.error_bound * factor.abs() + 2.0 * f64::EPSILON * value.abs();
        Measured {
            value,
            error_bound,
            method: self.method,
        }
    }
}

/// A point, the bound on its Euclidean distance to the true point, and its method.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeasuredPoint {
    /// World coordinates.
    pub point: [f64; 3],
    /// Bound on the Euclidean error of `point`.
    pub error_bound: f64,
    /// How the point was computed.
    pub method: IntegrationMethod,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn methods_order_from_proven_to_unbounded() {
        use IntegrationMethod::*;
        let order = [
            CertifiedEnclosure,
            RepresentationEnclosure,
            ClosedForm,
            AdaptiveQuadrature,
            NativeUnbounded,
        ];
        assert!(order.windows(2).all(|pair| pair[0] < pair[1]));
        let evidence = order.map(IntegrationMethod::evidence);
        assert_eq!(evidence[0], Evidence::Proven);
        assert!(evidence[1..4].iter().all(|e| *e == Evidence::Estimated));
        assert_eq!(evidence[4], Evidence::Unbounded);
        static_assertions::assert_impl_all!(Measured: Send, Sync, Copy);
    }

    #[test]
    fn scaling_keeps_the_method_and_covers_the_product_rounding() {
        let area = Measured {
            value: 3.0,
            error_bound: 1e-9,
            method: IntegrationMethod::RepresentationEnclosure,
        };
        let volume = area.scaled(-2.0);
        assert_eq!(volume.value, -6.0);
        assert!(volume.error_bound >= 2e-9);
        assert_eq!(volume.method, IntegrationMethod::RepresentationEnclosure);
        assert!(volume.encloses(-6.0 + 2e-9));
    }
}
