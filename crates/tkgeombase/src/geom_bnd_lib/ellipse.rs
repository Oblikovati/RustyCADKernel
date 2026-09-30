// Rust translation of GeomBndLib_Ellipse and GeomBndLib_Ellipse2d at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomBndLib_Ellipse` and `GeomBndLib_Ellipse2d`: analytic boxes of ellipses and arcs.
use super::elliptic_arc::{ArcTangent, EllipticArc, adjusted_arc, is_full_period};
use super::live;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dEllipse;
use rustycad_tkg3d::geom::GeomEllipse;
use rustycad_tkmath::bnd::{BoundingBox2, BoundingBox3};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Ellipse2, Ellipse3, Point2, Point3, Xy};

/// `GeomBndLib_Ellipse`: the box of a `Geom_Ellipse`, full or on an arc; the static
/// members take a `gp_Elips` directly.
pub struct Ellipse {
    geom: Handle<GeomEllipse>,
}

fn ellipse_arc<'a>(ellipse: &Ellipse3, eval: &'a dyn Fn(f64) -> Point3) -> EllipticArc<'a, Point3> {
    let position = ellipse.position();
    EllipticArc {
        major: ellipse.major_radius(),
        minor: ellipse.minor_radius(),
        origin: *position.location().coordinates(),
        x_dir: *position.x_direction().coordinates(),
        y_dir: *position.y_direction().coordinates(),
        tangent: ArcTangent::Ellipse,
        eval,
    }
}

fn ellipse_value(ellipse: &Ellipse3, u: f64) -> Point3 {
    elclib::ellipse_value(
        u,
        ellipse.position(),
        ellipse.major_radius(),
        ellipse.minor_radius(),
    )
}

impl Ellipse {
    /// The evaluator of `ellipse`; `Ellipse::new(handle)`.
    pub fn new(ellipse: Handle<GeomEllipse>) -> Self {
        Self { geom: ellipse }
    }
    /// `Geometry()`; `evaluator.geometry()`.
    pub fn geometry(&self) -> &Handle<GeomEllipse> {
        &self.geom
    }
    fn elips(&self) -> Result<Ellipse3, Failure> {
        live(&self.geom, "GeomBndLib_Ellipse")?.elips()
    }
    /// `Box(Tol)`: the whole ellipse; `evaluator.bounding_box(0.0)?`.
    pub fn bounding_box(&self, tolerance: f64) -> Result<BoundingBox3, Failure> {
        Ok(Self::elips_box(&self.elips()?, tolerance))
    }
    /// `Box(U1, U2, Tol)`: the arc; `evaluator.bounding_box_range(0.0, 1.0, 0.0)?`.
    pub fn bounding_box_range(
        &self,
        u1: f64,
        u2: f64,
        tolerance: f64,
    ) -> Result<BoundingBox3, Failure> {
        Ok(Self::elips_arc_box(&self.elips()?, u1, u2, tolerance))
    }
    /// `BoxOptimal(U1, U2, Tol)`: the analytic box;
    /// `evaluator.bounding_box_optimal_range(0.0, 1.0, 0.0)?`.
    pub fn bounding_box_optimal_range(
        &self,
        u1: f64,
        u2: f64,
        tolerance: f64,
    ) -> Result<BoundingBox3, Failure> {
        self.bounding_box_range(u1, u2, tolerance)
    }
    /// `Box(gp_Elips, Tol)`: per-coordinate extent `O +/- sqrt(Ra^2 Xk^2 + Rb^2 Yk^2)`;
    /// `Ellipse::elips_box(&ellipse, 0.0)`.
    pub fn elips_box(ellipse: &Ellipse3, tolerance: f64) -> BoundingBox3 {
        let eval = |u: f64| ellipse_value(ellipse, u);
        ellipse_arc(ellipse, &eval).full_box(tolerance)
    }
    /// `Box(gp_Elips, U1, U2, Tol)`: the whole ellipse for a span of at least a period,
    /// otherwise the arc in `[0, 2Pi)` with its ends and interior coordinate extrema;
    /// `Ellipse::elips_arc_box(&ellipse, 0.0, 1.0, 0.0)`.
    pub fn elips_arc_box(ellipse: &Ellipse3, u1: f64, u2: f64, tolerance: f64) -> BoundingBox3 {
        if is_full_period(u1, u2) {
            return Self::elips_box(ellipse, tolerance);
        }
        let (a1, a2) = adjusted_arc(u1, u2);
        let eval = |u: f64| ellipse_value(ellipse, u);
        let mut bounds: BoundingBox3 = BoundingBox3::default();
        ellipse_arc(ellipse, &eval).add_arc(&mut bounds, a1, a2);
        bounds.enlarge(tolerance);
        bounds
    }
}

/// `GeomBndLib_Ellipse2d`: the box of a `Geom2d_Ellipse`; the static members take a
/// `gp_Elips2d`.
pub struct Ellipse2d {
    geom: Handle<Geom2dEllipse>,
}

// The native lambda `gp_Pnt2d(aO + aMajR cos(t) aXd + aMinR sin(t) aYd)`.
fn ellipse2d_eval(ellipse: &Ellipse2, u: f64) -> Point2 {
    let frame = ellipse.axis();
    let origin: Xy = frame.location().xy();
    let x_part: Xy = frame.x_direction().xy() * (ellipse.major_radius() * u.cos());
    let y_part: Xy = frame.y_direction().xy() * (ellipse.minor_radius() * u.sin());
    Point2::from(origin + x_part + y_part)
}

fn ellipse2d_arc<'a>(
    ellipse: &Ellipse2,
    eval: &'a dyn Fn(f64) -> Point2,
) -> EllipticArc<'a, Point2> {
    let frame = ellipse.axis();
    let (location, x, y) = (frame.location(), frame.x_direction(), frame.y_direction());
    EllipticArc {
        major: ellipse.major_radius(),
        minor: ellipse.minor_radius(),
        origin: [location.x(), location.y(), 0.0],
        x_dir: [x.x(), x.y(), 0.0],
        y_dir: [y.x(), y.y(), 0.0],
        tangent: ArcTangent::Ellipse,
        eval,
    }
}

impl Ellipse2d {
    /// The evaluator of `ellipse`; `Ellipse2d::new(handle)`.
    pub fn new(ellipse: Handle<Geom2dEllipse>) -> Self {
        Self { geom: ellipse }
    }
    /// `Geometry()`; `evaluator.geometry()`.
    pub fn geometry(&self) -> &Handle<Geom2dEllipse> {
        &self.geom
    }
    fn elips2d(&self) -> Result<Ellipse2, Failure> {
        live(&self.geom, "GeomBndLib_Ellipse2d")?.elips2d()
    }
    /// `Box(Tol)`: the whole ellipse; `evaluator.bounding_box(0.0)?`.
    pub fn bounding_box(&self, tolerance: f64) -> Result<BoundingBox2, Failure> {
        Ok(Self::elips_box(&self.elips2d()?, tolerance))
    }
    /// `Box(U1, U2, Tol)`: the arc; `evaluator.bounding_box_range(0.0, 1.0, 0.0)?`.
    pub fn bounding_box_range(
        &self,
        u1: f64,
        u2: f64,
        tolerance: f64,
    ) -> Result<BoundingBox2, Failure> {
        Ok(Self::elips_arc_box(&self.elips2d()?, u1, u2, tolerance))
    }
    /// `BoxOptimal(U1, U2, Tol)`: the analytic box;
    /// `evaluator.bounding_box_optimal_range(0.0, 1.0, 0.0)?`.
    pub fn bounding_box_optimal_range(
        &self,
        u1: f64,
        u2: f64,
        tolerance: f64,
    ) -> Result<BoundingBox2, Failure> {
        self.bounding_box_range(u1, u2, tolerance)
    }
    /// `Box(gp_Elips2d, Tol)`; `Ellipse2d::elips_box(&ellipse, 0.0)`.
    pub fn elips_box(ellipse: &Ellipse2, tolerance: f64) -> BoundingBox2 {
        let eval = |u: f64| ellipse2d_eval(ellipse, u);
        ellipse2d_arc(ellipse, &eval).full_box(tolerance)
    }
    /// `Box(gp_Elips2d, U1, U2, Tol)`: the arc parameters are used as given (no
    /// `AdjustPeriodic`, as for Circle2d); `Ellipse2d::elips_arc_box(&ellipse, 0.0, 1.0, 0.0)`.
    pub fn elips_arc_box(ellipse: &Ellipse2, u1: f64, u2: f64, tolerance: f64) -> BoundingBox2 {
        if is_full_period(u1, u2) {
            return Self::elips_box(ellipse, tolerance);
        }
        let eval = |u: f64| ellipse2d_eval(ellipse, u);
        let mut bounds: BoundingBox2 = BoundingBox2::default();
        ellipse2d_arc(ellipse, &eval).add_arc(&mut bounds, u1, u2);
        bounds.enlarge(tolerance);
        bounds
    }
}
