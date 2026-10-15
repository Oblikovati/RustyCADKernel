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
    /// Caller identity.
    pub id: ProfileVertexId,
    /// Plane coordinates `[u, v]` along the plane x and y directions.
    pub point: [f64; 2],
}

/// Traversal sense of a circular arc around its center, seen from the plane normal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArcSense {
    /// Positive rotation about the plane normal.
    CounterClockwise,
    /// Negative rotation about the plane normal.
    Clockwise,
}

/// One solved sketch curve in plane coordinates.
///
/// Lines, circular and elliptic arcs, full circles and ellipses, and clamped polynomial or
/// rational B-splines are admitted (ADR 0083). A spline above degree 16 or with unclamped
/// ends is a named decline (`DeclineCause::UnsupportedCurve`).
#[derive(Clone, Debug, PartialEq)]
pub enum ProfileCurve {
    /// Straight segment between two vertices.
    Line {
        /// Caller identity.
        id: ProfileCurveId,
        /// First vertex.
        start: ProfileVertexId,
        /// Last vertex.
        end: ProfileVertexId,
    },
    /// Arc from `start` to `end` around `center`; both ends must lie on one circle
    /// within the sew tolerance.
    Arc {
        /// Caller identity.
        id: ProfileCurveId,
        /// First vertex.
        start: ProfileVertexId,
        /// Last vertex.
        end: ProfileVertexId,
        /// Circle center in plane coordinates.
        center: [f64; 2],
        /// Traversal sense from `start` to `end`.
        sense: ArcSense,
    },
    /// A full circle: one closed loop with no authored vertex.
    Circle {
        /// Caller identity.
        id: ProfileCurveId,
        /// Center in plane coordinates.
        center: [f64; 2],
        /// Radius in model units.
        radius: f64,
    },
    /// A full ellipse: one closed loop with no authored vertex. Its seam (native parameter
    /// 0) is the end of the major semi-axis.
    Ellipse {
        /// Caller identity.
        id: ProfileCurveId,
        /// Center in plane coordinates.
        center: [f64; 2],
        /// Major semi-axis vector in plane coordinates (its length is the major radius).
        major_axis: [f64; 2],
        /// Minor radius in model units; at most the major radius.
        minor_radius: f64,
    },
    /// Elliptic arc from `start` to `end`; both ends must lie on the ellipse within the sew
    /// tolerance.
    EllipticArc {
        /// Caller identity.
        id: ProfileCurveId,
        /// First vertex.
        start: ProfileVertexId,
        /// Last vertex.
        end: ProfileVertexId,
        /// Center in plane coordinates.
        center: [f64; 2],
        /// Major semi-axis vector in plane coordinates (its length is the major radius).
        major_axis: [f64; 2],
        /// Minor radius in model units; at most the major radius.
        minor_radius: f64,
        /// Traversal sense from `start` to `end`.
        sense: ArcSense,
    },
    /// A clamped polynomial or rational B-spline from `start` to `end` (equal ids give a
    /// closed spline). Its first and last poles must lie within the sew tolerance of the
    /// vertices; the published curve ends exactly at them.
    Spline {
        /// Caller identity.
        id: ProfileCurveId,
        /// First vertex.
        start: ProfileVertexId,
        /// Last vertex.
        end: ProfileVertexId,
        /// Polynomial degree, 1 through 16.
        degree: u32,
        /// Control poles in plane coordinates; at least `degree + 1`.
        poles: Vec<[f64; 2]>,
        /// One positive weight per pole for a rational spline, `None` for a polynomial one.
        weights: Option<Vec<f64>>,
        /// Flat nondecreasing knot vector of `poles.len() + degree + 1` values whose first and
        /// last `degree + 1` values are equal (clamped).
        knots: Vec<f64>,
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
            | Self::EllipticArc { id, .. }
            | Self::Spline { id, .. } => *id,
        }
    }

    /// Authored end vertices, `None` for a closed curve; `curve.ends()`.
    pub fn ends(&self) -> Option<[ProfileVertexId; 2]> {
        match self {
            Self::Line { start, end, .. }
            | Self::Arc { start, end, .. }
            | Self::EllipticArc { start, end, .. }
            | Self::Spline { start, end, .. } => Some([*start, *end]),
            Self::Circle { .. } | Self::Ellipse { .. } => None,
        }
    }
}

/// World placement of the sketch plane. `x_direction` is orthogonalized against `normal`
/// only when it is orthogonal within the angular tolerance; otherwise admission declines.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProfilePlane {
    /// World position of plane coordinates `[0, 0]`.
    pub origin: [f64; 3],
    /// World direction of the plane u axis.
    pub x_direction: [f64; 3],
    /// World plane normal; counterclockwise is positive rotation about it.
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
    /// World placement.
    pub plane: ProfilePlane,
    /// Points; unused points are ignored.
    pub vertices: Vec<ProfileVertex>,
    /// Curves; their order decides loop and region order.
    pub curves: Vec<ProfileCurve>,
}
