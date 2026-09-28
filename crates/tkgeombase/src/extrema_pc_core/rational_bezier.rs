// Copyright (c) 2025 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of fillBinomialRow and rationalBezierSquareDistance of
// ModelingData/TKGeomBase/ExtremaPC2d/ExtremaPC2d_GridEvaluator.hxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The certification of a constant distance from a rational 2D Bezier curve: the
//! square distance is `|Q(t)|^2 / w(t)^2` with the homogeneous polynomials `Q` and `w`,
//! so it is constant exactly when the Bernstein coefficients of `|Q|^2` are a multiple
//! of those of `w^2`. The coefficients are computed in `long double`, as natively; the
//! port uses the x87 80-bit emulation of the pinned profile ([`LongDouble`]).
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dBezierCurve;
use rustycad_tkmath::gp::Point2;
use rustycad_tkmath::math_poly::LongDouble;

/// `fillBinomialRow`: the binomial coefficients of `degree`, built as
/// `C[i] = C[i - 1] * (degree - i + 1) / i`.
fn binomial_row(degree: usize) -> Vec<LongDouble> {
    let mut row: Vec<LongDouble> = vec![LongDouble::new(1.0)];
    for index in 1..=degree {
        let next: LongDouble = row[index - 1] * LongDouble::from_usize(degree - index + 1)
            / LongDouble::from_usize(index);
        row.push(next);
    }
    row
}

/// The weighted pole offsets `w * (P - Q)` and the weights, in `long double`.
struct HomogeneousPoles {
    qx: Vec<LongDouble>,
    qy: Vec<LongDouble>,
    weights: Vec<LongDouble>,
}

fn homogeneous_poles(
    curve: &Geom2dBezierCurve,
    point: Point2,
) -> Result<HomogeneousPoles, Failure> {
    let degree: usize = curve.degree() as usize;
    let mut poles = HomogeneousPoles {
        qx: Vec::with_capacity(degree + 1),
        qy: Vec::with_capacity(degree + 1),
        weights: Vec::with_capacity(degree + 1),
    };
    for index in 0..=degree {
        let pole_index: i32 = index as i32 + 1;
        let weight: f64 = curve.weight(pole_index)?;
        let pole: Point2 = curve.pole(pole_index)?;
        poles.weights.push(LongDouble::new(weight));
        // The products are formed in double before the widening, as natively.
        poles
            .qx
            .push(LongDouble::new(weight * (pole.x() - point.x())));
        poles
            .qy
            .push(LongDouble::new(weight * (pole.y() - point.y())));
    }
    Ok(poles)
}

/// The Bernstein coefficients of degree `2 * degree` of `|Q|^2` and of `w^2`.
fn product_coefficients(
    poles: &HomogeneousPoles,
    degree: usize,
) -> (Vec<LongDouble>, Vec<LongDouble>) {
    let double_degree: usize = 2 * degree;
    let degree_binomial: Vec<LongDouble> = binomial_row(degree);
    let double_binomial: Vec<LongDouble> = binomial_row(double_degree);
    let mut distance_coefficients: Vec<LongDouble> = Vec::with_capacity(double_degree + 1);
    let mut weight_coefficients: Vec<LongDouble> = Vec::with_capacity(double_degree + 1);
    for (result_index, double_coefficient) in double_binomial.iter().enumerate() {
        let (mut distance, mut weight) = (LongDouble::ZERO, LongDouble::ZERO);
        let first: usize = result_index.saturating_sub(degree);
        let last: usize = degree.min(result_index);
        for index in first..=last {
            let second: usize = result_index - index;
            let factor: LongDouble =
                degree_binomial[index] * degree_binomial[second] / *double_coefficient;
            distance = distance
                + factor
                    * (poles.qx[index] * poles.qx[second] + poles.qy[index] * poles.qy[second]);
            weight = weight + factor * poles.weights[index] * poles.weights[second];
        }
        distance_coefficients.push(distance);
        weight_coefficients.push(weight);
    }
    (distance_coefficients, weight_coefficients)
}

/// `rationalBezierSquareDistance`: the constant square distance from `point` to a
/// rational Bezier curve, `None` for a null or non-rational curve or a residual above
/// `SquareConfusion * max(1, |coefficient|)`.
pub(super) fn rational_bezier_square_distance(
    curve: &Handle<Geom2dBezierCurve>,
    point: Point2,
) -> Result<Option<f64>, Failure> {
    let Some(curve) = curve.get().filter(|curve| curve.is_rational()) else {
        return Ok(None);
    };
    let degree: usize = curve.degree() as usize;
    let poles: HomogeneousPoles = homogeneous_poles(curve, point)?;
    let (distances, weights) = product_coefficients(&poles, degree);
    let reference_weight: LongDouble = weights[0];
    if reference_weight == LongDouble::ZERO {
        return Ok(None);
    }
    let square_distance: LongDouble = distances[0] / reference_weight;
    let tolerance: LongDouble = LongDouble::new(precision::square_confusion());
    for (distance, weight) in distances.iter().zip(&weights) {
        let residual: LongDouble = *distance - square_distance * *weight;
        let scale: LongDouble = LongDouble::new(1.0).max(distance.abs());
        if residual.abs() > tolerance * scale {
            return Ok(None);
        }
    }
    Ok(Some(square_distance.narrow()))
}
