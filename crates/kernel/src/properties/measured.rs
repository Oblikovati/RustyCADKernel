// SPDX-License-Identifier: GPL-2.0-only
//! Owned property results: plain values, no live shape, `Send + Sync`. Scalars and points
//! are the shared `rustycad_tkbo::measured` values (ADR 0081), as in `modeling`.
use crate::topology::{EdgeIndex, FaceIndex};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::geom_abs::SurfaceType;

pub use rustycad_tkbo::measured::{Evidence, IntegrationMethod, Measured, MeasuredPoint};

/// Whether a loop bounds material on its left (outer) or cuts a hole (inner).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LoopRole {
    /// Positive signed parameter area: counter-clockwise in the forward face.
    Outer,
    /// Negative signed parameter area: clockwise, a hole.
    Inner,
}

/// One wire of a face, measured in the face's forward parameter plane.
#[derive(Clone, Debug, PartialEq)]
pub struct LoopSummary {
    /// The bounding edge uses in wire exploration order, by stable body index: a seam
    /// appears once per side, degenerate edges are included, internal edges are not.
    pub edges: Vec<EdgeIndex>,
    /// Signed area enclosed in the `(u, v)` plane (Green's theorem over the p-curves).
    pub parameter_area: Measured,
    /// Outer or inner, from the sign of `parameter_area`.
    pub role: LoopRole,
}

/// Area, centroid and trim structure of one face.
#[derive(Clone, Debug, PartialEq)]
pub struct FaceProperties {
    /// The face's stable body index.
    pub face: FaceIndex,
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
