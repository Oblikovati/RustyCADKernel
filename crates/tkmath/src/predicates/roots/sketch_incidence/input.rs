// SPDX-License-Identifier: GPL-2.0-only
//! Owned admitted sketch curve representations; every value is an exact binary64 input.

/// Which part of the source line the consumer edits; contacts are always certified on the support.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SketchLineDomain {
    /// Closed segment between the two source points (trim).
    Segment,
    /// Infinite supporting line through the two source points (extension).
    Support,
}
/// The edited curve, its occurrence key and the consumer domain.
///
/// A line source is parameterized `A + t (B - A)`; a spline source by its own
/// parameter; a conic source by its sweep from the sector start (or frame axis).
#[derive(Clone, Debug, PartialEq)]
pub struct SketchIncidenceSource {
    /// Caller occurrence key; it identifies the source and certifies nothing.
    pub id: u64,
    /// Exact source curve; a line source must be bounded (its two points are the ends).
    pub curve: SketchIncidenceCurve,
    /// Requested consumer domain.
    pub domain: SketchLineDomain,
}
impl SketchIncidenceSource {
    /// Line source through two distinct points (t = 0 at `points[0]`, t = 1 at `points[1]`).
    /// Example: `SketchIncidenceSource::line(1, [[0., 0.], [1., 0.]], SketchLineDomain::Segment)`.
    pub fn line(id: u64, points: [[f64; 2]; 2], domain: SketchLineDomain) -> Self {
        Self {
            id,
            curve: SketchIncidenceCurve::Line {
                points,
                bounded: true,
            },
            domain,
        }
    }
}
/// Angular sector of a conic, bounded by directions from the center.
///
/// Directions are any nonzero vectors, typically authored endpoint minus center.
/// The sector sweeps from `start` to `end` in the stated rotational sense.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SketchConicSector {
    /// Direction of the start boundary ray.
    pub start: [f64; 2],
    /// Direction of the end boundary ray; not the same ray as `start`.
    pub end: [f64; 2],
    /// True for counterclockwise travel from `start` to `end`.
    pub counterclockwise: bool,
}
/// Angular sector of a conic given by parameter angles in radians.
///
/// The frame places angle `t` at `C + ra cos(t) a + rb sin(t) n`, with `a` the
/// unit `axis` and `n` its left normal (right normal when clockwise); a circle has
/// `ra = rb`. The binary64 bounds are exact real angles: a contact is never exactly
/// on a nonzero bound (its direction is algebraic, `tan(t)` is transcendental), so
/// only a bound of exactly `0` can report `AtStart` or `AtEnd`. Prefer rays for
/// authored arcs: an angle rounded from authored points is not their direction.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SketchConicAngles {
    /// Frame axis; for an ellipse, parallel to its major axis (either direction).
    pub axis: [f64; 2],
    /// True when angles increase counterclockwise.
    pub counterclockwise: bool,
    /// Start and end angles: `start < end`, `end - start` below one full turn,
    /// magnitudes at most 64.
    pub bounds: [f64; 2],
}
/// Bounds of a conic arc: boundary rays, or parameter angles.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SketchConicBounds {
    /// Exact boundary directions (preferred for authored arcs).
    Rays(SketchConicSector),
    /// Parameter angles in a conic frame, e.g. an application's radian span.
    Angles(SketchConicAngles),
}
impl From<SketchConicSector> for SketchConicBounds {
    fn from(sector: SketchConicSector) -> Self {
        Self::Rays(sector)
    }
}
impl From<SketchConicAngles> for SketchConicBounds {
    fn from(angles: SketchConicAngles) -> Self {
        Self::Angles(angles)
    }
}
/// One admitted curve representation, as a source or a target.
#[derive(Clone, Debug, PartialEq)]
pub enum SketchIncidenceCurve {
    /// Line through two distinct points: the closed segment, or the whole line when unbounded.
    Line {
        /// Start (target parameter 0) and end (parameter 1).
        points: [[f64; 2]; 2],
        /// False for an infinite supporting line, used by extension queries.
        bounded: bool,
    },
    /// Circle, or a circular arc when a sector is supplied.
    Circle {
        /// Exact center.
        center: [f64; 2],
        /// Strictly positive radius.
        radius: f64,
        /// None for the full periodic curve.
        sector: Option<SketchConicBounds>,
    },
    /// Ellipse, or an elliptical arc when a sector is supplied.
    Ellipse {
        /// Exact center.
        center: [f64; 2],
        /// Nonzero major-axis direction; its length is irrelevant.
        major_axis: [f64; 2],
        /// Strictly positive radii along the major axis and its left normal.
        radii: [f64; 2],
        /// None for the full periodic curve.
        sector: Option<SketchConicBounds>,
    },
    /// Non-periodic B-spline with flat knots, as built by the owned sketch constructor.
    Spline(SketchIncidenceSpline),
    /// Periodic B-spline with native knots and multiplicities (ADR 0080).
    PeriodicSpline(SketchPeriodicSpline),
    /// Offset of a basis curve at a signed distance (ADR 0080).
    Offset(SketchIncidenceOffset),
}
/// Offset curve `C(u) + distance * (C'(u) rotated clockwise) / |C'(u)|`, as
/// `Geom2d_OffsetCurve`: a positive distance lies right of the travel direction.
///
/// Bases: a line, a circle or ellipse (a full conic travels counterclockwise, an
/// arc in its sector sense), or a spline (periodic or not) that is C1 and regular
/// on its bounds. A circle offset is the concentric circle; a zero distance is the
/// basis itself. The offset keeps the basis parameterization, sector and bounds.
#[derive(Clone, Debug, PartialEq)]
pub struct SketchIncidenceOffset {
    /// Basis curve; it must not itself be an offset.
    pub basis: Box<SketchIncidenceCurve>,
    /// Finite signed offset distance.
    pub distance: f64,
}
/// Periodic B-spline in the native OCCT form: distinct knots with multiplicities.
///
/// It is unwrapped exactly into flat knots by `BSplCLib::KnotSequence`; every
/// knot shifted by the period must be exact in binary64 (`PeriodicKnotShift`).
#[derive(Clone, Debug, PartialEq)]
pub struct SketchPeriodicSpline {
    /// Degree, at least one.
    pub degree: usize,
    /// Control points of one period: the sum of the multiplicities but the last.
    pub poles: Vec<[f64; 2]>,
    /// Optional strictly positive weights, one per pole; None is non-rational.
    pub weights: Option<Vec<f64>>,
    /// Strictly increasing distinct knots; the period is `last - first`.
    pub knots: Vec<f64>,
    /// Multiplicities: interior ones in `1..=degree`, first equal to last.
    pub multiplicities: Vec<usize>,
    /// Trimmed parameter range inside `[knots[0], knots[last]]`, lower < upper.
    pub bounds: [f64; 2],
}
/// Exact B-spline recipe; closed sketch splines use their wrapped (unwrapped-knot) form.
#[derive(Clone, Debug, PartialEq)]
pub struct SketchIncidenceSpline {
    /// Degree, at least one.
    pub degree: usize,
    /// Control points; at least `degree + 1`.
    pub poles: Vec<[f64; 2]>,
    /// Optional strictly positive weights, one per pole; None is non-rational.
    pub weights: Option<Vec<f64>>,
    /// Nondecreasing knots with repetitions; length `poles + degree + 1`.
    pub flat_knots: Vec<f64>,
    /// Trimmed parameter range inside `[knots[degree], knots[poles]]`, lower < upper.
    pub bounds: [f64; 2],
}
/// One target occurrence. Identity comes from `id`, never from order or proximity.
#[derive(Clone, Debug, PartialEq)]
pub struct SketchIncidenceTarget {
    /// Caller occurrence key.
    pub id: u64,
    /// Exact target representation.
    pub curve: SketchIncidenceCurve,
}
/// Explicit version-one limits. No limit is widened or defaulted to unbounded work.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SketchIncidenceOptions {
    /// Policy version; this implementation accepts 1.
    pub version: u32,
    /// Largest admitted polynomial degree of any derived equation.
    pub max_degree: u16,
    /// Subdivision limit for each complete equation isolation.
    pub max_isolation_bisections: u16,
    /// Total bracket refinements for construction, classification and ordering.
    pub max_refinements: u32,
    /// Finite positive Euclidean construction error limit, in source world units.
    pub max_world_error: f64,
}
impl SketchIncidenceOptions {
    /// Version-one work limits with an explicit world error limit.
    /// Example: `SketchIncidenceOptions::v1(1e-9)` admits degree 32.
    pub fn v1(max_world_error: f64) -> Self {
        Self {
            version: 1,
            max_degree: 32,
            max_isolation_bisections: 4096,
            max_refinements: 100_000,
            max_world_error,
        }
    }
}
