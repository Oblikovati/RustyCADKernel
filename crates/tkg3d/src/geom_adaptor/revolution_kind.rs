// Rust translation of ModelingData/TKG3d/GeomAdaptor/GeomAdaptor_SurfaceOfRevolution.cxx (Load(gp_Ax1), GetType, Plane, Cylinder, Cone, Sphere, Torus) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The canonical kinds of a surface of revolution and the frame they are built on
//! (`myAxeRev`): a cylinder, plane or cone for a revolved line, a sphere or torus
//! for a coplanar circle.
use super::surface::GeomAdaptorSurface;
use crate::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use crate::geom::errors::construction;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::{
    Axis1, Axis3, Cone3, Cylinder3, Direction3, Line3, Plane3, Point3, Sphere3, Torus3, Vector3,
};

/// What the kind computations read: the base adaptor (for `Value(0, 0)`), the
/// basis curve, the axis and the current auxiliary frame.
pub(super) struct Revolved<'a> {
    pub(super) base: &'a GeomAdaptorSurface,
    pub(super) curve: &'a dyn Adaptor3dCurve,
    pub(super) axis: Axis1,
    pub(super) axe_rev: Axis3,
}
impl Revolved<'_> {
    fn origin_point(&self) -> Result<Point3, Failure> {
        self.base.value(0., 0.)
    }
    // gp_Vec(myAxeRev.Location(), P) * myAxeRev.XDirection().
    fn radial_distance(&self, point: Point3) -> f64 {
        Vector3::between(self.axe_rev.location(), point)
            .dot(Vector3::from(self.axe_rev.x_direction()))
    }
    fn axis_line(&self) -> Line3 {
        Line3::from(self.axis)
    }
}

/// `GetType` of `GeomAdaptor_SurfaceOfRevolution`; `revolution_type(&revolved)?`.
pub(super) fn revolution_type(revolved: &Revolved<'_>) -> Result<SurfaceType, Failure> {
    match revolved.curve.get_type()? {
        CurveType::Line => line_type(revolved),
        CurveType::Circle => circle_type(revolved),
        _ => Ok(SurfaceType::SurfaceOfRevolution),
    }
}

const TOL_CONF: f64 = precision::confusion();
const TOL_CONE_SEMI_ANG: f64 = precision::confusion();

fn cylinder_if_off_axis(revolved: &Revolved<'_>) -> Result<Option<SurfaceType>, Failure> {
    let radius: f64 = revolved.radial_distance(revolved.origin_point()?);
    Ok((radius > TOL_CONF).then_some(SurfaceType::Cylinder))
}

fn line_type(revolved: &Revolved<'_>) -> Result<SurfaceType, Failure> {
    let tol_ang: f64 = precision::angular();
    let line_axis: Axis1 = revolved.curve.line()?.position();
    if revolved.axis.is_parallel(line_axis, tol_ang) {
        return Ok(cylinder_if_off_axis(revolved)?.unwrap_or(SurfaceType::SurfaceOfRevolution));
    }
    if revolved.axis.is_normal(line_axis, tol_ang) {
        return Ok(SurfaceType::Plane);
    }
    if let Some(kind) = trimmed_line_type(revolved)? {
        return Ok(kind);
    }
    cone_or_revolution(revolved, line_axis)
}

// A bounded line whose chord is along the axis (cylinder) or across it (plane).
fn trimmed_line_type(revolved: &Revolved<'_>) -> Result<Option<SurfaceType>, Failure> {
    let (uf, ul) = (
        revolved.curve.first_parameter()?,
        revolved.curve.last_parameter()?,
    );
    if precision::is_infinite(uf) || precision::is_infinite(ul) {
        return Ok(None);
    }
    let (pf, pl) = (revolved.curve.value(uf)?, revolved.curve.value(ul)?);
    let length: f64 = pf.distance(&pl);
    // Compute the distance projected onto the axis.
    let chord = Vector3::between(pf, pl);
    let axis_vector = Vector3::from(revolved.axis.direction());
    let projected: f64 = axis_vector.dot(chord).abs();
    if length - projected <= TOL_CONF {
        return cylinder_if_off_axis(revolved);
    }
    Ok((projected <= TOL_CONF).then_some(SurfaceType::Plane))
}

fn cone_or_revolution(revolved: &Revolved<'_>, line_axis: Axis1) -> Result<SurfaceType, Failure> {
    let to_line = Vector3::between(revolved.axis.location(), revolved.curve.line()?.location());
    let line_dir = Vector3::from(line_axis.direction());
    let axis_dir = Vector3::from(revolved.axis.direction());
    let projection: f64 = line_dir.dot(axis_dir).abs();
    let in_plane: bool = to_line.dot_cross(axis_dir, line_dir).abs() <= TOL_CONF;
    if in_plane && (TOL_CONE_SEMI_ANG..=1. - TOL_CONE_SEMI_ANG).contains(&projection) {
        return Ok(SurfaceType::Cone);
    }
    Ok(SurfaceType::SurfaceOfRevolution)
}

fn circle_type(revolved: &Revolved<'_>) -> Result<SurfaceType, Failure> {
    let circle = revolved.curve.circle()?;
    let center: Point3 = circle.location();
    let radius: f64 = circle.radius();
    if !circle
        .position()
        .is_coplanar_axis(revolved.axis, TOL_CONF, precision::angular())
    {
        return Ok(SurfaceType::SurfaceOfRevolution);
    }
    let major_radius: f64 = revolved.axis_line().distance(center);
    if major_radius <= TOL_CONF {
        return Ok(SurfaceType::Sphere);
    }
    if major_radius > radius {
        return Ok(SurfaceType::Torus);
    }
    Ok(SurfaceType::SurfaceOfRevolution)
}

/// `Plane`: the auxiliary frame moved to the projection of `Value(0, 0)` on the axis,
/// its X direction against the line; `revolution_plane(&revolved)?`.
pub(super) fn revolution_plane(revolved: &Revolved<'_>) -> Result<Plane3, Failure> {
    let mut frame: Axis3 = revolved.axe_rev;
    let on_curve: Point3 = revolved.origin_point()?;
    let (location, direction) = (
        revolved.axis.location().xyz(),
        revolved.axis.direction().xyz(),
    );
    let dot: f64 = (on_curve.xyz() - location).dot(direction);
    frame.set_location(Point3::from(location + dot * direction));
    let line_direction: Direction3 = revolved.curve.line()?.direction();
    if frame.x_direction().dot(line_direction) >= -precision::confusion() {
        frame.x_reverse();
    }
    Ok(Plane3::from(frame))
}

/// `Cylinder`: the auxiliary frame and the radial distance of `Value(0, 0)`;
/// `revolution_cylinder(&revolved)?`.
pub(super) fn revolution_cylinder(revolved: &Revolved<'_>) -> Result<Cylinder3, Failure> {
    let radius: f64 = revolved.radial_distance(revolved.origin_point()?);
    Cylinder3::new(revolved.axe_rev, radius)
}

/// `Cone`: the auxiliary frame, the angle between the axis and the line (negated
/// when the apex side requires it) and the distance of `Value(0, 0)` to the frame
/// origin; `revolution_cone(&revolved)?`.
pub(super) fn revolution_cone(revolved: &Revolved<'_>) -> Result<Cone3, Failure> {
    let frame: Axis3 = revolved.axe_rev;
    let line_direction: Direction3 = revolved.curve.line()?.direction();
    let mut angle: f64 = frame.direction().angle(&line_direction);
    let p0: Point3 = revolved.origin_point()?;
    let radius: f64 = frame.location().distance(&p0);
    if radius >= precision::confusion() {
        let mut op0 = Vector3::between(frame.location(), p0);
        let x_direction = Vector3::from(frame.x_direction());
        let mut t: f64 = op0.dot(x_direction);
        t /= Vector3::from(line_direction).dot(x_direction);
        op0 += Vector3::from(line_direction) * -t;
        if op0.dot(Vector3::from(frame.direction())) > 0. {
            angle = -angle;
        }
    }
    Cone3::new(frame, angle, radius)
}

/// `Sphere`: the auxiliary frame at the circle center, of the circle radius;
/// `revolution_sphere(&revolved)?`.
pub(super) fn revolution_sphere(revolved: &Revolved<'_>) -> Result<Sphere3, Failure> {
    let circle = revolved.curve.circle()?;
    let mut frame: Axis3 = revolved.axe_rev;
    frame.set_location(circle.location());
    Sphere3::new(frame, circle.radius())
}

/// `Torus`: the auxiliary frame, the distance of the circle center to the axis and
/// the circle radius; `revolution_torus(&revolved)?`.
pub(super) fn revolution_torus(revolved: &Revolved<'_>) -> Result<Torus3, Failure> {
    let circle = revolved.curve.circle()?;
    let major_radius: f64 = revolved.axis_line().distance(circle.location());
    Torus3::new(revolved.axe_rev, major_radius, circle.radius())
}

/// The auxiliary frame computed by `Load(gp_Ax1)` (`myAxeRev`): origin on the axis
/// level with `Value(0, 0)`, main direction the axis (reversed against a line going
/// the other way), X direction towards the curve; Standard_ConstructionError when the
/// curve lies on the axis; `axe_of_revolution(&revolved)?`.
pub(super) fn axe_of_revolution(revolved: &Revolved<'_>) -> Result<Axis3, Failure> {
    let mut oz: Direction3 = revolved.axis.direction();
    let curve_type: CurveType = revolved.curve.get_type()?;
    let mut y_reversed: bool = false;
    if curve_type == CurveType::Line && revolved.curve.line()?.direction().dot(oz) < 0. {
        y_reversed = true;
        oz.reverse();
    }
    let (p, q) = reference_points(revolved, curve_type)?;
    let dz: Direction3 = revolved.axis.direction();
    let mut origin: Point3 = revolved.axis.location();
    let height: f64 = Vector3::between(origin, p).dot(Vector3::from(dz));
    origin.set_xyz(origin.xyz() + height * dz.xyz());
    let ox: Direction3 = if revolved.axis_line().distance(q) > precision::confusion() {
        Direction3::from_xyz(q.xyz() - origin.xyz())?
    } else {
        off_axis_direction(revolved, origin, oz)?
    };
    let mut frame = Axis3::new(origin, oz, ox)?;
    if y_reversed {
        frame.y_reverse();
    } else if curve_type == CurveType::Circle {
        let circle_direction: Direction3 = revolved.curve.circle()?.axis().direction();
        if ox.crossed(oz)?.dot(circle_direction) < 0. {
            frame.z_reverse();
        }
    }
    Ok(frame)
}

// The points P and Q of Load: the circle center twice, else Value(0, 0) and a point
// of the first curve parameter (or of the line at parameter 1 for a cone through
// the apex). GetType() here reads the frame as it was before this Load.
fn reference_points(
    revolved: &Revolved<'_>,
    curve_type: CurveType,
) -> Result<(Point3, Point3), Failure> {
    if curve_type == CurveType::Circle {
        let center: Point3 = revolved.curve.circle()?.location();
        return Ok((center, center));
    }
    let first: f64 = revolved.curve.first_parameter()?;
    let p: Point3 = revolved.origin_point()?;
    let q: Point3 = if revolution_type(revolved)? == SurfaceType::Cone {
        if revolved.axis_line().distance(p) <= precision::confusion() {
            elclib::value(1., &revolved.curve.line()?)
        } else {
            p
        }
    } else if precision::is_infinite(first) {
        p
    } else {
        revolved.base.value(0., first)?
    };
    Ok((p, q))
}

// A point of the curve off the axis (at most 99 tries along the range), then the
// direction from the axis towards it: (Oz ^ (PP - O)) ^ Oz.
fn off_axis_direction(
    revolved: &Revolved<'_>,
    origin: Point3,
    oz: Direction3,
) -> Result<Direction3, Failure> {
    let (first, last) = (
        revolved.curve.first_parameter()?,
        revolved.curve.last_parameter()?,
    );
    let mut ratio: i32 = 1;
    let mut point: Point3;
    loop {
        point = revolved
            .curve
            .value(first + (last - first) / f64::from(ratio))?;
        let distance: f64 = revolved.axis_line().distance(point);
        ratio += 1;
        if !(distance < precision::confusion() && ratio < 100) {
            break;
        }
    }
    if ratio >= 100 {
        return Err(construction(
            "Adaptor3d_SurfaceOfRevolution : Axe and meridian are confused",
        ));
    }
    let radial: Direction3 = Direction3::from_xyz(point.xyz() - origin.xyz())?;
    oz.crossed(radial)?.crossed(oz)
}
