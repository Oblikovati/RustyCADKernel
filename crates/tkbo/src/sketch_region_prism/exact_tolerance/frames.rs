// SPDX-License-Identifier: GPL-2.0-only
//! Invertibility concerns exact operands, never a rounded scale-times-matrix product.
use rustycad_tkmath::predicates::roots::metric_bounds::MetricEmbedding;
use rustycad_tkmath::predicates::{NonFiniteCoordinate, compare_scalar_triple_product_3d};
use std::cmp::Ordering;

pub(super) struct UsableToleranceFrame {
    pub placement: MetricEmbedding<3>,
    pub unscaled_determinant_sign: Ordering,
}

#[derive(Debug, PartialEq)]
pub(super) enum ToleranceFrameFailure {
    Origin { axis: usize, actual: f64 },
    Scale { actual: f64 },
    Matrix(NonFiniteCoordinate),
    Singular { actual: [[f64; 3]; 3] },
}

/// Certify usable exact placement operands; example: `admit_tolerance_frame(placement)`.
pub(super) fn admit_tolerance_frame(
    placement: MetricEmbedding<3>,
) -> Result<UsableToleranceFrame, ToleranceFrameFailure> {
    finite_origin(placement.origin)?;
    nonzero_scale(placement.scale)?;
    let [first, second, third] = placement.linear;
    let sign = compare_scalar_triple_product_3d(first, second, third)
        .map_err(ToleranceFrameFailure::Matrix)?;
    if sign == Ordering::Equal {
        return Err(ToleranceFrameFailure::Singular {
            actual: placement.linear,
        });
    }
    Ok(UsableToleranceFrame {
        placement,
        unscaled_determinant_sign: sign,
    })
}

fn finite_origin(origin: [f64; 3]) -> Result<(), ToleranceFrameFailure> {
    for (axis, actual) in origin.into_iter().enumerate() {
        if !actual.is_finite() {
            return Err(ToleranceFrameFailure::Origin { axis, actual });
        }
    }
    Ok(())
}

fn nonzero_scale(actual: f64) -> Result<(), ToleranceFrameFailure> {
    if !actual.is_finite() || actual == 0. {
        return Err(ToleranceFrameFailure::Scale { actual });
    }
    Ok(())
}

impl std::fmt::Display for ToleranceFrameFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Origin { axis, actual } => write!(
                formatter,
                "placement origin axis {axis} is {actual:?}; expected finite coordinate"
            ),
            Self::Scale { actual } => write!(
                formatter,
                "placement scale {actual:?}; expected finite nonzero scalar"
            ),
            Self::Matrix(cause) => write!(formatter, "placement matrix: {cause}"),
            Self::Singular { actual } => write!(
                formatter,
                "placement matrix {actual:?}; expected nonzero exact determinant"
            ),
        }
    }
}

impl std::error::Error for ToleranceFrameFailure {}
