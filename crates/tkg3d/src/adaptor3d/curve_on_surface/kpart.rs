// Rust translation of ModelingData/TKG3d/Adaptor3d/Adaptor3d_CurveOnSurface.cxx (EvalKPart and the file-local to3d functions) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `EvalKPart`: the curve kind of a curve on a surface. On a plane it is the kind of
//! the 2D curve (lines and circles mapped to 3D); on the other elementary surfaces a
//! 2D line along an iso direction is a circle or a line of the surface.
use super::{Adaptor3dCurveOnSurface, CosState};
use crate::adaptor3d::Adaptor3dSurface;
use crate::geom::GeomSurfaceD1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkmath::elslib;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::{
    Axis1, Axis2, Axis3, Axis22d, Circle2, Circle3, Direction2, Direction3, Ellipse2, Ellipse3,
    Hyperbola2, Hyperbola3, Line3, Parabola2, Parabola3, Plane3, Point2, Point3,
    StandardDirection2, Vector2, Vector3,
};

/// `to3d(gp_Pln, gp_Pnt2d)`: ElSLib::Value on the plane; `to3d_point(&plane, p)`.
pub(super) fn to3d_point(plane: &Plane3, point: Point2) -> Point3 {
    elslib::plane_value(point.x(), point.y(), plane.position())
}

// to3d(gp_Pln, gp_Vec2d): Vx * V.X() + Vy * V.Y() (Multiply, then Add).
fn to3d_vector(plane: &Plane3, (x, y): (f64, f64)) -> Vector3 {
    let vx = Vector3::from(plane.x_axis().direction()) * x;
    let vy = Vector3::from(plane.y_axis().direction()) * y;
    vx + vy
}

// to3d(gp_Pln, gp_Ax22d): the frame at the mapped location with the mapped X
// direction and the normal X ^ Y.
fn to3d_frame(plane: &Plane3, axis: Axis22d) -> Result<Axis2, Failure> {
    let point = to3d_point(plane, axis.location());
    let (x, y): (Direction2, Direction2) = (axis.x_direction(), axis.y_direction());
    let vx = to3d_vector(plane, (x.x(), x.y()));
    let vy = to3d_vector(plane, (y.x(), y.y()));
    Axis2::with_x_direction(
        point,
        Direction3::from_vector(vx.crossed(vy))?,
        Direction3::from_vector(vx)?,
    )
}

/// `to3d(gp_Pln, gp_Circ2d)`; `to3d_circle(&plane, circle)?`.
pub(super) fn to3d_circle(plane: &Plane3, circle: Circle2) -> Result<Circle3, Failure> {
    Circle3::new(to3d_frame(plane, circle.axis())?, circle.radius())
}
/// `to3d(gp_Pln, gp_Elips2d)`; `to3d_ellipse(&plane, ellipse)?`.
pub(super) fn to3d_ellipse(plane: &Plane3, ellipse: Ellipse2) -> Result<Ellipse3, Failure> {
    Ellipse3::new(
        to3d_frame(plane, ellipse.axis())?,
        ellipse.major_radius(),
        ellipse.minor_radius(),
    )
}
/// `to3d(gp_Pln, gp_Hypr2d)`; `to3d_hyperbola(&plane, hyperbola)?`.
pub(super) fn to3d_hyperbola(plane: &Plane3, hyperbola: Hyperbola2) -> Result<Hyperbola3, Failure> {
    Hyperbola3::new(
        to3d_frame(plane, hyperbola.axis())?,
        hyperbola.major_radius(),
        hyperbola.minor_radius(),
    )
}
/// `to3d(gp_Pln, gp_Parab2d)`; `to3d_parabola(&plane, parabola)?`.
pub(super) fn to3d_parabola(plane: &Plane3, parabola: Parabola2) -> Result<Parabola3, Failure> {
    Parabola3::new(to3d_frame(plane, parabola.axis())?, parabola.focal())
}

/// The special kind with its line or circle (the stored ones when unchanged).
struct KPart {
    kind: CurveType,
    circle: Circle3,
    line: Line3,
}

impl Adaptor3dCurveOnSurface {
    /// `EvalKPart`: derive the curve kind and its line or circle from the curve and
    /// surface kinds; `self.eval_k_part()?`.
    pub(super) fn eval_k_part(&self) -> Result<(), Failure> {
        let state: CosState = self.snapshot();
        self.state.write().kind = CurveType::OtherCurve;
        let mut part = KPart {
            kind: CurveType::OtherCurve,
            circle: state.circle,
            line: state.line,
        };
        k_part(&state, &mut part)?;
        let mut stored = self.state.write();
        (stored.kind, stored.circle, stored.line) = (part.kind, part.circle, part.line);
        Ok(())
    }
}

fn k_part(state: &CosState, part: &mut KPart) -> Result<(), Failure> {
    let (surface, curve) = (state.surface()?, state.curve()?);
    let surface_kind: SurfaceType = surface.get_type()?;
    let curve_kind: CurveType = curve.get_type()?;
    if surface_kind == SurfaceType::Plane {
        return plane_part(surface, curve, curve_kind, part);
    }
    if curve_kind != CurveType::Line {
        return Ok(());
    }
    let direction: Direction2 = curve.line()?.direction();
    let dx = Direction2::from_standard(StandardDirection2::X);
    let dy = Direction2::from_standard(StandardDirection2::Y);
    if direction.is_parallel(dx, precision::angular()) {
        let location: Point2 = curve.line()?.location();
        let opposite: bool = direction.is_opposite(dx, precision::angular());
        return v_iso_part(surface, surface_kind, (location, opposite), part);
    }
    if direction.is_parallel(dy, precision::angular()) {
        let location: Point2 = curve.line()?.location();
        let opposite: bool = direction.is_opposite(dy, precision::angular());
        return u_iso_part(surface, surface_kind, (location, opposite), part);
    }
    Ok(())
}

// On a plane: the 2D kind; a circle mapped to 3D, a line through the point of
// parameter 0 along the mapped tangent.
fn plane_part(
    surface: &dyn Adaptor3dSurface,
    curve: &dyn Adaptor2dCurve2d,
    curve_kind: CurveType,
    part: &mut KPart,
) -> Result<(), Failure> {
    part.kind = curve_kind;
    if curve_kind == CurveType::Circle {
        part.circle = to3d_circle(&surface.plane()?, curve.circle()?)?;
    } else if curve_kind == CurveType::Line {
        let (puv, duv): (Point2, Vector2) = curve.d1(0.)?;
        let d1: GeomSurfaceD1 = surface.d1(puv.x(), puv.y())?;
        let mut v = Vector3::default();
        v.set_linear_form(duv.x(), d1.d1u, duv.y(), d1.d1v);
        part.line = Line3::new(d1.point, Direction3::from_vector(v)?);
    }
    Ok(())
}

// Reverse the circle's main direction when the 2D line runs backwards.
fn reversed_if(mut circle: Circle3, opposite: bool) -> Result<Circle3, Failure> {
    if opposite {
        let mut frame: Axis2 = circle.position();
        frame.set_direction(frame.direction().reversed())?;
        circle.set_position(frame);
    }
    Ok(circle)
}

// Rotate `circle` about the axis of `frame` (X ^ Y at its origin) by `angle`.
fn rotated_about_z(circle: Circle3, frame: &Axis3, angle: f64) -> Result<Circle3, Failure> {
    let rev = frame.x_direction().crossed(frame.y_direction());
    circle.rotated(Axis1::new(frame.location(), rev?), angle)
}

// A 2D line along U (iso V = P.Y) on a sphere (away from the poles), cylinder, cone
// or torus: the V iso circle rotated by P.X.
fn v_iso_part(
    surface: &dyn Adaptor3dSurface,
    kind: SurfaceType,
    (p, opposite): (Point2, bool),
    part: &mut KPart,
) -> Result<(), Failure> {
    let (circle, frame): (Circle3, Axis3) = match kind {
        SurfaceType::Sphere => {
            if (p.y().abs() - std::f64::consts::FRAC_PI_2).abs() < precision::p_confusion() {
                return Ok(());
            }
            let sphere = surface.sphere()?;
            let frame = sphere.position();
            (elslib::sphere_v_iso(frame, sphere.radius(), p.y()), frame)
        }
        SurfaceType::Cylinder => {
            let cylinder = surface.cylinder()?;
            let frame = cylinder.position();
            (
                elslib::cylinder_v_iso(frame, cylinder.radius(), p.y()),
                frame,
            )
        }
        SurfaceType::Cone => {
            let cone = surface.cone()?;
            let frame = cone.position();
            (
                elslib::cone_v_iso(frame, cone.ref_radius(), cone.semi_angle(), p.y()),
                frame,
            )
        }
        SurfaceType::Torus => {
            let torus = surface.torus()?;
            let frame = torus.position();
            (
                elslib::torus_v_iso(frame, torus.major_radius(), torus.minor_radius(), p.y()),
                frame,
            )
        }
        _ => return Ok(()),
    };
    part.kind = CurveType::Circle;
    part.circle = reversed_if(rotated_about_z(circle, &frame, p.x())?, opposite)?;
    Ok(())
}

// A 2D line along V (iso U = P.X): a meridian circle of a sphere or torus, a ruling
// line of a cylinder or cone, shifted along itself by P.Y.
fn u_iso_part(
    surface: &dyn Adaptor3dSurface,
    kind: SurfaceType,
    (p, opposite): (Point2, bool),
    part: &mut KPart,
) -> Result<(), Failure> {
    match kind {
        SurfaceType::Sphere => {
            let sphere = surface.sphere()?;
            let frame = sphere.position();
            // Compute the iso 0, rotate it in its plane by P.Y, then about Z by P.X.
            let circle = elslib::sphere_u_iso(frame, sphere.radius(), 0.);
            let rev = frame.x_direction().crossed(frame.direction());
            let circle = circle.rotated(Axis1::new(frame.location(), rev?), p.y())?;
            let circle = rotated_about_z(circle, &frame, p.x())?;
            part.circle = reversed_if(circle, opposite)?;
            part.kind = CurveType::Circle;
        }
        SurfaceType::Cylinder => {
            let cylinder = surface.cylinder()?;
            let line = elslib::cylinder_u_iso(cylinder.position(), cylinder.radius(), p.x());
            part.line = shifted_line(line, p.y(), opposite);
            part.kind = CurveType::Line;
        }
        SurfaceType::Cone => {
            let cone = surface.cone()?;
            let line =
                elslib::cone_u_iso(cone.position(), cone.ref_radius(), cone.semi_angle(), p.x());
            part.line = shifted_line(line, p.y(), opposite);
            part.kind = CurveType::Line;
        }
        SurfaceType::Torus => {
            let torus = surface.torus()?;
            let circle = elslib::torus_u_iso(
                torus.position(),
                torus.major_radius(),
                torus.minor_radius(),
                p.x(),
            );
            let circle = circle.rotated(circle.axis(), p.y())?;
            part.circle = reversed_if(circle, opposite)?;
            part.kind = CurveType::Circle;
        }
        _ => {}
    }
    Ok(())
}

// Translate a ruling by `shift` along its direction, reversed for a backward 2D line.
fn shifted_line(mut line: Line3, shift: f64, opposite: bool) -> Line3 {
    let mut translation = Vector3::from(line.direction());
    translation *= shift;
    line.translate(translation);
    if opposite {
        line.reverse();
    }
    line
}
