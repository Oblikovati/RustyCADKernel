// SPDX-License-Identifier: GPL-2.0-only
//! Owned sketch construction policy; extension ADR 0011, not a mapped OCCT API.
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Point2;
mod definition;
mod knots;
mod measure;
mod solve;
mod stations;
mod validate;
pub use definition::SketchSplineDefinition;

/// The application's Smooth, Chord, and Sweet station policies respectively.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SketchFitSpacing {
    /// Square root of chord distance (Smooth).
    Centripetal,
    /// Chord distance (Chord).
    ChordLength,
    /// Equal station intervals (Sweet).
    Uniform,
}
/// Authored control polygon and interpolated fit points have different meanings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SketchSplineIntent {
    /// Input points are control poles.
    Control,
    /// Input points are interpolation stations.
    Fit(SketchFitSpacing),
}
/// Explicit version-one numerical limits; derivatives use normalized parameter `[0,1]`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SketchSplineOptionsV1 {
    /// Dimensionless minimum pivot passed unchanged to the native solver.
    pub minimum_pivot: f64,
    /// Maximum distance from each analytic fit station to its source point.
    pub maximum_fit_residual: f64,
    /// Position, first derivative, and second derivative seam distance limits.
    pub maximum_seam_residuals: [f64; 3],
}
impl Default for SketchSplineOptionsV1 {
    /// Use machine-resolution pivots and native confusion in working units.
    /// Example: `let options = SketchSplineOptionsV1::default();`.
    fn default() -> Self {
        let spatial = rustycad_tkernel::precision::confusion();
        Self {
            minimum_pivot: f64::EPSILON,
            maximum_fit_residual: spatial,
            maximum_seam_residuals: [spatial; 3],
        }
    }
}
/// An accepted immutable recipe with measured checks; no mutable curve is retained.
#[derive(Clone, Debug, PartialEq)]
pub struct SketchSplineReport {
    /// Accepted independent curve recipe.
    pub definition: SketchSplineDefinition,
    /// Fit stations correspond to source indices; closed fits append the return to index zero.
    /// Control intent has no interpolated station list.
    pub stations: Vec<f64>,
    /// Measured maximum fit distance; absent for control intent.
    pub fit_residual: Option<f64>,
    /// Measured closure errors for derivative orders zero through two.
    pub seam_residuals: Option<[f64; 3]>,
    /// Requested limits, without widening or other automatic adjustment.
    pub options: SketchSplineOptionsV1,
}
/// Build a control or global fit curve without active derivative handles.
/// Example: `let result = build_unhandled_spline(&points, SketchSplineIntent::Control, false, options)?;`.
pub fn build_unhandled_spline(
    points: &[Point2],
    intent: SketchSplineIntent,
    closed: bool,
    options: SketchSplineOptionsV1,
) -> Result<SketchSplineReport, Failure> {
    validate::request(points, closed, options)?;
    let stations = stations::for_intent(points, intent, closed)?;
    let system = solve::Collocation::new(points, closed, &stations, intent);
    let definition = system.construct(options.minimum_pivot)?;
    measure::report(definition, stations, points, closed, options)
}
pub(super) fn decline(message: String) -> Failure {
    Failure::with_type("SketchSpline_ConstructionError", &message, "")
}
