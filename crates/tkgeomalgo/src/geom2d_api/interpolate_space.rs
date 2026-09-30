// Rust translation of the dimension-dependent parts of Geom2dAPI_Interpolate.cxx and
// GeomAPI_Interpolate.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The space of an interpolation: Geom2dAPI_Interpolate and GeomAPI_Interpolate have
//! the same body over `gp_Pnt2d`/`gp_Vec2d`/`Geom2d_BSplineCurve` and
//! `gp_Pnt`/`gp_Vec`/`Geom_BSplineCurve` (the loops run over 2 or 3 coordinates),
//! so the body is one generic over this trait (docs/geomalgo-design.md section 15).
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, RuntimeTyped, Transient};
use rustycad_tkg2d::geom2d::Geom2dBSplineCurve;
use rustycad_tkmath::bsplclib::CurvePoint;
use rustycad_tkmath::gp::{Point2, Vector2};

/// The point, the vector and the B-spline curve of one instantiation, with the
/// point and vector members the interpolation calls.
pub trait InterpolateSpace: 'static {
    /// `gp_Pnt2d` or `gp_Pnt`.
    type Point: CurvePoint<Vector = Self::Vector> + Send + Sync + std::fmt::Debug;
    /// `gp_Vec2d` or `gp_Vec`.
    type Vector: Copy + Default + Send + Sync + std::fmt::Debug + PartialEq + 'static;
    /// `Geom2d_BSplineCurve` or `Geom_BSplineCurve`.
    type Curve: RuntimeTyped + Transient + Send + Sync;
    /// The native class name, for the exception messages.
    const NAME: &'static str;
    /// `A.SquareDistance(B)`.
    fn square_distance(a: &Self::Point, b: &Self::Point) -> f64;
    /// `A.Distance(B)`.
    fn distance(a: &Self::Point, b: &Self::Point) -> f64;
    /// `V.SquareMagnitude()`.
    fn square_magnitude(v: &Self::Vector) -> f64;
    /// `new Geom*_BSplineCurve(Poles, Knots, Mults, Degree, Periodic)`.
    fn new_curve(
        poles: &Array1<'_, Self::Point>,
        knots: &Array1<'_, f64>,
        mults: &Array1<'_, i32>,
        degree: i32,
        periodic: bool,
    ) -> Result<Self::Curve, Failure>;
}

/// The coordinates of a vector as a point (the native `a_point.SetCoord(jj,
/// Tangent.Coord(jj))` loops).
pub(super) fn vector_as_point<S: InterpolateSpace>(v: &S::Vector) -> S::Point {
    let dimension: usize = <S::Point as rustycad_tkmath::plib::PolePoint>::DIMENSION;
    let coordinates: Vec<f64> = (0..dimension)
        .map(|k| S::Point::vector_coordinate(v, k))
        .collect();
    S::Point::point_from(&coordinates)
}

/// The 2D space of Geom2dAPI_Interpolate.
pub struct Interpolate2dSpace;

impl InterpolateSpace for Interpolate2dSpace {
    type Point = Point2;
    type Vector = Vector2;
    type Curve = Geom2dBSplineCurve;
    const NAME: &'static str = "Geom2dAPI_Interpolate";
    fn square_distance(a: &Point2, b: &Point2) -> f64 {
        a.square_distance(*b)
    }
    fn distance(a: &Point2, b: &Point2) -> f64 {
        a.distance(*b)
    }
    fn square_magnitude(v: &Vector2) -> f64 {
        v.square_magnitude()
    }
    fn new_curve(
        poles: &Array1<'_, Point2>,
        knots: &Array1<'_, f64>,
        mults: &Array1<'_, i32>,
        degree: i32,
        periodic: bool,
    ) -> Result<Geom2dBSplineCurve, Failure> {
        Geom2dBSplineCurve::new(poles, knots, mults, degree, periodic)
    }
}
