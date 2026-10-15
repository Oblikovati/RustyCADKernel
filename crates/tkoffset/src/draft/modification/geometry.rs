// Rust translation of Draft_Modification::NewSurface and Draft_Modification::NewCurve (the
// private overloads) and the statics Orientation and FindRotation of
// ModelingAlgorithms/TKOffset/Draft/Draft_Modification_1.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The drafted surfaces and edge curves of a face.
use super::live;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Face, Shape};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{
    GeomConicalSurface, GeomCurve, GeomCylindricalSurface, GeomLine, GeomPlane, GeomSurface,
};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeombase::int_ana::{IntConicQuad, QuadQuadGeo, ResultType};
use rustycad_tkmath::elclib;
use rustycad_tkmath::elslib;
use rustycad_tkmath::gp::{Axis1, Cone3, Cylinder3, Direction3, Line3, Plane3};
use std::f64::consts::PI;

/// `Orientation(S, F)`: the orientation of the face in the shape (FORWARD when it is
/// not found).
pub(super) fn orientation(s: &Shape, f: &Face) -> Result<Orientation, Failure> {
    for current in Explorer::new(s, ShapeEnum::Face, ShapeEnum::Shape)? {
        if current.is_same(f) {
            return Ok(current.orientation());
        }
    }
    Ok(Orientation::Forward)
}

/// `FindRotation(Pl, Oris, Direction, Angle, NeutralPlane, Axe, theta)`: the rotation
/// about the intersection of the plane and the neutral plane that drafts the plane by
/// the angle, `None` when the planes are parallel, the direction lies along the line, or
/// the angle cannot be reached.
fn find_rotation(
    pl: &Plane3,
    oris: Orientation,
    direction: Direction3,
    angle: f64,
    neutral_plane: &Plane3,
) -> Result<Option<(Axis1, f64)>, Failure> {
    let i2pl = QuadQuadGeo::planes(*pl, *neutral_plane, precision::angular(), precision::confusion())?;

    if i2pl.is_done() && i2pl.type_inter()? == ResultType::Line {
        let li = i2pl.line(1)?;
        // Try to turn around this line
        let nx = li.direction();
        let ny = pl.axis().direction().crossed(nx)?;
        let a = direction.dot(nx);
        if a.abs() <= 1. - precision::angular() {
            let mut b = direction.dot(ny);
            let mut c = direction.dot(pl.axis().direction());
            let direct = pl.direct()?;
            if (direct && oris == Orientation::Reversed) || (!direct && oris == Orientation::Forward) {
                b = -b;
                c = -c;
            }
            let denom = (1. - a * a).sqrt();
            let sina = angle.sin();
            if denom > sina.abs() {
                let phi = (b / denom).atan2(c / denom);
                let theta0 = (sina / denom).acos();
                let mut theta = theta0 - phi;
                if theta.cos() < 0. {
                    theta = -theta0 - phi;
                }
                while theta.abs() > PI {
                    theta += PI * if theta < 0. { 1. } else { -1. };
                }
                return Ok(Some((li.position(), theta)));
            }
        }
    }
    Ok(None)
}

/// The rotated copy of a surface (`S->Rotated(Axe, Theta)`).
fn rotated_surface(s: &Handle<dyn GeomSurface>, axe: Axis1, theta: f64) -> Result<Handle<dyn GeomSurface>, Failure> {
    Ok(live(s, "surface")?.rotated(axe, theta)?.cast())
}

/// `NewSurface(S, Oris, Direction, Angle, NeutralPlane)`: the drafted surface (the
/// surface itself for a null rotation or angle); null for an unsupported surface or
/// configuration.
pub(super) fn new_surface(
    s: &Handle<dyn GeomSurface>,
    oris: Orientation,
    direction: Direction3,
    angle: f64,
    neutral_plane: &Plane3,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    let mut new_s: Handle<dyn GeomSurface> = Handle::default();
    let surface = live(s, "surface")?;

    if surface.is_instance_name("Geom_Plane") {
        let pl = live(&s.clone().cast::<GeomPlane>(), "plane")?.pln();
        if let Some((axe, theta)) = find_rotation(&pl, oris, direction, angle, neutral_plane)? {
            if theta.abs() > precision::angular() {
                new_s = rotated_surface(s, axe, theta)?;
            } else {
                new_s = s.clone();
            }
        }
    } else if surface.is_instance_name("Geom_CylindricalSurface") {
        let mut testdir = direction.dot(neutral_plane.axis().direction());
        if testdir.abs() <= 1. - precision::angular() {
            return Ok(new_s);
        }
        let cy: Cylinder3 = live(&s.clone().cast::<GeomCylindricalSurface>(), "cylinder")?.cylinder()?;
        testdir = direction.dot(cy.axis().direction());
        if testdir.abs() <= 1. - precision::angular() {
            return Ok(new_s);
        }
        if angle.abs() > precision::angular() {
            let i2s = QuadQuadGeo::plane_cylinder(
                *neutral_plane,
                cy,
                precision::angular(),
                precision::confusion(),
                0.0,
            )?;
            let mut is_int_done = i2s.is_done();

            if i2s.type_inter()? == ResultType::Ellipse {
                let an_el = i2s.ellipse(1)?;
                let a_major_r = an_el.major_radius();
                let a_minor_r = an_el.minor_radius();
                is_int_done = a_major_r < 100000.0 * a_minor_r;
            }

            if !is_int_done || i2s.type_inter()? != ResultType::Circle {
                return Ok(new_s);
            }
            let axcone = cy.position();
            // Pb : Where is the material???
            let mut alpha = angle;
            let direct = axcone.direct()?;
            if (direct && oris == Orientation::Reversed) || (!direct && oris == Orientation::Forward) {
                alpha = -alpha;
            }

            let center = i2s.circle(1)?.location();
            if testdir < 0. {
                alpha = -alpha;
            }
            let z = elclib::line_parameter(cy.axis(), center);
            let mut rad = cy.radius() + z * alpha.tan();
            if rad < 0. {
                rad = -rad;
            } else {
                alpha = -alpha;
            }
            let co = Cone3::new(axcone, alpha, rad)?;
            new_s = Handle::new(GeomConicalSurface::from_cone(co)).cast();
        } else {
            new_s = s.clone();
        }
    } else if surface.is_instance_name("Geom_ConicalSurface") {
        let mut testdir = direction.dot(neutral_plane.axis().direction());
        if testdir.abs() <= 1. - precision::angular() {
            return Ok(new_s);
        }

        let co1: Cone3 = live(&s.clone().cast::<GeomConicalSurface>(), "cone")?.cone()?;

        testdir = direction.dot(co1.axis().direction());
        if testdir.abs() <= 1. - precision::angular() {
            return Ok(new_s);
        }

        let i2s = QuadQuadGeo::plane_cone(*neutral_plane, co1, precision::angular(), precision::confusion())?;
        if !i2s.is_done() || i2s.type_inter()? != ResultType::Circle {
            return Ok(new_s);
        }
        let axcone = co1.position();
        // Pb : Where is the material???
        let mut alpha = angle;
        let direct = axcone.direct()?;
        if (direct && oris == Orientation::Reversed) || (!direct && oris == Orientation::Forward) {
            alpha = -alpha;
        }

        let center = i2s.circle(1)?.location();
        if angle.abs() > precision::angular() {
            if testdir < 0. {
                alpha = -alpha;
            }
            let z = elclib::line_parameter(co1.axis(), center);
            let mut rad = i2s.circle(1)?.radius() + z * alpha.tan();
            if rad < 0. {
                rad = -rad;
            } else {
                alpha = -alpha;
            }
            if (alpha - co1.semi_angle()).abs() < precision::angular() {
                new_s = s.clone();
            } else {
                let co = Cone3::new(axcone, alpha, rad)?;
                new_s = Handle::new(GeomConicalSurface::from_cone(co)).cast();
            }
        } else {
            new_s = Handle::new(GeomCylindricalSurface::from_cylinder(Cylinder3::new(
                axcone,
                i2s.circle(1)?.radius(),
            )?))
            .cast();
        }
    }
    Ok(new_s)
}

/// `NewCurve(C, S, Oris, Direction, Angle, NeutralPlane, Flag)`: the drafted curve of an
/// edge of the face: rotated with a plane, else the line through the neutral point
/// rotated about the surface normal; null otherwise.
pub(super) fn new_curve(
    c: &Handle<dyn GeomCurve>,
    s: &Handle<dyn GeomSurface>,
    oris: Orientation,
    direction: Direction3,
    angle: f64,
    neutral_plane: &Plane3,
) -> Result<Handle<dyn GeomCurve>, Failure> {
    let mut new_c: Handle<dyn GeomCurve> = Handle::default();
    let surface = live(s, "surface")?;

    if surface.is_instance_name("Geom_Plane") {
        let pl = live(&s.clone().cast::<GeomPlane>(), "plane")?.pln();
        if let Some((axe, theta)) = find_rotation(&pl, oris, direction, angle, neutral_plane)? {
            if theta.abs() > precision::angular() {
                new_c = live(c, "curve")?.rotated(axe, theta)?.cast();
            } else {
                new_c = c.clone();
            }
        }
        return Ok(new_c);
    }

    if !live(c, "curve")?.is_instance_name("Geom_Line") {
        return Ok(new_c);
    }

    let lin: Line3 = live(&c.clone().cast::<GeomLine>(), "line")?.lin();
    // The native normal stays the default direction on other surfaces.
    let mut norm = Direction3::default();
    if surface.is_instance_name("Geom_CylindricalSurface") {
        let cy = live(&s.clone().cast::<GeomCylindricalSurface>(), "cylinder")?.cylinder()?;
        let (u, v) = elslib::cylinder_parameters(cy.position(), cy.radius(), lin.location());
        let (_, d1u, d1v) = elslib::cylinder_d1(u, v, cy.position(), cy.radius());
        norm = Direction3::from_vector(d1u.crossed(d1v))?;
    } else if surface.is_instance_name("Geom_ConicalSurface") {
        let co = live(&s.clone().cast::<GeomConicalSurface>(), "cone")?.cone()?;
        let (u, v) = elslib::cone_parameters(co.position(), co.ref_radius(), co.semi_angle(), lin.location());
        let (_, d1u, d1v) = elslib::cone_d1(u, v, co.position(), co.ref_radius(), co.semi_angle());
        norm = Direction3::from_vector(d1u.crossed(d1v))?;
    }

    let ilipl = IntConicQuad::line_plane(lin, *neutral_plane, precision::angular(), 0., 0.)?;
    if ilipl.is_done() && ilipl.nb_points()? != 0 {
        if oris == Orientation::Reversed {
            norm.reverse();
        }
        let axrot = Axis1::new(ilipl.point(1)?, norm.crossed(direction)?);
        let mut lires = Line3::from(Axis1::new(ilipl.point(1)?, direction)).rotated(axrot, angle)?;
        if lires.direction().dot(lin.direction()) < 0. {
            lires.reverse();
        }
        new_c = Handle::new(GeomLine::from_lin(lires)).cast();
    }
    Ok(new_c)
}
