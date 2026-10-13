// SPDX-License-Identifier: GPL-2.0-only
use super::{SketchSplineDefinition, SketchSplineOptionsV1, SketchSplineReport, decline};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve};
use rustycad_tkmath::gp::Point2;
pub(super) fn report(
    definition: SketchSplineDefinition,
    stations: Vec<f64>,
    points: &[Point2],
    closed: bool,
    options: SketchSplineOptionsV1,
) -> Result<SketchSplineReport, Failure> {
    super::validate::coordinates(definition.poles())?;
    let curve = definition.curve()?;
    let fit_residual = fit_residual(&curve, &stations, points, options.maximum_fit_residual)?;
    let seam_residuals = if closed {
        Some(seam_residuals(&curve, options.maximum_seam_residuals)?)
    } else {
        None
    };
    Ok(SketchSplineReport {
        definition,
        stations,
        fit_residual,
        seam_residuals,
        options,
    })
}
fn fit_residual(
    curve: &Geom2dBSplineCurve,
    stations: &[f64],
    points: &[Point2],
    limit: f64,
) -> Result<Option<f64>, Failure> {
    if stations.is_empty() {
        return Ok(None);
    }
    let mut maximum: f64 = 0.0;
    for (index, &station) in stations.iter().enumerate() {
        let actual = curve.value(station)?;
        let expected = points[index % points.len()];
        let residual = (actual.x() - expected.x()).hypot(actual.y() - expected.y());
        require_residual(residual, limit, &format!("fit station {index} = {station}"))?;
        maximum = maximum.max(residual);
    }
    Ok(Some(maximum))
}
fn seam_residuals(curve: &Geom2dBSplineCurve, limits: [f64; 3]) -> Result<[f64; 3], Failure> {
    let first = curve.eval_d2(0.0)?;
    let last = curve.eval_d2(1.0)?;
    let residuals = [
        (first.point.x() - last.point.x()).hypot(first.point.y() - last.point.y()),
        (first.d1.x() - last.d1.x()).hypot(first.d1.y() - last.d1.y()),
        (first.d2.x() - last.d2.x()).hypot(first.d2.y() - last.d2.y()),
    ];
    for (order, (&residual, &limit)) in residuals.iter().zip(&limits).enumerate() {
        require_residual(residual, limit, &format!("closed derivative order {order}"))?;
    }
    Ok(residuals)
}
pub(super) fn require_residual(residual: f64, limit: f64, context: &str) -> Result<(), Failure> {
    if !residual.is_finite() || residual > limit {
        return Err(decline(format!(
            "{context}, residual {residual}: expected finite residual <= {limit}"
        )));
    }
    Ok(())
}
