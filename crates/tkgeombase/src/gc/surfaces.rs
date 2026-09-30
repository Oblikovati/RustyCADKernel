// Rust translation of GC_MakePlane, GC_MakeConicalSurface and GC_MakeCylindricalSurface
// (.hxx/.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! GC_MakePlane, GC_MakeConicalSurface and GC_MakeCylindricalSurface: the Geom
//! elementary surfaces.
use crate::gce::{self, ErrorType};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomConicalSurface, GeomCylindricalSurface, GeomPlane};
use rustycad_tkmath::gp::{
    self, Axis1, Axis2, Axis3, Circle3, Cone3, Cylinder3, Direction3, Line3, Plane3, Point3,
};
use std::f64::consts::PI;

/// GC_MakePlane: builds a Geom_Plane from a gp plane, a point and a normal, an
/// equation, three points, a parallel plane, or an axis.
///
/// `let plane = MakePlane::from_pln(Plane3::default()).value()?;`
#[derive(Clone, Default)]
pub struct MakePlane {
    the_result: Handle<GeomPlane>,
    the_error: ErrorType,
}
gc_result!(MakePlane, GeomPlane, "GC_MakePlane");
gc_from_gce!(MakePlane, gce::MakePln, GeomPlane::from_pln);

impl MakePlane {
    /// GC_MakePlane(Pl): the Geom plane of `plane`. `MakePlane::from_pln(plane)`.
    pub fn from_pln(plane: Plane3) -> Self {
        Self::built(GeomPlane::from_pln(plane))
    }

    /// GC_MakePlane(P, V): the plane through `point` of normal `normal`.
    /// `MakePlane::new(p, n)?`.
    pub fn new(point: Point3, normal: Direction3) -> Result<Self, Failure> {
        Ok(Self::built(GeomPlane::from_point(point, normal)?))
    }

    /// GC_MakePlane(A, B, C, D): the plane `gp_Pln(A, B, C, D)`; gce_BadEquation
    /// when `sqrt(a² + b² + c²)` is at most gp::Resolution() (gce_MakePln compares
    /// the square). `MakePlane::from_equation(0.0, 0.0, 1.0, 0.0)?`.
    pub fn from_equation(a: f64, b: f64, c: f64, d: f64) -> Result<Self, Failure> {
        if (a * a + b * b + c * c).sqrt() <= gp::resolution() {
            return Ok(Self::failed(ErrorType::BadEquation));
        }
        Ok(Self::built(GeomPlane::from_pln(Plane3::from_equation(
            a, b, c, d,
        )?)))
    }

    /// GC_MakePlane(P1, P2, P3): gce_MakePln(P1, P2, P3).
    /// `MakePlane::through_points(a, b, c)?`.
    pub fn through_points(p1: Point3, p2: Point3, p3: Point3) -> Result<Self, Failure> {
        Self::from_gce(gce::MakePln::through_points(p1, p2, p3)?)
    }

    /// GC_MakePlane(Pl, Dist): gce_MakePln(Pl, Dist).
    /// `MakePlane::from_plane_distance(plane, 1.0)?`.
    pub fn from_plane_distance(plane: Plane3, dist: f64) -> Result<Self, Failure> {
        Self::from_gce(gce::MakePln::from_plane_distance(plane, dist)?)
    }

    /// GC_MakePlane(Pl, Point): gce_MakePln(Pl, Point).
    /// `MakePlane::from_plane_point(plane, p)?`.
    pub fn from_plane_point(plane: Plane3, point: Point3) -> Result<Self, Failure> {
        Self::from_gce(gce::MakePln::from_plane_point(plane, point)?)
    }

    /// GC_MakePlane(Axis): gce_MakePln(Axis). `MakePlane::from_axis(axis)?`.
    pub fn from_axis(axis: Axis1) -> Result<Self, Failure> {
        Self::from_gce(gce::MakePln::from_axis(axis)?)
    }
}

/// GC_MakeConicalSurface: builds a Geom_ConicalSurface from a frame, a gp cone,
/// four points, or two points and two radii.
///
/// `let cone = MakeConicalSurface::new(Axis2::default(), 0.5, 1.0)?.value()?;`
#[derive(Clone, Default)]
pub struct MakeConicalSurface {
    the_result: Handle<GeomConicalSurface>,
    the_error: ErrorType,
}
gc_result!(
    MakeConicalSurface,
    GeomConicalSurface,
    "GC_MakeConicalSurface"
);
gc_from_gce!(
    MakeConicalSurface,
    gce::MakeCone,
    GeomConicalSurface::from_cone
);

impl MakeConicalSurface {
    /// GC_MakeConicalSurface(A2, Ang, Radius): gce_NegativeRadius, then
    /// gce_BadAngle when `angle <= Resolution` or `angle >= PI/2 - Resolution`
    /// (gce_MakeCone tests `PI/2 - angle <= Resolution`).
    /// `MakeConicalSurface::new(frame, 0.5, 1.0)?`.
    pub fn new(frame: Axis2, angle: f64, radius: f64) -> Result<Self, Failure> {
        if radius < 0.0 {
            return Ok(Self::failed(ErrorType::NegativeRadius));
        }
        if angle <= gp::resolution() || angle >= PI / 2.0 - gp::resolution() {
            return Ok(Self::failed(ErrorType::BadAngle));
        }
        let cone = GeomConicalSurface::new(Axis3::from_axis2(frame), angle, radius)?;
        Ok(Self::built(cone))
    }

    /// GC_MakeConicalSurface(C): the Geom cone of `cone`.
    /// `MakeConicalSurface::from_cone(cone)`.
    pub fn from_cone(cone: Cone3) -> Self {
        Self::built(GeomConicalSurface::from_cone(cone))
    }

    /// GC_MakeConicalSurface(P1, P2, P3, P4): gce_MakeCone(P1, P2, P3, P4).
    /// `MakeConicalSurface::through_points(p1, p2, p3, p4)?`.
    pub fn through_points(p1: Point3, p2: Point3, p3: Point3, p4: Point3) -> Result<Self, Failure> {
        Self::from_gce(gce::MakeCone::through_points(p1, p2, p3, p4)?)
    }

    /// GC_MakeConicalSurface(P1, P2, R1, R2): gce_MakeCone(P1, P2, R1, R2).
    /// `MakeConicalSurface::from_points_radii(p1, p2, 1.0, 2.0)?`.
    pub fn from_points_radii(p1: Point3, p2: Point3, r1: f64, r2: f64) -> Result<Self, Failure> {
        Self::from_gce(gce::MakeCone::from_points_radii(p1, p2, r1, r2)?)
    }
}

/// GC_MakeCylindricalSurface: builds a Geom_CylindricalSurface from a gp
/// cylinder, a frame, an axis, a circle, three points, or a parallel cylinder.
///
/// `let cylinder = MakeCylindricalSurface::new(Axis2::default(), 1.0)?.value()?;`
#[derive(Clone, Default)]
pub struct MakeCylindricalSurface {
    the_result: Handle<GeomCylindricalSurface>,
    the_error: ErrorType,
}
gc_result!(
    MakeCylindricalSurface,
    GeomCylindricalSurface,
    "GC_MakeCylindricalSurface"
);
gc_from_gce!(
    MakeCylindricalSurface,
    gce::MakeCylinder,
    GeomCylindricalSurface::from_cylinder
);

impl MakeCylindricalSurface {
    /// GC_MakeCylindricalSurface(C): the Geom cylinder of `cylinder`.
    /// `MakeCylindricalSurface::from_cylinder(c)`.
    pub fn from_cylinder(cylinder: Cylinder3) -> Self {
        Self::built(GeomCylindricalSurface::from_cylinder(cylinder))
    }

    /// GC_MakeCylindricalSurface(A2, Radius): gce_NegativeRadius when `radius < 0`.
    /// `MakeCylindricalSurface::new(frame, 1.0)?`.
    pub fn new(frame: Axis2, radius: f64) -> Result<Self, Failure> {
        if radius < 0.0 {
            return Ok(Self::failed(ErrorType::NegativeRadius));
        }
        Ok(Self::built(GeomCylindricalSurface::new(
            Axis3::from_axis2(frame),
            radius,
        )?))
    }

    /// GC_MakeCylindricalSurface(A1, Radius): gce_MakeCylinder(A1, Radius).
    /// `MakeCylindricalSurface::from_axis(axis, 1.0)?`.
    pub fn from_axis(axis: Axis1, radius: f64) -> Result<Self, Failure> {
        Self::from_gce(gce::MakeCylinder::from_axis(axis, radius)?)
    }

    /// GC_MakeCylindricalSurface(Circ): the conversion of gce_MakeCylinder(Circ).
    /// `MakeCylindricalSurface::from_circ(circle)?`.
    pub fn from_circ(circle: Circle3) -> Result<Self, Failure> {
        let cylinder: Cylinder3 = gce::MakeCylinder::from_circ(circle)?.operator()?;
        Ok(Self::built(GeomCylindricalSurface::from_cylinder(cylinder)))
    }

    /// GC_MakeCylindricalSurface(P1, P2, P3): gce_MakeCylinder(P1, P2, P3).
    /// `MakeCylindricalSurface::through_points(a, b, c)?`.
    pub fn through_points(p1: Point3, p2: Point3, p3: Point3) -> Result<Self, Failure> {
        Self::from_gce(gce::MakeCylinder::through_points(p1, p2, p3)?)
    }

    /// GC_MakeCylindricalSurface(Cyl, Dist): `cylinder` with radius
    /// `|Radius - dist|` (gce_MakeCylinder adds the distance); always done.
    /// `MakeCylindricalSurface::from_cylinder_distance(c, 1.0)?`.
    pub fn from_cylinder_distance(cylinder: Cylinder3, dist: f64) -> Result<Self, Failure> {
        let radius: f64 = (cylinder.radius() - dist).abs();
        let surface = GeomCylindricalSurface::from_cylinder(cylinder);
        surface.set_radius(radius)?;
        Ok(Self::built(surface))
    }

    /// GC_MakeCylindricalSurface(Cyl, Point): `cylinder` through `point`; always
    /// done. `MakeCylindricalSurface::from_cylinder_point(c, p)?`.
    pub fn from_cylinder_point(cylinder: Cylinder3, point: Point3) -> Result<Self, Failure> {
        let mut result: Cylinder3 = cylinder;
        let radius: f64 = Line3::from_axis(result.axis()).distance(point);
        result.set_radius(radius)?;
        Ok(Self::built(GeomCylindricalSurface::from_cylinder(result)))
    }
}
