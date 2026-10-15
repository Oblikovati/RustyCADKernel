// SPDX-License-Identifier: GPL-2.0-only
//! Explicit tolerance input and achieved tolerance output (ADR 0068). Spatial, angular and
//! display tolerances are distinct; the cap on the achieved tolerance has no implicit
//! default. Measures use the shared `crate::measured::Measured` (ADR 0081).

/// Limit on the achieved B-rep tolerance of a result.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ToleranceCap {
    /// Accept any achieved tolerance; must be chosen explicitly.
    NoLimit,
    /// Accept, but record a finding when the achieved tolerance exceeds the limit.
    Report(f64),
    /// Decline a result whose achieved tolerance exceeds the limit.
    Refuse(f64),
}

/// Display tessellation limits; carried to the consumer, never used for modelling decisions.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DisplayTolerance {
    /// Maximum chordal deviation in model units.
    pub chordal: f64,
    /// Maximum angular deviation between adjacent facets, in radians.
    pub angular: f64,
}

/// Caller resolution of one operation.
///
/// - `weld`: distinct authored vertices closer than this, and curves shorter than this,
///   are declined instead of being merged or collapsed.
/// - `sew`: largest admitted distance between a curve's exact end and its authored vertex
///   (an arc's two radii must agree within it).
/// - `angular`: smallest admitted angle, in radians (arc sweep, plane frame orthogonality,
///   extrusion direction against the plane).
/// - `display`: tessellation limits for presentation only.
/// - `cap`: the explicit limit on the achieved B-rep tolerance.
///
/// ```
/// use rustycad_tkbo::profile_extrusion::{DisplayTolerance, Resolution, ToleranceCap};
/// let display = DisplayTolerance { chordal: 1e-3, angular: 0.1 };
/// let resolution = Resolution::new(1e-6, 1e-7, 1e-9, display, ToleranceCap::Refuse(1e-6));
/// assert!(resolution.admit().is_ok());
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Resolution {
    /// Spatial weld tolerance (model units).
    pub weld: f64,
    /// Spatial sew tolerance (model units).
    pub sew: f64,
    /// Angular tolerance (radians).
    pub angular: f64,
    /// Display tessellation limits.
    pub display: DisplayTolerance,
    /// Limit on the achieved B-rep tolerance.
    pub cap: ToleranceCap,
}

impl Resolution {
    /// Name every tolerance explicitly; `Resolution::new(weld, sew, angular, display, cap)`.
    pub fn new(
        weld: f64,
        sew: f64,
        angular: f64,
        display: DisplayTolerance,
        cap: ToleranceCap,
    ) -> Self {
        Self {
            weld,
            sew,
            angular,
            display,
            cap,
        }
    }

    /// Check every field is finite and positive; returns the first offending field and value.
    /// Example: `resolution.admit()?`.
    pub fn admit(&self) -> Result<(), (&'static str, f64)> {
        let fields = [
            ("weld", self.weld),
            ("sew", self.sew),
            ("angular", self.angular),
            ("display.chordal", self.display.chordal),
            ("display.angular", self.display.angular),
        ];
        let mut all = fields.to_vec();
        if let ToleranceCap::Report(limit) | ToleranceCap::Refuse(limit) = self.cap {
            all.push(("cap", limit));
        }
        match all
            .into_iter()
            .find(|(_, value)| !(value.is_finite() && *value > 0.0))
        {
            Some(offending) => Err(offending),
            None => Ok(()),
        }
    }
}

/// Largest stored B-rep tolerance per entity kind of a published result.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AchievedTolerance {
    /// Largest vertex tolerance.
    pub vertex: f64,
    /// Largest edge tolerance.
    pub edge: f64,
    /// Largest face tolerance.
    pub face: f64,
}

impl AchievedTolerance {
    /// The largest of the three; `achieved.max()`.
    pub fn max(&self) -> f64 {
        self.vertex.max(self.edge).max(self.face)
    }

    /// Whether `cap` admits this tolerance; `achieved.within(cap)`.
    pub fn within(&self, cap: ToleranceCap) -> bool {
        match cap {
            ToleranceCap::NoLimit => true,
            ToleranceCap::Report(limit) | ToleranceCap::Refuse(limit) => self.max() <= limit,
        }
    }
}
