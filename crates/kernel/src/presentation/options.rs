// SPDX-License-Identifier: GPL-2.0-only
//! Display approximation limits. They are not modelling tolerances: they only bound
//! how far the presentation mesh may deviate from the exact faces and edges.
use crate::query_error::{BodyQuery, BodyQueryError, BodyQueryFailure, QueryEntity};
use rustycad_tkernel::precision;
use std::f64::consts::PI;

/// Requested display deflections, in model length units and radians.
///
/// Both limits are absolute (never relative to the body size) so the same options
/// mean the same accuracy for every body.
///
/// ```
/// use rustycad_kernel::presentation::DisplayOptions;
/// let options = DisplayOptions::new(0.01, 0.5)?;
/// assert_eq!(options.linear_deflection(), 0.01);
/// assert!(DisplayOptions::new(0.0, 0.5).is_err());
/// # Ok::<(), rustycad_kernel::query_error::BodyQueryError>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DisplayOptions {
    linear_deflection: f64,
    angular_deflection: f64,
}

impl DisplayOptions {
    /// Checks the limits: `linear_deflection` finite and at least
    /// `Precision::Confusion()`, `angular_deflection` in `[Precision::Angular(), PI]`.
    /// See the type example.
    pub fn new(linear_deflection: f64, angular_deflection: f64) -> Result<Self, BodyQueryError> {
        let linear_ok =
            linear_deflection.is_finite() && linear_deflection >= precision::confusion();
        let angular_range = precision::angular()..=PI;
        if !linear_ok || !angular_range.contains(&angular_deflection) {
            return Err(invalid_options(linear_deflection, angular_deflection));
        }
        Ok(Self {
            linear_deflection,
            angular_deflection,
        })
    }

    /// The maximum chord distance between the mesh and the exact geometry;
    /// `options.linear_deflection()`.
    pub fn linear_deflection(&self) -> f64 {
        self.linear_deflection
    }

    /// The maximum angle between adjacent mesh segments or facet normals;
    /// `options.angular_deflection()`.
    pub fn angular_deflection(&self) -> f64 {
        self.angular_deflection
    }
}

fn invalid_options(linear: f64, angular: f64) -> BodyQueryError {
    let cause = format!(
        "display options linear={linear}, angular={angular}: expected a finite linear \
         deflection >= {} and an angular deflection in [{}, PI]",
        precision::confusion(),
        precision::angular()
    );
    BodyQueryError::new(
        BodyQuery::Tessellate,
        QueryEntity::Body,
        BodyQueryFailure::InvalidOptions,
        cause,
    )
}

/// The settings the mesher actually used after completing the request: the
/// interior deflections and the minimum element size it derived. These are
/// effective settings, not measured errors.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EffectiveMeshSettings {
    /// Linear deflection used on edges.
    pub linear_deflection: f64,
    /// Angular deflection used on edges.
    pub angular_deflection: f64,
    /// Linear deflection used inside faces.
    pub interior_linear_deflection: f64,
    /// Angular deflection used inside faces.
    pub interior_angular_deflection: f64,
    /// Minimum element size the mesher derived.
    pub min_size: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_out_of_domain_limits() {
        for (linear, angular) in [(f64::NAN, 0.5), (1e-9, 0.5), (0.1, 0.0), (0.1, 4.0)] {
            let error = DisplayOptions::new(linear, angular).unwrap_err();
            assert_eq!(error.class, BodyQueryFailure::InvalidOptions);
            assert!(error.cause.contains(&format!("linear={linear}")));
        }
    }

    #[test]
    fn keeps_valid_limits() {
        let options = DisplayOptions::new(0.25, PI).unwrap();
        assert_eq!(options.linear_deflection(), 0.25);
        assert_eq!(options.angular_deflection(), PI);
    }
}
