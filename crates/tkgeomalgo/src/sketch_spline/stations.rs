// SPDX-License-Identifier: GPL-2.0-only
use super::{SketchFitSpacing, SketchSplineIntent, decline};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Point2;
pub(super) fn for_intent(
    points: &[Point2],
    intent: SketchSplineIntent,
    closed: bool,
) -> Result<Vec<f64>, Failure> {
    let SketchSplineIntent::Fit(spacing) = intent else {
        return Ok(Vec::new());
    };
    let count = points.len() + usize::from(closed);
    let mut cumulative: Vec<f64> = vec![0.0];
    for index in 1..count {
        let step = chord_step(points[index - 1], points[index % points.len()], spacing);
        let next = cumulative[index - 1] + step;
        if !next.is_finite() || next <= cumulative[index - 1] {
            return Err(decline(format!(
                "fit station {index}, cumulative {next}, step {step}: expected finite strictly increasing chord accumulation"
            )));
        }
        cumulative.push(next);
    }
    normalize(cumulative)
}
fn chord_step(first: Point2, second: Point2, spacing: SketchFitSpacing) -> f64 {
    if spacing == SketchFitSpacing::Uniform {
        return 1.0;
    }
    let dx = second.x() - first.x();
    let dy = second.y() - first.y();
    let chord = (dx * dx + dy * dy).sqrt();
    match spacing {
        SketchFitSpacing::Centripetal => chord.sqrt(),
        _ => chord,
    }
}
fn normalize(mut cumulative: Vec<f64>) -> Result<Vec<f64>, Failure> {
    let total = cumulative[cumulative.len() - 1];
    for station in &mut cumulative {
        *station /= total;
    }
    if cumulative.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(decline(format!(
            "normalized stations {cumulative:?}: expected strictly increasing representable parameters"
        )));
    }
    Ok(cumulative)
}
