// SPDX-License-Identifier: GPL-2.0-only
//! Capture source coefficients before algebra; normalize homogeneous components together.
use super::super::{RootIsolationError, bernstein, error::poll};
use super::polynomial::ExactCoefficientPolynomial;
use crate::predicates::{NonFiniteCoordinate, exact_coordinate, exact_point};
use num_bigint::{BigInt, Sign};

#[derive(Clone, Debug, PartialEq)]
pub(super) struct OwnedBezierSource {
    pub controls: Vec<[f64; 2]>,
    pub weights: Option<Vec<f64>>,
    pub interval: [f64; 2],
}
#[derive(Clone, Debug, PartialEq)]
/// Original capture or exact homogeneous-conversion cause.
pub enum PairSourceFailure {
    /// Captured curve ordinal and unchanged original capture cause.
    Curve {
        /// Input curve ordinal, zero or one.
        curve_index: usize,
        /// Unchanged original failure.
        cause: Box<PairSourceFailure>,
    },
    /// Empty controls; expected at least one Bernstein control.
    EmptyControls,
    /// Weight count differs from the actual control count.
    WeightCount {
        /// Captured number of controls.
        controls: usize,
        /// Offending supplied weight count.
        weights: usize,
    },
    /// Nonfinite coordinate with original point/axis/value provenance.
    Coordinate(NonFiniteCoordinate),
    /// Nonfinite homogeneous weight.
    Weight {
        /// Original control index.
        index: usize,
        /// Offending weight value.
        value: f64,
    },
    /// Original exact arithmetic or cancellation cause.
    Algebra(RootIsolationError),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct HomogeneousPolynomials {
    pub components: [ExactCoefficientPolynomial; 3],
}
fn check_weight_count(controls: usize, weights: Option<&[f64]>) -> Result<(), PairSourceFailure> {
    if let Some(weights) = weights
        && weights.len() != controls
    {
        return Err(PairSourceFailure::WeightCount {
            controls,
            weights: weights.len(),
        });
    }
    Ok(())
}
impl OwnedBezierSource {
    pub fn capture(
        controls: &[[f64; 2]],
        weights: Option<&[f64]>,
        interval: [f64; 2],
    ) -> Result<Self, PairSourceFailure> {
        if controls.is_empty() {
            return Err(PairSourceFailure::EmptyControls);
        }
        check_weight_count(controls.len(), weights)?;
        Ok(Self {
            controls: controls.to_vec(),
            weights: weights.map(<[f64]>::to_vec),
            interval,
        })
    }
    pub fn homogeneous(
        &self,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<HomogeneousPolynomials, PairSourceFailure> {
        let controls = self.homogeneous_controls(cancelled)?;
        let mut components = Vec::with_capacity(3);
        for component in controls {
            let powers = bernstein::power_coefficients(component, cancelled)
                .map_err(PairSourceFailure::Algebra)?;
            components.push(ExactCoefficientPolynomial::new(powers));
        }
        let components: [ExactCoefficientPolynomial; 3] =
            components.try_into().expect("three homogeneous components");
        Ok(HomogeneousPolynomials { components }.reduce_shared_content())
    }
    fn homogeneous_controls(
        &self,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<[Vec<BigInt>; 3], PairSourceFailure> {
        let unit = exact_coordinate(1., 0, 0).expect("finite unit in existing coordinate scale");
        let mut components: [Vec<BigInt>; 3] =
            std::array::from_fn(|_| Vec::with_capacity(self.controls.len()));
        for (index, control) in self.controls.iter().enumerate() {
            poll(cancelled).map_err(PairSourceFailure::Algebra)?;
            let point = exact_point(*control, index).map_err(PairSourceFailure::Coordinate)?;
            let weight = self.exact_weight(index)?;
            components[0].push(&point[0] * &weight);
            components[1].push(&point[1] * &weight);
            components[2].push(&unit * &weight);
        }
        Ok(components)
    }
    fn exact_weight(&self, index: usize) -> Result<BigInt, PairSourceFailure> {
        let value = self.weights.as_ref().map_or(1., |weights| weights[index]);
        exact_coordinate(value, index, 0).map_err(|_| PairSourceFailure::Weight { index, value })
    }
}
impl HomogeneousPolynomials {
    fn reduce_shared_content(mut self) -> Self {
        // Independent primitive reduction of Nx/Ny/D would change the source curve.
        let content = self
            .components
            .iter()
            .flat_map(|p| &p.0)
            .fold(BigInt::from(0), positive_gcd);
        if content.sign() == Sign::NoSign {
            return self;
        }
        for component in &mut self.components {
            for coefficient in &mut component.0 {
                *coefficient /= &content;
            }
        }
        self
    }
}
fn positive_gcd(left: BigInt, right: &BigInt) -> BigInt {
    let mut left = if left.sign() == Sign::Minus {
        -left
    } else {
        left
    };
    let mut right = if right.sign() == Sign::Minus {
        -right
    } else {
        right.clone()
    };
    while right.sign() != Sign::NoSign {
        let remainder = left % &right;
        left = right;
        right = remainder;
    }
    left
}

pub(super) fn capture_context(curve_index: usize, cause: PairSourceFailure) -> PairSourceFailure {
    PairSourceFailure::Curve {
        curve_index,
        cause: Box::new(cause),
    }
}
impl std::fmt::Display for PairSourceFailure {
    fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            output,
            "{self:?}; expected finite nonempty Bernstein controls with one weight per control"
        )
    }
}
impl std::error::Error for PairSourceFailure {}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_homogeneous_power_components_preserve_nonuniform_rational_weights() {
        let source =
            OwnedBezierSource::capture(&[[1., 0.], [0.5, 1.]], Some(&[1., 2.]), [0., 1.]).unwrap();
        let result = source.homogeneous(&mut || false).unwrap();
        assert_eq!(result.components[0].0, vec![1.into()]);
        assert_eq!(result.components[1].0, vec![0.into(), 2.into()]);
        assert_eq!(result.components[2].0, vec![1.into(), 1.into()]);
    }
    #[test]
    fn source_capture_isolated_from_overwritten_inputs_and_denominator_sign() {
        let mut controls = [[0., 0.], [0.5, 0.], [1., 1.]];
        let mut weights = [-1.; 3];
        let captured = OwnedBezierSource::capture(&controls, Some(&weights), [0., 1.]).unwrap();
        controls.fill([999., 999.]);
        weights.fill(999.);
        let result = captured.homogeneous(&mut || false).unwrap();
        assert_eq!(result.components[0].0, vec![0.into(), (-1).into()]);
        assert_eq!(
            result.components[1].0,
            vec![0.into(), 0.into(), (-1).into()]
        );
        assert_eq!(result.components[2].0, vec![(-1).into()]);
        assert_eq!(captured.interval, [0., 1.]);
    }
    #[test]
    fn invalid_counts_preserve_offending_cardinality() {
        assert_eq!(
            OwnedBezierSource::capture(&[], None, [0., 1.]),
            Err(PairSourceFailure::EmptyControls)
        );
        assert_eq!(
            OwnedBezierSource::capture(&[[0., 0.]], Some(&[]), [0., 1.]),
            Err(PairSourceFailure::WeightCount {
                controls: 1,
                weights: 0
            })
        );
    }
    #[test]
    fn nonfinite_coordinates_weights_and_cancellation_preserve_typed_causes() {
        let invalid = OwnedBezierSource::capture(&[[f64::INFINITY, 0.]], None, [0., 1.]).unwrap();
        assert!(matches!(
            invalid.homogeneous(&mut || false),
            Err(PairSourceFailure::Coordinate(_))
        ));
        let weight =
            OwnedBezierSource::capture(&[[0., 0.]], Some(&[f64::INFINITY]), [0., 1.]).unwrap();
        assert_eq!(
            weight.homogeneous(&mut || false),
            Err(PairSourceFailure::Weight {
                index: 0,
                value: f64::INFINITY
            })
        );
        assert_eq!(
            weight.homogeneous(&mut || true),
            Err(PairSourceFailure::Algebra(RootIsolationError::Cancelled))
        );
    }
}
