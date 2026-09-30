// Rust translation of GC_MakeArcOfCircle2d, GC_MakeArcOfEllipse2d, GC_MakeArcOfHyperbola2d
// and GC_MakeArcOfParabola2d (.hxx/.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The 2D arc algorithms: trimmed Geom2d conics.
use crate::gce::{self, ErrorType, Root};
use crate::int_ana2d::AnaIntersection;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::{
    Geom2dCircle, Geom2dCurve, Geom2dEllipse, Geom2dHyperbola, Geom2dParabola, Geom2dTrimmedCurve,
};
use rustycad_tkmath::gp::{
    Circle2, Direction2, Ellipse2, Hyperbola2, Line2, Parabola2, Point2, Vector2,
};

conic_arc!(
    MakeArcOfCircle2d,
    "GC_MakeArcOfCircle2d",
    Circle2,
    Point2,
    Geom2dCircle::from_circ2d,
    Geom2dCurve,
    Geom2dTrimmedCurve
);
conic_arc!(
    MakeArcOfEllipse2d,
    "GC_MakeArcOfEllipse2d",
    Ellipse2,
    Point2,
    Geom2dEllipse::from_elips2d,
    Geom2dCurve,
    Geom2dTrimmedCurve
);
conic_arc!(
    MakeArcOfHyperbola2d,
    "GC_MakeArcOfHyperbola2d",
    Hyperbola2,
    Point2,
    Geom2dHyperbola::from_hypr2d,
    Geom2dCurve,
    Geom2dTrimmedCurve
);
conic_arc!(
    MakeArcOfParabola2d,
    "GC_MakeArcOfParabola2d",
    Parabola2,
    Point2,
    Geom2dParabola::from_parab2d,
    Geom2dCurve,
    Geom2dTrimmedCurve
);

impl MakeArcOfCircle2d {
    /// GC_MakeArcOfCircle2d(P1, P2, P3): the arc of the gce circle through the
    /// three points, from the parameter of P1 to that of P3; the status of
    /// gce_MakeCirc2d. `MakeArcOfCircle2d::through_points(a, b, c)?`.
    pub fn through_points(p1: Point2, p2: Point2, p3: Point2) -> Result<Self, Failure> {
        let circle = gce::MakeCirc2d::through_points(p1, p2, p3)?;
        if circle.status() != ErrorType::Done {
            return Ok(Self::failed(circle.status()));
        }
        Self::from_points(circle.value()?, p1, p3, true)
    }

    /// GC_MakeArcOfCircle2d(P1, V, P2): the arc from P1 to P2 tangent to `tangent`
    /// at P1, centered where the chord's bisector meets the normal at P1, and
    /// direct when `tangent ^ chord > 0`. The chord comes from the conversion of
    /// gce_MakeLin2d(P1, P2), so confused points raise StdFail_NotDone; a normal
    /// parallel to the bisector leaves gce_ConfusedPoints.
    /// `MakeArcOfCircle2d::from_tangent(p1, v, p2)?`.
    pub fn from_tangent(p1: Point2, tangent: Vector2, p2: Point2) -> Result<Self, Failure> {
        let dir: Direction2 = gce::MakeLin2d::from_points(p1, p2)?.operator()?.direction();
        let middle = Point2::new((p1.x() + p2.x()) / 2.0, (p1.y() + p2.y()) / 2.0);
        let bisector = Line2::new(middle, Direction2::new(-dir.y(), dir.x())?);
        let normal = Line2::new(p1, Direction2::new(-tangent.y(), tangent.x())?);
        let intersection = AnaIntersection::lines(bisector, normal);
        if !intersection.is_done() || intersection.is_empty()? {
            return Ok(Self::failed(ErrorType::ConfusedPoints));
        }
        let center: Point2 = intersection.point(1)?.value();
        let radius: f64 = (center.distance(p1) + center.distance(p2)) / 2.0;
        let circle: Circle2 = gce::MakeCirc2d::from_center(center, radius, true)?.operator()?;
        let sense: bool = tangent.crossed(Vector2::from(dir)) > 0.0;
        Self::from_points(circle, p1, p2, sense)
    }
}
