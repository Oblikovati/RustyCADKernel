// SPDX-License-Identifier: GPL-2.0-only
//! Admitted profile: checked curves in caller order and the orthonormal world frame.
use super::ellipse::EllipseCarrier;
use super::profile::{ArcSense, ProfileCurveId, ProfileVertexId};
use super::resolution::Resolution;
use super::spline::AdmittedSpline;
use std::collections::BTreeMap;

/// Orthonormal right-handed sketch frame in world coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct WorldFrame {
    pub origin: [f64; 3],
    pub x: [f64; 3],
    pub y: [f64; 3],
    pub normal: [f64; 3],
}

impl WorldFrame {
    /// World point of plane coordinates `[u, v]`; one evaluation per authored vertex keeps
    /// shared ends bitwise equal.
    pub(crate) fn world(&self, [u, v]: [f64; 2]) -> [f64; 3] {
        std::array::from_fn(|axis| self.origin[axis] + u * self.x[axis] + v * self.y[axis])
    }

    /// World direction of the plane vector `[u, v]`.
    pub(crate) fn direction(&self, [u, v]: [f64; 2]) -> [f64; 3] {
        std::array::from_fn(|axis| u * self.x[axis] + v * self.y[axis])
    }
}

/// A checked circle carrier: center and radius in plane coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CircleCarrier {
    pub center: [f64; 2],
    pub radius: f64,
}

/// A checked admitted curve.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum AdmittedCurve {
    Line {
        id: ProfileCurveId,
        ends: [ProfileVertexId; 2],
    },
    Arc {
        id: ProfileCurveId,
        ends: [ProfileVertexId; 2],
        carrier: CircleCarrier,
        sense: ArcSense,
    },
    Circle {
        id: ProfileCurveId,
        carrier: CircleCarrier,
    },
    EllipticArc {
        id: ProfileCurveId,
        ends: [ProfileVertexId; 2],
        carrier: EllipseCarrier,
        sense: ArcSense,
    },
    Ellipse {
        id: ProfileCurveId,
        carrier: EllipseCarrier,
    },
    Spline {
        id: ProfileCurveId,
        ends: [ProfileVertexId; 2],
        spline: AdmittedSpline,
    },
}

impl AdmittedCurve {
    pub(crate) fn id(&self) -> ProfileCurveId {
        match self {
            Self::Line { id, .. }
            | Self::Arc { id, .. }
            | Self::Circle { id, .. }
            | Self::EllipticArc { id, .. }
            | Self::Ellipse { id, .. }
            | Self::Spline { id, .. } => *id,
        }
    }

    pub(crate) fn ends(&self) -> Option<[ProfileVertexId; 2]> {
        match self {
            Self::Line { ends, .. }
            | Self::Arc { ends, .. }
            | Self::EllipticArc { ends, .. }
            | Self::Spline { ends, .. } => Some(*ends),
            Self::Circle { .. } | Self::Ellipse { .. } => None,
        }
    }
}

/// The admitted profile; every later stage reads only this value.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AdmittedProfile {
    pub frame: WorldFrame,
    pub resolution: Resolution,
    pub vertices: BTreeMap<ProfileVertexId, [f64; 2]>,
    pub curves: Vec<AdmittedCurve>,
}

impl AdmittedProfile {
    /// Plane coordinates of an admitted vertex.
    pub(crate) fn point(&self, vertex: ProfileVertexId) -> [f64; 2] {
        self.vertices[&vertex]
    }

    /// Admitted curve with caller id `id`.
    pub(crate) fn curve(&self, id: ProfileCurveId) -> &AdmittedCurve {
        self.curves
            .iter()
            .find(|curve| curve.id() == id)
            .expect("loop uses name admitted curves only")
    }
}
