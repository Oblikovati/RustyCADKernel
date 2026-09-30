// Rust translation of GC_MakeLine2d and GC_MakeSegment2d (.hxx/.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! GC_MakeLine2d and GC_MakeSegment2d: Geom2d_Line and trimmed Geom2d lines.
use crate::gce::{self, ErrorType};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine, Geom2dTrimmedCurve};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{self, Axis2d, Direction2, Line2, Point2};

/// GC_MakeLine2d: builds a Geom2d_Line from an axis, a gp line, a point and a
/// direction, two points, or a parallel line through a point or at a distance.
///
/// `let line = MakeLine2d::from_axis(Axis2d::default()).value()?;`
#[derive(Clone, Default)]
pub struct MakeLine2d {
    the_result: Handle<Geom2dLine>,
    the_error: ErrorType,
}
gc_result!(MakeLine2d, Geom2dLine, "GC_MakeLine2d");
gc_from_gce!(MakeLine2d, gce::MakeLin2d, Geom2dLine::from_lin2d);

impl MakeLine2d {
    /// GC_MakeLine2d(A): the line of axis `axis`. `MakeLine2d::from_axis(axis)`.
    pub fn from_axis(axis: Axis2d) -> Self {
        Self::built(Geom2dLine::new(axis))
    }

    /// GC_MakeLine2d(L): the Geom2d line of `line`. `MakeLine2d::from_lin(line)`.
    pub fn from_lin(line: Line2) -> Self {
        Self::built(Geom2dLine::from_lin2d(line))
    }

    /// GC_MakeLine2d(P, V): the line through `point` of direction `direction`.
    /// `MakeLine2d::new(p, d)`.
    pub fn new(point: Point2, direction: Direction2) -> Self {
        Self::built(Geom2dLine::from_point(point, direction))
    }

    /// GC_MakeLine2d(P1, P2): gce_MakeLin2d(P1, P2). `MakeLine2d::from_points(a, b)?`.
    pub fn from_points(p1: Point2, p2: Point2) -> Result<Self, Failure> {
        Self::from_gce(gce::MakeLin2d::from_points(p1, p2)?)
    }

    /// GC_MakeLine2d(Lin, Point): gce_MakeLin2d(Lin, Point).
    /// `MakeLine2d::from_line_point(line, p)?`.
    pub fn from_line_point(line: Line2, point: Point2) -> Result<Self, Failure> {
        Self::from_gce(gce::MakeLin2d::from_line_point(line, point))
    }

    /// GC_MakeLine2d(Lin, Dist): gce_MakeLin2d(Lin, Dist).
    /// `MakeLine2d::from_line_distance(line, 1.0)?`.
    pub fn from_line_distance(line: Line2, dist: f64) -> Result<Self, Failure> {
        Self::from_gce(gce::MakeLin2d::from_line_distance(line, dist))
    }
}

/// GC_MakeSegment2d: builds a trimmed Geom2d_Line from a point, a direction and
/// an end point, two points, or a line and points or parameters.
///
/// `let segment = MakeSegment2d::from_parameters(line, 0.0, 1.0)?.value()?;`
#[derive(Clone, Default)]
pub struct MakeSegment2d {
    the_result: Handle<Geom2dTrimmedCurve>,
    the_error: ErrorType,
}
gc_result!(MakeSegment2d, Geom2dTrimmedCurve, "GC_MakeSegment2d");

impl MakeSegment2d {
    /// `Geom2d_TrimmedCurve(L, U1, U2, true)`: the segment of `line` between two
    /// parameters.
    fn trimmed(line: Handle<Geom2dLine>, u1: f64, u2: f64) -> Result<Self, Failure> {
        let basis: Handle<dyn Geom2dCurve> = line.cast();
        Ok(Self::built(Geom2dTrimmedCurve::new(
            &basis, u1, u2, true, true,
        )?))
    }

    /// GC_MakeSegment2d(P1, V, P2): the segment of the line (P1, V) from P1 to the
    /// parameter of P2; gce_ConfusedPoints when that parameter is within
    /// gp::Resolution() of 0. `MakeSegment2d::from_direction(p1, d, p2)?`.
    pub fn from_direction(p1: Point2, direction: Direction2, p2: Point2) -> Result<Self, Failure> {
        let line = Line2::new(p1, direction);
        let u_last: f64 = elclib::parameter(&line, p2)?;
        if u_last.abs() > gp::resolution() {
            return Self::trimmed(Handle::new(Geom2dLine::from_lin2d(line)), 0.0, u_last);
        }
        Ok(Self::failed(ErrorType::ConfusedPoints))
    }

    /// GC_MakeSegment2d(P1, P2): the line of GC_MakeLine2d(P1, P2) trimmed to
    /// `[0, |P1 P2|]`; gce_ConfusedPoints when the points are within
    /// gp::Resolution(). `MakeSegment2d::from_points(a, b)?`.
    pub fn from_points(p1: Point2, p2: Point2) -> Result<Self, Failure> {
        let dist: f64 = p1.distance(p2);
        if dist.is_nan() || dist <= gp::resolution() {
            return Ok(Self::failed(ErrorType::ConfusedPoints));
        }
        let line: Handle<Geom2dLine> = Handle::try_from(&MakeLine2d::from_points(p1, p2)?)?;
        Self::trimmed(line, 0.0, dist)
    }

    /// GC_MakeSegment2d(Line, Point, U): the segment from the parameter of `point`
    /// to `u`. `MakeSegment2d::from_point_parameter(line, p, 1.0)?`.
    pub fn from_point_parameter(line: Line2, point: Point2, u: f64) -> Result<Self, Failure> {
        let u_first: f64 = elclib::parameter(&line, point)?;
        Self::trimmed(Handle::new(Geom2dLine::from_lin2d(line)), u_first, u)
    }

    /// GC_MakeSegment2d(Line, P1, P2): the segment between the parameters of `p1`
    /// and `p2`. `MakeSegment2d::from_line_points(line, a, b)?`.
    pub fn from_line_points(line: Line2, p1: Point2, p2: Point2) -> Result<Self, Failure> {
        let u_first: f64 = elclib::parameter(&line, p1)?;
        let u_last: f64 = elclib::parameter(&line, p2)?;
        Self::trimmed(Handle::new(Geom2dLine::from_lin2d(line)), u_first, u_last)
    }

    /// GC_MakeSegment2d(Line, U1, U2): the segment between two parameters.
    /// `MakeSegment2d::from_parameters(line, 0.0, 1.0)?`.
    pub fn from_parameters(line: Line2, u1: f64, u2: f64) -> Result<Self, Failure> {
        Self::trimmed(Handle::new(Geom2dLine::from_lin2d(line)), u1, u2)
    }
}
