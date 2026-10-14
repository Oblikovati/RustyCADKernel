// SPDX-License-Identifier: GPL-2.0-only
use super::super::super::polynomial::IntegerPolynomial;
use super::super::{MetricCurve, MetricEmbedding, coefficients};
use super::{
    ExactRatio, PolynomialSideFailure as Fault, PolynomialSideOptions, Vector, checkpoint,
};
use num_bigint::BigInt;

pub(super) fn admit_policy(options: PolynomialSideOptions) -> Result<(), Fault> {
    if options.behavior_version != 1
        || options.max_degree > 16
        || options.max_subdivisions > 65536
        || !options.absolute_error_limit.is_finite()
        || options.absolute_error_limit < 0.
    {
        return Err(Fault::Policy(options));
    }
    Ok(())
}
pub(super) fn admit_curve<const DIM: usize>(
    curve: MetricCurve<'_, DIM>,
    options: PolynomialSideOptions,
) -> Result<(), Fault> {
    let degree = stored_degree(curve)?;
    if degree > usize::from(options.max_degree) {
        return Err(Fault::Degree {
            actual: degree,
            limit: options.max_degree,
        });
    }
    Ok(())
}
fn stored_degree<const DIM: usize>(curve: MetricCurve<'_, DIM>) -> Result<usize, Fault> {
    let degree = match curve {
        MetricCurve::Line { .. } => 1,
        MetricCurve::Bezier(bezier) => {
            if let Some(weights) = bezier.weights {
                return Err(Fault::RationalWeights {
                    count: weights.len(),
                });
            }
            bezier.poles.len().saturating_sub(1)
        }
    };
    Ok(degree)
}

pub(super) fn cross_controls<const DIM: usize>(
    curve: MetricCurve<'_, DIM>,
    embedding: MetricEmbedding<DIM>,
    placement: MetricEmbedding<3>,
    extrusion: [f64; 3],
    cancelled: &mut impl FnMut() -> bool,
) -> Result<Vec<Vector>, Fault> {
    let embedded = coefficients::embedded(curve, embedding, "side integral curve", cancelled)?;
    let placed = coefficients::place(embedded, placement)?;
    let vector = exact_extrusion(extrusion)?;
    let powers = derivative_cross(&placed, &vector, cancelled)?;
    power_to_bernstein(&powers, cancelled)
}
fn exact_extrusion(extrusion: [f64; 3]) -> Result<Vector, Fault> {
    for (index, value) in extrusion.iter().enumerate() {
        coefficients::scalar(*value, "side integral extrusion", index)?;
    }
    Ok(extrusion.map(ExactRatio::from_finite))
}
fn derivative_cross(
    curve: &coefficients::HomogeneousCurve,
    vector: &Vector,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<Vec<Vector>, Fault> {
    let derivatives = raw_derivatives(curve);
    let count = derivatives.iter().map(Vec::len).max().unwrap().max(1);
    let mut result = Vec::with_capacity(count);
    for power in 0..count {
        checkpoint(cancelled)?;
        let derivative = derivative_coefficient(&derivatives, &curve[3][0], power);
        result.push(cross(&derivative, vector));
    }
    Ok(result)
}
fn raw_derivatives(curve: &coefficients::HomogeneousCurve) -> [Vec<BigInt>; 3] {
    // The raw constructor and derivative preserve content; normalization would change area.
    std::array::from_fn(|axis| IntegerPolynomial(curve[axis].clone()).derivative().0)
}
fn derivative_coefficient(
    derivatives: &[Vec<BigInt>; 3],
    denominator: &BigInt,
    power: usize,
) -> Vector {
    std::array::from_fn(|axis| {
        let numerator = derivatives[axis].get(power).cloned().unwrap_or_default();
        ExactRatio::new(numerator, denominator.clone())
    })
}

fn cross(left: &Vector, right: &Vector) -> Vector {
    std::array::from_fn(|axis| {
        let next = (axis + 1) % 3;
        let last = (axis + 2) % 3;
        left[next]
            .multiply(&right[last])
            .subtract(&left[last].multiply(&right[next]))
    })
}
fn power_to_bernstein(
    powers: &[Vector],
    cancelled: &mut impl FnMut() -> bool,
) -> Result<Vec<Vector>, Fault> {
    let degree = powers.len() - 1;
    let mut controls = Vec::with_capacity(powers.len());
    for index in 0..=degree {
        checkpoint(cancelled)?;
        let mut control = zero_vector();
        for (power, coefficient) in powers.iter().enumerate().take(index + 1) {
            let factor = ExactRatio::new(binomial(index, power), binomial(degree, power));
            control = add(&control, &scale(coefficient, &factor));
        }
        controls.push(control);
    }
    Ok(controls)
}
fn binomial(degree: usize, power: usize) -> BigInt {
    let mut result = BigInt::from(1);
    for index in 0..power {
        result = result * (degree - index) / (index + 1);
    }
    result
}
pub(super) fn zero_vector() -> Vector {
    let zero = ExactRatio::from_finite(0.);
    std::array::from_fn(|_| zero.clone())
}
pub(super) fn scale(vector: &Vector, factor: &ExactRatio) -> Vector {
    // Exact scaling must precede norm rounding, including extreme exponents.
    std::array::from_fn(|axis| vector[axis].multiply(factor))
}
pub(super) fn add(left: &Vector, right: &Vector) -> Vector {
    // Coefficient addition keeps cancellation exact across all three axes.
    std::array::from_fn(|axis| left[axis].add(&right[axis]))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn power_conversion_and_cross_product_preserve_coefficient_magnitude() {
        let powers = vec![[0., 0., 0.], [2., 0., 0.], [-2., 0., 0.]];
        let exact: Vec<Vector> = powers
            .into_iter()
            .map(|v| v.map(ExactRatio::from_finite))
            .collect();
        let controls = power_to_bernstein(&exact, &mut || false).unwrap();
        assert_eq!(controls[1][0], ExactRatio::from_finite(1.));
        assert_eq!(controls[2], zero_vector());
        let z = [0., 0., 3.].map(ExactRatio::from_finite);
        assert_eq!(
            cross(&controls[1], &z),
            [0., -3., 0.].map(ExactRatio::from_finite)
        );
        assert_eq!(binomial(16, 8), BigInt::from(12870));
    }
}
