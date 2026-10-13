// SPDX-License-Identifier: GPL-2.0-only
use super::{BezierInput, BezierLineError, BezierLineQuery, BezierStage};
use crate::predicates::{exact_coordinate, exact_determinant, exact_dot, exact_point};
use num_bigint::BigInt;
pub(super) struct Projection {
    pub numerator: Vec<BigInt>,
    pub denominator: Vec<BigInt>,
}
pub(super) fn project(
    query: BezierLineQuery<'_>,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<Projection, BezierLineError> {
    project_with(query, cancelled, &mut |_, _, _| {})
}
pub(super) fn project_directed(
    query: BezierLineQuery<'_>,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<(Projection, Vec<BigInt>), BezierLineError> {
    let mut longitudinal = Vec::with_capacity(query.controls.len());
    let projected = project_with(query, cancelled, &mut |direction, displacement, weight| {
        let dot = exact_dot(direction, displacement);
        longitudinal.push(dot * weight);
    })?;
    Ok((projected, longitudinal))
}
fn project_with(
    query: BezierLineQuery<'_>,
    cancelled: &mut impl FnMut() -> bool,
    capture: &mut impl FnMut(&[BigInt; 2], &[BigInt; 2], &BigInt),
) -> Result<Projection, BezierLineError> {
    validate_counts(query)?;
    let origin = point(query.line[0], 0, BezierInput::Line)?;
    let end = point(query.line[1], 1, BezierInput::Line)?;
    if origin == end {
        return Err(BezierLineError::DegenerateLine(query.line));
    }
    let direction = [&end[0] - &origin[0], &end[1] - &origin[1]];
    project_controls(query, origin, direction, cancelled, capture)
}
fn validate_counts(query: BezierLineQuery<'_>) -> Result<(), BezierLineError> {
    if query.controls.is_empty() {
        return Err(BezierLineError::EmptyControls);
    }
    if let Some(weights) = query.weights
        && weights.len() != query.controls.len()
    {
        return Err(BezierLineError::WeightCount {
            controls: query.controls.len(),
            weights: weights.len(),
        });
    }
    Ok(())
}
fn project_controls(
    query: BezierLineQuery<'_>,
    origin: [BigInt; 2],
    direction: [BigInt; 2],
    cancelled: &mut impl FnMut() -> bool,
    capture: &mut impl FnMut(&[BigInt; 2], &[BigInt; 2], &BigInt),
) -> Result<Projection, BezierLineError> {
    let mut result = Projection::new(query.controls.len());
    for (index, control) in query.controls.iter().enumerate() {
        poll_projection(cancelled)?;
        let position = point(*control, index, BezierInput::Control)?;
        let displacement = [&position[0] - &origin[0], &position[1] - &origin[1]];
        let weight = weight(query.weights, index)?;
        capture(&direction, &displacement, &weight);
        result
            .numerator
            .push(exact_determinant(&direction, &displacement) * &weight);
        result.denominator.push(weight);
    }
    Ok(result)
}
fn point(
    value: [f64; 2],
    index: usize,
    input: BezierInput,
) -> Result<[BigInt; 2], BezierLineError> {
    exact_point(value, index).map_err(|cause| BezierLineError::Coordinate { input, cause })
}
fn weight(weights: Option<&[f64]>, index: usize) -> Result<BigInt, BezierLineError> {
    let Some(weights) = weights else {
        return Ok(BigInt::from(1));
    };
    let value = weights[index];
    exact_coordinate(value, index, 0).map_err(|_| BezierLineError::Weight { index, value })
}

fn poll_projection(cancelled: &mut impl FnMut() -> bool) -> Result<(), BezierLineError> {
    super::super::error::poll(cancelled).map_err(|cause| BezierLineError::Root {
        stage: BezierStage::Input,
        cause,
    })
}

impl Projection {
    fn new(count: usize) -> Self {
        Self {
            numerator: Vec::with_capacity(count),
            denominator: Vec::with_capacity(count),
        }
    }
}
