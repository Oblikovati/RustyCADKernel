// SPDX-License-Identifier: GPL-2.0-only
//! Public solved-sketch input (ADR 0068): curves join through caller vertex ids, never by
//! coordinate proximity, so identity survives every coordinate edit.

/// Caller identity of a sketch point; equal ids are one topological vertex.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProfileVertexId(pub u64);

/// Caller identity of a sketch curve; it names every edge and face generated from it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProfileCurveId(pub u64);

/// A solved sketch point in plane coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProfileVertex {
    pub id: ProfileVertexId,
    pub point: [f64; 2],
}

/// Traversal sense of a circular arc around its center, seen from the plane normal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArcSense {
    CounterClockwise,
    Clockwise,
}

/// One solved sketch curve in plane coordinates.
///
/// Lines, circular arcs and full circles are admitted. Ellipses and splines are named
/// declines (`RegionDeclineCause::UnsupportedCurve`) until their published carrier and
/// history are certified (ADR 0068 "Limits").
#[derive(Clone, Debug, PartialEq)]
pub enum ProfileCurve {
    Line {
        id: ProfileCurveId,
        start: ProfileVertexId,
        end: ProfileVertexId,
    },
    /// Arc from `start` to `end` around `center`; both ends must lie on one circle
    /// within the sew tolerance.
    Arc {
        id: ProfileCurveId,
        start: ProfileVertexId,
        end: ProfileVertexId,
        center: [f64; 2],
        sense: ArcSense,
    },
    /// A full circle: one closed loop with no authored vertex.
    Circle {
        id: ProfileCurveId,
        center: [f64; 2],
        radius: f64,
    },
    /// A full ellipse; currently declined.
    Ellipse {
        id: ProfileCurveId,
        center: [f64; 2],
        major_axis: [f64; 2],
        minor_radius: f64,
    },
    /// A spline through caller poles; currently declined.
    Spline {
        id: ProfileCurveId,
        start: ProfileVertexId,
        end: ProfileVertexId,
        poles: Vec<[f64; 2]>,
    },
}

impl ProfileCurve {
    /// The caller id of this curve; `curve.id()`.
    pub fn id(&self) -> ProfileCurveId {
        match self {
            Self::Line { id, .. }
            | Self::Arc { id, .. }
            | Self::Circle { id, .. }
            | Self::Ellipse { id, .. }
            | Self::Spline { id, .. } => *id,
        }
    }

    /// Authored end vertices, `None` for a closed curve; `curve.ends()`.
    pub fn ends(&self) -> Option<[ProfileVertexId; 2]> {
        match self {
            Self::Line { start, end, .. }
            | Self::Arc { start, end, .. }
            | Self::Spline { start, end, .. } => Some([*start, *end]),
            Self::Circle { .. } | Self::Ellipse { .. } => None,
        }
    }
}

/// World placement of the sketch plane. `x_direction` is orthogonalized against `normal`
/// only when it is orthogonal within the angular tolerance; otherwise admission declines.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProfilePlane {
    pub origin: [f64; 3],
    pub x_direction: [f64; 3],
    pub normal: [f64; 3],
}

impl ProfilePlane {
    /// The world XY plane; `ProfilePlane::world_xy()`.
    pub fn world_xy() -> Self {
        Self {
            origin: [0.0; 3],
            x_direction: [1.0, 0.0, 0.0],
            normal: [0.0, 0.0, 1.0],
        }
    }
}

/// A solved sketch: the plane, its points and its curves, in caller order.
#[derive(Clone, Debug, PartialEq)]
pub struct SketchProfile {
    pub plane: ProfilePlane,
    pub vertices: Vec<ProfileVertex>,
    pub curves: Vec<ProfileCurve>,
}
