// Rust translation of ModelingData/TKG3d/Adaptor3d/Adaptor3d_IsoCurve.cxx (GetType, Line, Circle, Ellipse, Hyperbola, Parabola, Degree, IsRational, NbPoles, NbKnots, Bezier, BSpline) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The curve kind of `Adaptor3d_IsoCurve` and its geometry, read from the surface
//! kind: lines and circles of the elementary surfaces, the (rotated or translated)
//! basis curve of a swept surface and the iso curves of the spline surfaces.
use super::{IsoState, none_iso};
use crate::adaptor3d::Adaptor3dSurface;
use crate::geom::errors::no_such_object;
use crate::geom::{GeomBSplineCurve, GeomBezierCurve, GeomCurve, GeomSurface};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::elslib;
use rustycad_tkmath::geom_abs::{CurveType, IsoType, SurfaceType};
use rustycad_tkmath::gp::{
    Axis1, Axis2, Axis3, Circle3, Direction3, Ellipse3, Line3, Point3, Vector3,
};

fn not_on(member: &str) -> Failure {
    no_such_object(&format!("Adaptor3d_IsoCurve:{member}"))
}

/// `GetType`; `get_type(&state)?`.
pub(super) fn get_type(state: &IsoState) -> Result<CurveType, Failure> {
    let surface = state.surface()?;
    Ok(match (surface.get_type()?, state.iso) {
        (SurfaceType::Plane, _) => CurveType::Line,
        (SurfaceType::Cylinder | SurfaceType::Cone, IsoType::IsoU) => CurveType::Line,
        (SurfaceType::Cylinder | SurfaceType::Cone, IsoType::IsoV) => CurveType::Circle,
        (SurfaceType::Sphere | SurfaceType::Torus, _) => CurveType::Circle,
        (SurfaceType::BezierSurface, _) => CurveType::BezierCurve,
        (SurfaceType::BSplineSurface, _) => CurveType::BSplineCurve,
        (SurfaceType::SurfaceOfRevolution, IsoType::IsoU) => surface.basis_curve()?.get_type()?,
        (SurfaceType::SurfaceOfRevolution, IsoType::IsoV) => CurveType::Circle,
        (SurfaceType::SurfaceOfExtrusion, IsoType::IsoU) => CurveType::Line,
        (SurfaceType::SurfaceOfExtrusion, IsoType::IsoV) => surface.basis_curve()?.get_type()?,
        (
            SurfaceType::Cylinder
            | SurfaceType::Cone
            | SurfaceType::SurfaceOfRevolution
            | SurfaceType::SurfaceOfExtrusion,
            IsoType::NoneIso,
        ) => return Err(none_iso()),
        _ => CurveType::OtherCurve,
    })
}

/// `Line`: through the point at parameter 0 along its derivative, whatever the
/// kind; `line(point, derivative)?`.
pub(super) fn line((point, derivative): (Point3, Vector3)) -> Result<Line3, Failure> {
    Ok(Line3::new(point, Direction3::from_vector(derivative)?))
}

// computeHR: the height and radius of `point` in `axes`.
fn compute_hr(axes: &Axis3, point: Point3) -> (f64, f64) {
    let vector = Vector3::between(axes.location(), point);
    (
        vector.dot(Vector3::from(axes.direction())),
        vector.dot(Vector3::from(axes.x_direction())),
    )
}

/// `Circle`: the circle iso of an elementary or swept surface; `value0` is the
/// point of parameter 0 (read by the iso-V revolution branch); `circle(&state, p0)?`.
pub(super) fn circle(
    state: &IsoState,
    value0: impl FnOnce() -> Result<Point3, Failure>,
) -> Result<Circle3, Failure> {
    let surface = state.surface()?;
    let p = state.parameter;
    match surface.get_type()? {
        SurfaceType::Cylinder => {
            let cylinder = surface.cylinder()?;
            match state.iso {
                IsoType::IsoU => Err(not_on("UIso")),
                IsoType::IsoV => Ok(elslib::cylinder_v_iso(
                    cylinder.position(),
                    cylinder.radius(),
                    p,
                )),
                IsoType::NoneIso => Err(none_iso()),
            }
        }
        SurfaceType::Cone => {
            let cone = surface.cone()?;
            match state.iso {
                IsoType::IsoU => Err(not_on("UIso")),
                IsoType::IsoV => Ok(elslib::cone_v_iso(
                    cone.position(),
                    cone.ref_radius(),
                    cone.semi_angle(),
                    p,
                )),
                IsoType::NoneIso => Err(none_iso()),
            }
        }
        SurfaceType::Sphere => sphere_circle(surface, state.iso, p),
        SurfaceType::Torus => torus_circle(surface, state.iso, p),
        SurfaceType::SurfaceOfRevolution => revolution_circle(surface, state, value0),
        SurfaceType::SurfaceOfExtrusion => {
            let direction = Vector3::from(surface.direction()?);
            Ok(surface.basis_curve()?.circle()?.translated(direction * p))
        }
        _ => Err(not_on("Circle")),
    }
}

fn sphere_circle(surface: &dyn Adaptor3dSurface, iso: IsoType, p: f64) -> Result<Circle3, Failure> {
    let sphere = surface.sphere()?;
    match iso {
        IsoType::IsoU => Ok(elslib::sphere_u_iso(sphere.position(), sphere.radius(), p)),
        IsoType::IsoV => Ok(elslib::sphere_v_iso(sphere.position(), sphere.radius(), p)),
        IsoType::NoneIso => Err(none_iso()),
    }
}

fn torus_circle(surface: &dyn Adaptor3dSurface, iso: IsoType, p: f64) -> Result<Circle3, Failure> {
    let torus = surface.torus()?;
    let (major, minor) = (torus.major_radius(), torus.minor_radius());
    match iso {
        IsoType::IsoU => Ok(elslib::torus_u_iso(torus.position(), major, minor, p)),
        IsoType::IsoV => Ok(elslib::torus_v_iso(torus.position(), major, minor, p)),
        IsoType::NoneIso => Err(none_iso()),
    }
}

// An iso-V curve of a surface of revolution is the circle of the point at U = 0
// around the axis (of radius 0 on the axis); an iso-U curve the rotated basis.
fn revolution_circle(
    surface: &dyn Adaptor3dSurface,
    state: &IsoState,
    value0: impl FnOnce() -> Result<Point3, Failure>,
) -> Result<Circle3, Failure> {
    if state.iso != IsoType::IsoV {
        let basis: Circle3 = surface.basis_curve()?.circle()?;
        return basis.rotated(surface.axe_of_revolution()?, state.parameter);
    }
    let value: Point3 = value0()?;
    let axis: Axis1 = surface.axe_of_revolution()?;
    if Line3::from_axis(axis).contains(value, precision::confusion()) {
        return Circle3::new(Axis2::new(value, axis.direction())?, 0.0);
    }
    let dx = Vector3::between(axis.location(), value);
    let mut axes = Axis3::new(
        axis.location(),
        axis.direction(),
        Direction3::from_vector(dx)?,
    )?;
    let (h, radius) = compute_hr(&axes, value);
    let vt = Vector3::from(axes.direction());
    axes.translate(vt * h);
    Circle3::new(axes.ax2()?, radius)
}

/// `Ellipse`: the translated basis ellipse of an extrusion; `ellipse(&state)?`.
pub(super) fn ellipse(state: &IsoState) -> Result<Ellipse3, Failure> {
    let surface = state.surface()?;
    match surface.get_type()? {
        SurfaceType::SurfaceOfExtrusion => {
            let direction = Vector3::from(surface.direction()?);
            Ok(surface
                .basis_curve()?
                .ellipse()?
                .translated(direction * state.parameter))
        }
        _ => Err(not_on("Ellipse")),
    }
}

/// `Hyperbola`: always Standard_NoSuchObject; `hyperbola()`.
pub(super) fn hyperbola() -> Failure {
    not_on("Hyperbola")
}
/// `Parabola`: always Standard_NoSuchObject; `parabola()`.
pub(super) fn parabola() -> Failure {
    not_on("Parabola")
}

/// The spline or swept-basis count behind Degree, IsRational, NbPoles and NbKnots:
/// `spline(surface, u_iso)` for Bezier/B-spline surfaces (the V member for an iso-U
/// curve), `basis` on the basis curve of the swept surface of that direction.
pub(super) fn swept_or_spline<T>(
    state: &IsoState,
    spline_kinds: &[SurfaceType],
    spline: impl FnOnce(&dyn Adaptor3dSurface, bool) -> Result<T, Failure>,
    basis: impl FnOnce(&dyn crate::adaptor3d::Adaptor3dCurve) -> Result<T, Failure>,
) -> Result<T, Failure> {
    let surface = state.surface()?;
    let kind: SurfaceType = surface.get_type()?;
    if spline_kinds.contains(&kind) {
        return match state.iso {
            IsoType::IsoU => spline(surface, true),
            IsoType::IsoV => spline(surface, false),
            IsoType::NoneIso => Err(none_iso()),
        };
    }
    match (kind, state.iso) {
        (SurfaceType::SurfaceOfRevolution, IsoType::IsoU)
        | (SurfaceType::SurfaceOfExtrusion, IsoType::IsoV) => basis(&*surface.basis_curve()?),
        _ => Err(none_iso()),
    }
}

/// `Bezier`: the rotated or translated copy of a swept basis, else the Bezier iso of
/// the surface (null when that iso is not a Bezier curve); `bezier(&state)?`.
pub(super) fn bezier(state: &IsoState) -> Result<Handle<GeomBezierCurve>, Failure> {
    let surface = state.surface()?;
    let curve: Handle<dyn GeomCurve> = match surface.get_type()? {
        SurfaceType::SurfaceOfRevolution | SurfaceType::SurfaceOfExtrusion => swept_copy(
            surface,
            state.parameter,
            surface.basis_curve()?.bezier()?.cast(),
        )?,
        _ if state.iso == IsoType::IsoU => surface.bezier()?.u_iso(state.parameter)?,
        _ => surface.bezier()?.v_iso(state.parameter)?,
    };
    Ok(curve.cast())
}

/// `BSpline`, as `bezier`; `bspline(&state)?`.
pub(super) fn bspline(state: &IsoState) -> Result<Handle<GeomBSplineCurve>, Failure> {
    let surface = state.surface()?;
    let curve: Handle<dyn GeomCurve> = match surface.get_type()? {
        SurfaceType::SurfaceOfRevolution | SurfaceType::SurfaceOfExtrusion => swept_copy(
            surface,
            state.parameter,
            surface.basis_curve()?.bspline()?.cast(),
        )?,
        _ if state.iso == IsoType::IsoU => surface.bspline()?.u_iso(state.parameter)?,
        _ => surface.bspline()?.v_iso(state.parameter)?,
    };
    Ok(curve.cast())
}

// A copy of the basis spline rotated about the axis (revolution) or translated along
// the direction (extrusion) by the fixed parameter.
fn swept_copy(
    surface: &dyn Adaptor3dSurface,
    parameter: f64,
    basis: Handle<dyn GeomCurve>,
) -> Result<Handle<dyn GeomCurve>, Failure> {
    // Natively a null basis spline is dereferenced.
    let Some(source) = basis.get() else {
        return Err(no_such_object(
            "Adaptor3d_IsoCurve: the swept basis is not a spline",
        ));
    };
    let copy: Handle<dyn GeomCurve> = source.copy()?.cast();
    if surface.get_type()? == SurfaceType::SurfaceOfRevolution {
        copy.rotate(surface.axe_of_revolution()?, parameter)?;
    } else {
        copy.translate(Vector3::from(surface.direction()?) * parameter)?;
    }
    Ok(copy)
}
