// SPDX-License-Identifier: GPL-2.0-only
use rustycad_tkernel::{ncollection::Array1, standard::Failure};
use rustycad_tkg2d::geom2d::Geom2dBSplineCurve;
use rustycad_tkmath::gp::Point2;
/// Copied poles and full knots, including wrapped closure; each native curve is independent.
#[derive(Clone, Debug, PartialEq)]
pub struct SketchSplineDefinition {
    degree: i32,
    poles: Vec<Point2>,
    flat_knots: Vec<f64>,
}
impl SketchSplineDefinition {
    pub(super) fn new(degree: i32, poles: Vec<Point2>, flat_knots: Vec<f64>) -> Self {
        Self {
            degree,
            poles,
            flat_knots,
        }
    }
    /// Inspect degree; example: `assert_eq!(definition.degree(), 3)`.
    pub fn degree(&self) -> i32 {
        self.degree
    }
    /// Inspect immutable owned poles; example: `definition.poles().len()`.
    pub fn poles(&self) -> &[Point2] {
        &self.poles
    }
    /// Inspect knots with repetitions; example: `definition.flat_knots()[0]`.
    pub fn flat_knots(&self) -> &[f64] {
        &self.flat_knots
    }
    /// Build a fresh native curve; editing it cannot mutate this recipe or another curve.
    /// Example: `let independent = definition.curve()?;`.
    pub fn curve(&self) -> Result<Geom2dBSplineCurve, Failure> {
        let (knots, multiplicities) = compress_knots(&self.flat_knots);
        Geom2dBSplineCurve::new(
            &Array1::from_shared(&self.poles, 1),
            &Array1::from_shared(&knots, 1),
            &Array1::from_shared(&multiplicities, 1),
            self.degree,
            false,
        )
    }
}
fn compress_knots(flat: &[f64]) -> (Vec<f64>, Vec<i32>) {
    let mut knots: Vec<f64> = Vec::new();
    let mut multiplicities: Vec<i32> = Vec::new();
    for &knot in flat {
        if knots.last() == Some(&knot) {
            if let Some(last) = multiplicities.last_mut() {
                *last += 1;
            }
        } else {
            knots.push(knot);
            multiplicities.push(1);
        }
    }
    (knots, multiplicities)
}
