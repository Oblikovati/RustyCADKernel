// SPDX-License-Identifier: GPL-2.0-only
use super::{
    SketchFitSpacing, SketchSplineDefinition, SketchSplineIntent, SketchSplineOptionsV1,
    SketchSplineReport, build_unhandled_spline, decline,
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point2, Vector2};
mod check;
mod hermite;
mod tangents;
mod validate;
/// An authored endpoint anchored at a source fit point; input order has no meaning.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SketchSplineHandle {
    /// Index in the source fit-point array.
    pub fit_index: usize,
    /// Absolute endpoint in the same coordinates as the fit points.
    pub end: Point2,
}
/// Explicit acceptance policy for natural fitting and the resulting Hermite curve.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SketchHandleOptionsV1 {
    /// Applied only to the natural global fit that supplies derivative magnitudes.
    pub natural: SketchSplineOptionsV1,
    /// Maximum position error on both sides of every Hermite station.
    pub maximum_position_residual: f64,
    /// Maximum derivative error on both sides, with respect to normalized parameter.
    pub maximum_derivative_residual: f64,
}
impl Default for SketchHandleOptionsV1 {
    /// Use native working-unit confusion; example: `SketchHandleOptionsV1::default()`.
    fn default() -> Self {
        Self {
            natural: SketchSplineOptionsV1::default(),
            maximum_position_residual: rustycad_tkernel::precision::confusion(),
            maximum_derivative_residual: rustycad_tkernel::precision::confusion(),
        }
    }
}
/// Owned handled curve with separate natural-fit and final-curve checks (ADR 0012).
#[derive(Clone, Debug, PartialEq)]
pub struct SketchHandledSplineReport {
    /// Global fit before handle overrides, including stations and its own checked limits.
    pub natural: SketchSplineReport,
    /// Cubic Hermite recipe; triple interior knots imply formal native C0 continuity.
    pub definition: SketchSplineDefinition,
    /// Requested global derivatives in source-point order.
    pub derivatives: Vec<Vector2>,
    /// Source indices of coincident handles whose natural derivatives were retained.
    pub ignored_coincident_handles: Vec<usize>,
    /// Maximum measured position and derivative errors against station requests.
    pub station_residuals: [f64; 2],
    /// Closed seam position and derivative errors; no C2 guarantee for handles.
    pub seam_residuals: Option<[f64; 2]>,
    /// Requested limits, without widening.
    pub options: SketchHandleOptionsV1,
}
/// Construct all Hermite spans for a fit curve with at least one active handle.
/// Use `build_unhandled_spline` for no handles. Example: `build_handled_spline(&points, spacing, false, &handles, options)?`.
pub fn build_handled_spline(
    points: &[Point2],
    spacing: SketchFitSpacing,
    closed: bool,
    handles: &[SketchSplineHandle],
    options: SketchHandleOptionsV1,
) -> Result<SketchHandledSplineReport, Failure> {
    validate::request(points, handles, options)?;
    let natural = build_unhandled_spline(
        points,
        SketchSplineIntent::Fit(spacing),
        closed,
        options.natural,
    )?;
    finish_handled(points, closed, handles, natural, options)
}
fn finish_handled(
    points: &[Point2],
    closed: bool,
    handles: &[SketchSplineHandle],
    natural: SketchSplineReport,
    options: SketchHandleOptionsV1,
) -> Result<SketchHandledSplineReport, Failure> {
    let (derivatives, ignored) = tangents::effective(points, closed, handles, &natural)?;
    let definition = hermite::construct(points, &natural.stations, &derivatives)?;
    let (station_residuals, seam_residuals) = check::measure(
        &definition,
        points,
        &natural.stations,
        &derivatives,
        closed,
        options,
    )?;
    Ok(SketchHandledSplineReport {
        natural,
        definition,
        derivatives,
        ignored_coincident_handles: ignored,
        station_residuals,
        seam_residuals,
        options,
    })
}
