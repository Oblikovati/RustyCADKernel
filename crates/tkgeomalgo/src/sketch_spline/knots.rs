// SPDX-License-Identifier: GPL-2.0-only
use super::SketchSplineIntent;
pub(super) fn for_intent(
    count: usize,
    degree: i32,
    intent: SketchSplineIntent,
    closed: bool,
    stations: &[f64],
) -> Vec<f64> {
    match (intent, closed) {
        (SketchSplineIntent::Control, false) => clamped_control(count, degree as usize),
        (SketchSplineIntent::Control, true) => (0..count + 7)
            .map(|i| (i as f64 - 3.0) / count as f64)
            .collect(),
        (SketchSplineIntent::Fit(_), false) => averaged(stations, degree as usize),
        (SketchSplineIntent::Fit(_), true) => periodic(stations),
    }
}
fn clamped_control(count: usize, degree: usize) -> Vec<f64> {
    let mut knots = vec![0.0; count + degree + 1];
    for (index, knot) in knots.iter_mut().enumerate().skip(degree + 1) {
        *knot = if index >= count {
            1.0
        } else {
            (index - degree) as f64 / (count - degree) as f64
        };
    }
    knots
}
fn averaged(stations: &[f64], degree: usize) -> Vec<f64> {
    let count = stations.len();
    let mut knots = vec![0.0; count + degree + 1];
    knots[count..].fill(1.0);
    for start in 1..count - degree {
        knots[start + degree] = stations[start..start + degree].iter().sum::<f64>() / degree as f64;
    }
    knots
}
fn periodic(stations: &[f64]) -> Vec<f64> {
    let count = stations.len() - 1;
    let mut knots = vec![0.0; count + 7];
    knots[3..count + 4].copy_from_slice(stations);
    for offset in 1..=3 {
        knots[3 - offset] = stations[0] - (1.0 - (stations[count - offset] - stations[0]));
        knots[3 + count + offset] = stations[count] + (stations[offset] - stations[0]);
    }
    knots
}
