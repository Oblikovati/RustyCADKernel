// SPDX-License-Identifier: GPL-2.0-only
//! Owned property results: plain values, no live shape, `Send + Sync`.
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::geom_abs::SurfaceType;

/// How a value was integrated, from strongest to weakest evidence.
/// Combining results keeps the weakest method (`max` of the ordering).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IntegrationMethod {
    /// Exact antiderivatives; the bound covers floating-point rounding only.
    ClosedForm,
    /// Adaptive Gauss-Kronrod over the trims; the bound is the rule's error estimate
    /// plus rounding, not a proven enclosure.
    AdaptiveQuadrature,
    /// The ported `BRepGProp` fixed-order Gauss integration, used only for faces whose trims
    /// do not close in the parameter plane. It reports no error: the bound is infinite.
    /// (Its adaptive variant is not used: natively and in the port it reports a relative
    /// error of 7.4e-17 for an elliptic prism whose volume it misses by 4e-7.)
    NativeUnbounded,
}

/// A scalar and its absolute error bound under the stated [`IntegrationMethod`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Measured {
    /// The computed value.
    pub value: f64,
    /// Absolute bound on `|true value - value|` under the method's error model.
    pub error_bound: f64,
}

/// A point and the bound on its Euclidean distance to the true point.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeasuredPoint {
    /// World coordinates.
    pub point: [f64; 3],
    /// Bound on the Euclidean error of `point`.
    pub error_bound: f64,
}

/// Whether a loop bounds material on its left (outer) or cuts a hole (inner).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LoopRole {
    /// Positive signed parameter area: counter-clockwise in the forward face.
    Outer,
    /// Negative signed parameter area: clockwise, a hole.
    Inner,
}

/// One wire of a face, measured in the face's forward parameter plane.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LoopSummary {
    /// Number of edge uses, seams counted on each side, degenerate edges included.
    pub edge_uses: usize,
    /// Signed area enclosed in the `(u, v)` plane (Green's theorem over the p-curves).
    pub parameter_area: Measured,
    /// Outer or inner, from the sign of `parameter_area`.
    pub role: LoopRole,
}

/// Area, centroid and trim structure of one face.
#[derive(Clone, Debug, PartialEq)]
pub struct FaceProperties {
    /// The analytic kind of the underlying surface.
    pub surface: SurfaceType,
    /// The face's orientation in its parent (material side of the surface normal).
    pub orientation: Orientation,
    /// Surface area; non-negative for a well-trimmed face.
    pub area: Measured,
    /// Area centroid in world coordinates.
    pub centroid: MeasuredPoint,
    /// One entry per wire, in exploration order.
    pub loops: Vec<LoopSummary>,
    /// The weakest method used for `area` and `centroid`.
    pub method: IntegrationMethod,
}

/// Area, volume and centroids of a body.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BodyProperties {
    /// Total area of all faces, each face occurrence counted once.
    pub area: Measured,
    /// Enclosed volume by the divergence theorem; negative when the shells face inward,
    /// meaningless for open shells.
    pub volume: Measured,
    /// Volume centroid (centre of mass at unit density).
    pub centroid: MeasuredPoint,
    /// Area centroid of the boundary.
    pub surface_centroid: MeasuredPoint,
    /// Number of face occurrences integrated.
    pub face_count: usize,
    /// The weakest method used.
    pub method: IntegrationMethod,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn results_are_owned_thread_safe_values() {
        static_assertions::assert_impl_all!(BodyProperties: Send, Sync);
        static_assertions::assert_impl_all!(FaceProperties: Send, Sync);
        assert!(IntegrationMethod::ClosedForm < IntegrationMethod::NativeUnbounded);
    }
}
