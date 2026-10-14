// SPDX-License-Identifier: GPL-2.0-only
//! Versioned bounded capture refuses excess; these fixed limits never clamp source settings.
use super::types::CaptureResult;
use super::types::PrismFreezeCause;
use crate::sketch_region_geometry_capture::CapturedMetricCurve;

pub(super) const MAXIMUM_USES: usize = 4096;
pub(super) const MAXIMUM_POLES: usize = 17;
#[derive(Clone, Copy, Debug)]
pub(crate) struct FrozenCapturePolicy {
    pub(crate) maximum_directed_occurrences: usize,
    pub(crate) maximum_polynomial_poles: usize,
    pub(crate) exact_endpoint_spatial_limit: f64,
}
pub(super) fn fixed_policy() -> FrozenCapturePolicy {
    FrozenCapturePolicy {
        maximum_directed_occurrences: MAXIMUM_USES,
        maximum_polynomial_poles: MAXIMUM_POLES,
        exact_endpoint_spatial_limit: 0.,
    }
}
pub(super) fn check_count(actual: usize) -> CaptureResult<()> {
    if actual > MAXIMUM_USES {
        return Err(PrismFreezeCause::LimitExceeded {
            quantity: "directed occurrences",
            actual,
            maximum: MAXIMUM_USES,
        });
    }
    Ok(())
}
pub(super) fn polynomial<const D: usize>(curve: &CapturedMetricCurve<D>) -> CaptureResult<()> {
    if let CapturedMetricCurve::Bezier { poles, weights } = curve {
        if let Some(weights) = weights {
            return Err(PrismFreezeCause::RationalWeights {
                actual: weights.len(),
            });
        }
        if poles.is_empty() || poles.len() > MAXIMUM_POLES {
            return Err(PrismFreezeCause::PolynomialPoles {
                actual: poles.len(),
                minimum: 1,
                maximum: MAXIMUM_POLES,
            });
        }
    }
    Ok(())
}
