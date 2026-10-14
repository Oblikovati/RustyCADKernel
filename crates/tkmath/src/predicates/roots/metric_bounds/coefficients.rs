// SPDX-License-Identifier: GPL-2.0-only
//! All four homogeneous coordinates use one shared binary64 scale, preserved exactly.
use super::super::{bernstein, error::poll};
use super::{MetricBezier, MetricCurve, MetricEmbedding, ResidualError};
use num_bigint::{BigInt, Sign};
pub(super) type HomogeneousCurve = [Vec<BigInt>; 4];
pub(super) fn scalar(
    value: f64,
    role: &'static str,
    index: usize,
) -> Result<BigInt, ResidualError> {
    super::super::super::exact_coordinate(value, index, 0).map_err(|_| ResidualError::Value {
        role,
        index,
        value,
    })
}
fn embedded_bezier<const DIM: usize>(
    curve: MetricBezier<'_, DIM>,
    embedding: MetricEmbedding<DIM>,
    role: &'static str,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<HomogeneousCurve, ResidualError> {
    embedded_bezier_using(curve, embedding, role, cancelled, weight)
}
fn embedded_bezier_using<const DIM: usize>(
    curve: MetricBezier<'_, DIM>,
    embedding: MetricEmbedding<DIM>,
    role: &'static str,
    cancelled: &mut impl FnMut() -> bool,
    weight_coefficient: fn(MetricBezier<'_, DIM>, usize) -> Result<BigInt, ResidualError>,
) -> Result<HomogeneousCurve, ResidualError> {
    validate_curve(curve, role)?;
    let axes = exact_embedding(embedding)?;
    let mut controls: HomogeneousCurve = std::array::from_fn(|_| Vec::new());
    for (index, pole) in curve.poles.iter().enumerate() {
        poll(cancelled)?;
        append_pole(
            &mut controls,
            pole,
            weight_coefficient(curve, index)?,
            &axes,
        )?;
    }
    let mut coefficients: HomogeneousCurve = std::array::from_fn(|_| Vec::new());
    for (target, source) in coefficients.iter_mut().zip(controls) {
        *target = bernstein::power_coefficients(source, cancelled)?;
    }
    remove_shared_power_of_two(&mut coefficients);
    Ok(coefficients)
}
pub(super) fn validate_curve<const DIM: usize>(
    curve: MetricBezier<'_, DIM>,
    role: &'static str,
) -> Result<(), ResidualError> {
    if curve.poles.is_empty() || curve.weights.is_some_and(|w| w.len() != curve.poles.len()) {
        return Err(ResidualError::Shape {
            representation: role,
            poles: curve.poles.len(),
            weights: curve.weights.map(<[f64]>::len),
        });
    }
    Ok(())
}
fn weight<const DIM: usize>(
    curve: MetricBezier<'_, DIM>,
    index: usize,
) -> Result<BigInt, ResidualError> {
    let value = curve.weights.map_or(1., |weights| weights[index]);
    if !value.is_finite() || value <= 0. {
        return Err(ResidualError::Value {
            role: "weight",
            index,
            value,
        });
    }
    scalar(value, "weight", index)
}
fn exact_embedding<const DIM: usize>(
    embedding: MetricEmbedding<DIM>,
) -> Result<[Vec<BigInt>; 3], ResidualError> {
    let mut result: [Vec<BigInt>; 3] = std::array::from_fn(|_| Vec::new());
    for (row, axis) in result.iter_mut().enumerate() {
        for (column, value) in embedding.linear[row].iter().enumerate() {
            axis.push(
                scalar(*value, "placement", row * DIM + column)?
                    * scalar(embedding.scale, "scale", 0)?,
            );
        }
        axis.push(scalar(embedding.origin[row], "origin", row)? << 1074usize);
    }
    Ok(result)
}
fn append_pole<const DIM: usize>(
    controls: &mut HomogeneousCurve,
    pole: &[f64; DIM],
    weight: BigInt,
    embedding: &[Vec<BigInt>; 3],
) -> Result<(), ResidualError> {
    let coordinates = exact_pole(pole)?;
    let scale = BigInt::from(1) << 1074usize;
    for row in 0..3 {
        let projected = coordinates
            .iter()
            .zip(&embedding[row])
            .fold(&embedding[row][DIM] * &scale, |sum, (point, axis)| {
                sum + point * axis
            });
        controls[row].push(projected * &weight);
    }
    controls[3].push(weight * &scale * &scale * &scale);
    Ok(())
}
fn remove_shared_power_of_two(curve: &mut HomogeneousCurve) {
    let shift = curve
        .iter()
        .flatten()
        .filter(|n| n.sign() != Sign::NoSign)
        .filter_map(BigInt::trailing_zeros)
        .min()
        .unwrap_or(0);
    for coefficient in curve.iter_mut().flatten() {
        *coefficient >>= shift;
    }
}

pub(super) fn embedded<const DIM: usize>(
    curve: MetricCurve<'_, DIM>,
    embedding: MetricEmbedding<DIM>,
    role: &'static str,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<HomogeneousCurve, ResidualError> {
    match curve {
        MetricCurve::Bezier(curve) => embedded_bezier(curve, embedding, role, cancelled),
        MetricCurve::Line { origin, direction } => {
            embedded_line(origin, direction, embedding, cancelled)
        }
    }
}
fn embedded_line<const DIM: usize>(
    origin: [f64; DIM],
    direction: [f64; DIM],
    mut embedding: MetricEmbedding<DIM>,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<HomogeneousCurve, ResidualError> {
    poll(cancelled)?;
    let weight = BigInt::from(1) << 1074usize;
    let mut result: HomogeneousCurve = std::array::from_fn(|_| Vec::new());
    let origin_axes = exact_embedding(embedding)?;
    append_pole(&mut result, &origin, weight.clone(), &origin_axes)?;
    embedding.origin = [0.; 3];
    let direction_axes = exact_embedding(embedding)?;
    append_pole(&mut result, &direction, weight, &direction_axes)?;
    result[3].truncate(1);
    Ok(normalized(result))
}
pub(super) fn place(
    curve: HomogeneousCurve,
    embedding: MetricEmbedding<3>,
) -> Result<HomogeneousCurve, ResidualError> {
    let axes = exact_embedding(embedding)?;
    let scale = BigInt::from(1) << 1074usize;
    let mut result: HomogeneousCurve =
        std::array::from_fn(|_| vec![BigInt::from(0); curve[3].len().max(curve[0].len())]);
    for row in 0..3 {
        place_row(&curve, &axes[row], &mut result[row]);
    }
    result[3] = curve[3].iter().map(|n| n * &scale * &scale).collect();
    Ok(normalized(result))
}

fn exact_pole<const DIM: usize>(pole: &[f64; DIM]) -> Result<Vec<BigInt>, ResidualError> {
    pole.iter()
        .enumerate()
        .map(|(index, value)| scalar(*value, "pole", index))
        .collect::<Result<_, _>>()
}
fn normalized(mut curve: HomogeneousCurve) -> HomogeneousCurve {
    remove_shared_power_of_two(&mut curve);
    curve
}
fn place_row(curve: &HomogeneousCurve, axis: &[BigInt], target: &mut [BigInt]) {
    for (index, coefficient) in curve[3].iter().enumerate() {
        target[index] += &axis[3] * coefficient;
    }
    for coordinate in 0..3 {
        for (index, coefficient) in curve[coordinate].iter().enumerate() {
            target[index] += &axis[coordinate] * coefficient;
        }
    }
}

/// Signed admission is confined to callers that certify denominator root freedom.
pub(super) fn embedded_signed_bezier<const DIM: usize>(
    curve: MetricBezier<'_, DIM>,
    embedding: MetricEmbedding<DIM>,
    role: &'static str,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<HomogeneousCurve, ResidualError> {
    embedded_bezier_using(curve, embedding, role, cancelled, signed_weight)
}
fn signed_weight<const DIM: usize>(
    curve: MetricBezier<'_, DIM>,
    index: usize,
) -> Result<BigInt, ResidualError> {
    scalar(
        curve.weights.map_or(1., |weights| weights[index]),
        "weight",
        index,
    )
}
