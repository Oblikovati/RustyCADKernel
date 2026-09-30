// Rust translation of gce_MakeLin.hxx/.cxx and gce_MakeLin2d.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! gce_MakeLin and gce_MakeLin2d: construction of gp_Lin and gp_Lin2d.
use super::{ErrorType, gce_result};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{
    self, Axis1, Axis2d, Direction2, Direction3, Line2, Line3, Point2, Point3, Xy,
};

/// gce_MakeLin: builds a line from an axis, a point and a direction, two points,
/// or a parallel line.
///
/// `let line = MakeLin::from_axis(Axis1::default()).value()?;`
#[derive(Clone, Copy, Debug, Default)]
pub struct MakeLin {
    the_result: Line3,
    the_error: ErrorType,
}
gce_result!(MakeLin, Line3, "gce_MakeLin");

impl MakeLin {
    fn built(line: Line3) -> Self {
        Self {
            the_result: line,
            the_error: ErrorType::Done,
        }
    }

    /// gce_MakeLin(A1): the line of axis `axis`. `MakeLin::from_axis(axis)`.
    pub fn from_axis(axis: Axis1) -> Self {
        Self::built(Line3::from_axis(axis))
    }

    /// gce_MakeLin(P, V): the line through `point` of direction `direction`.
    /// `MakeLin::new(p, d)`.
    pub fn new(point: Point3, direction: Direction3) -> Self {
        Self::built(Line3::new(point, direction))
    }

    /// gce_MakeLin(P1, P2): the line from `p1` toward `p2`; gce_ConfusedPoints
    /// when they are closer than gp::Resolution(). `MakeLin::from_points(a, b)?`.
    pub fn from_points(p1: Point3, p2: Point3) -> Result<Self, Failure> {
        if p1.distance(&p2) >= gp::resolution() {
            return Ok(Self::built(Line3::new(
                p1,
                Direction3::from_xyz(p2.xyz() - p1.xyz())?,
            )));
        }
        Ok(Self {
            the_error: ErrorType::ConfusedPoints,
            ..Self::default()
        })
    }

    /// gce_MakeLin(Lin, P): the line parallel to `line` through `point`.
    /// `MakeLin::from_line_point(line, p)`.
    pub fn from_line_point(line: Line3, point: Point3) -> Self {
        Self::built(Line3::new(point, line.direction()))
    }
}

/// gce_MakeLin2d: builds a 2D line from an axis, a point and a direction, an
/// equation, two points, or a parallel line.
///
/// `let line = MakeLin2d::from_axis(Axis2d::default()).value()?;`
#[derive(Clone, Copy, Debug, Default)]
pub struct MakeLin2d {
    the_result: Line2,
    the_error: ErrorType,
}
gce_result!(MakeLin2d, Line2, "gce_MakeLin2d");

impl MakeLin2d {
    fn built(line: Line2) -> Self {
        Self {
            the_result: line,
            the_error: ErrorType::Done,
        }
    }
    fn failed(error: ErrorType) -> Self {
        Self {
            the_error: error,
            ..Self::default()
        }
    }

    /// gce_MakeLin2d(A): the line of axis `axis`. `MakeLin2d::from_axis(axis)`.
    pub fn from_axis(axis: Axis2d) -> Self {
        Self::built(Line2::new(axis.location(), axis.direction()))
    }

    /// gce_MakeLin2d(P, V): the line through `point` of direction `direction`.
    /// `MakeLin2d::new(p, d)`.
    pub fn new(point: Point2, direction: Direction2) -> Self {
        Self::built(Line2::new(point, direction))
    }

    /// gce_MakeLin2d(A, B, C): the line `a x + b y + c = 0`; gce_NullAxis when
    /// `a² + b²` is at most gp::Resolution(). `MakeLin2d::from_equation(0.0, 1.0, 0.0)?`.
    pub fn from_equation(a: f64, b: f64, c: f64) -> Result<Self, Failure> {
        if a * a + b * b <= gp::resolution() {
            return Ok(Self::failed(ErrorType::NullAxis));
        }
        Ok(Self::built(Line2::from_equation(a, b, c)?))
    }

    /// gce_MakeLin2d(P1, P2): the line from `p1` toward `p2`; gce_ConfusedPoints
    /// when they are closer than gp::Resolution(). `MakeLin2d::from_points(a, b)?`.
    pub fn from_points(p1: Point2, p2: Point2) -> Result<Self, Failure> {
        if p1.distance(p2) >= gp::resolution() {
            return Ok(Self::built(Line2::new(
                p1,
                Direction2::from_xy(p2.xy() - p1.xy())?,
            )));
        }
        Ok(Self::failed(ErrorType::ConfusedPoints))
    }

    /// gce_MakeLin2d(Line, Point): the line parallel to `line` through `point`.
    /// `MakeLin2d::from_line_point(line, p)`.
    pub fn from_line_point(line: Line2, point: Point2) -> Self {
        Self::built(Line2::new(point, line.direction()))
    }

    /// gce_MakeLin2d(Line, Dist): the line parallel to `line` at signed distance
    /// `dist` along its left normal. `MakeLin2d::from_line_distance(line, 1.0)`.
    pub fn from_line_distance(line: Line2, dist: f64) -> Self {
        let direction: Direction2 = line.direction();
        let normal: Xy = Xy::new(-direction.y(), direction.x());
        let point: Point2 = Point2::from(line.location().xy() + normal * dist);
        Self::built(Line2::new(point, direction))
    }
}
