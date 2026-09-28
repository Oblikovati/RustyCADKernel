// Copyright (c) 2025 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of findExactPolePlane, isParallel, projectBasisCurve, ProjectBezier,
// ProjectBSpline and ProjectOffset of ModelingData/TKGeomBase/ExtremaPC/ExtremaPC_Planar.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The exact, parameter-preserving projections of planar spline and offset curves.
//! A control polygon is planar only when every pole's triple product with two
//! spanning vectors is exactly zero, computed in `long double` as natively.
use crate::plane_projection as projlib;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCircle, Geom2dCurve, Geom2dEllipse,
    Geom2dHyperbola, Geom2dLine, Geom2dOffsetCurve, Geom2dParabola, Geom2dTrimmedCurve,
};
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBezierCurve, GeomCircle, GeomCurve, GeomEllipse, GeomHyperbola, GeomLine,
    GeomOffsetCurve, GeomParabola, GeomTrimmedCurve,
};
use rustycad_tkmath::elslib;
use rustycad_tkmath::gp::{Axis3, Direction3, Plane3, Point2, Point3, Vector3};
use rustycad_tkmath::math_poly::LongDouble;

/// `findExactPolePlane`: the plane of an exactly planar control polygon, `None`
/// otherwise; coincident poles give the XY plane at the first pole, collinear ones a
/// plane containing their line.
fn exact_pole_plane(poles: &[Point3]) -> Result<Option<Plane3>, Failure> {
    let Some(&origin) = poles.first() else {
        return Ok(None);
    };
    let mut base: Vector3 = Vector3::default();
    for pole in poles {
        base = Vector3::between(origin, *pole);
        if base.square_magnitude() > 0.0 {
            break;
        }
    }
    if base.square_magnitude() == 0.0 {
        let frame = Axis3::new(origin, Direction3::Z, Direction3::X)?;
        return Ok(Some(Plane3::from_position(frame)));
    }
    let (second, mut normal) = spanning_normal(poles, origin, base);
    let x_direction: Direction3 = Direction3::from_vector(base)?;
    if normal.square_magnitude() == 0.0 {
        let reference = if x_direction.z().abs() < 0.9 {
            Direction3::Z
        } else {
            Direction3::Y
        };
        normal = Vector3::from(x_direction).crossed(Vector3::from(reference));
        let frame = Axis3::new(origin, Direction3::from_vector(normal)?, x_direction)?;
        return Ok(Some(Plane3::from_position(frame)));
    }
    if poles.iter().any(|pole| {
        triple_product(base, second, Vector3::between(origin, *pole)) != LongDouble::ZERO
    }) {
        return Ok(None);
    }
    let frame = Axis3::new(origin, Direction3::from_vector(normal)?, x_direction)?;
    Ok(Some(Plane3::from_position(frame)))
}

// The first pole not collinear with `base` and the normal it spans (the last pole
// and a null normal when all are collinear).
fn spanning_normal(poles: &[Point3], origin: Point3, base: Vector3) -> (Vector3, Vector3) {
    let (mut second, mut normal) = (Vector3::default(), Vector3::default());
    for pole in poles {
        second = Vector3::between(origin, *pole);
        normal = base.crossed(second);
        if normal.square_magnitude() > 0.0 {
            break;
        }
    }
    (second, normal)
}

// `base . (second x vector)` expanded in `long double`, in the native order.
fn triple_product(base: Vector3, second: Vector3, vector: Vector3) -> LongDouble {
    let ld = LongDouble::new;
    let term_x = ld(base.x()) * (ld(second.y()) * ld(vector.z()) - ld(second.z()) * ld(vector.y()));
    let term_y = ld(base.y()) * (ld(second.x()) * ld(vector.z()) - ld(second.z()) * ld(vector.x()));
    let term_z = ld(base.z()) * (ld(second.x()) * ld(vector.y()) - ld(second.y()) * ld(vector.x()));
    term_x - term_y + term_z
}

/// `isParallel`: exactly parallel directions (a null cross product).
fn is_parallel(first: Direction3, second: Direction3) -> bool {
    Vector3::from(first)
        .crossed(Vector3::from(second))
        .square_magnitude()
        == 0.0
}

// The poles projected on the plane, with the bounds of the 3D array.
fn projected_poles(
    poles: &Array1<'static, Point3>,
    plane: &Plane3,
) -> Result<Array1<'static, Point2>, Failure> {
    let mut projected: Array1<'static, Point2> = Array1::with_bounds(poles.lower(), poles.upper())
        .map_err(|error| error.into_failure("ExtremaPC::ProjectBezier"))?;
    for index in poles.lower()..=poles.upper() {
        projected[index] = projlib::project_point(plane, poles[index]);
    }
    Ok(projected)
}

/// `ProjectBezier`: the 2D Bezier curve of a planar Bezier curve without an evaluation
/// representation, in `plane`'s coordinates. `plane` and `curve2d` are the native
/// output parameters (set as natively, also when false is returned after the
/// projection); true when an exact projection keeping the rationality was built;
/// `project_bezier(&curve, &mut plane, &mut curve2d)?`.
pub fn project_bezier(
    curve: &Handle<GeomBezierCurve>,
    plane: &mut Plane3,
    curve2d: &mut Handle<Geom2dBezierCurve>,
) -> Result<bool, Failure> {
    curve2d.nullify();
    let Some(bezier) = curve.get() else {
        return Ok(false);
    };
    if bezier.has_eval_representation() {
        return Ok(false);
    }
    let poles3d: Array1<'static, Point3> = bezier.poles();
    let Some(found) = exact_pole_plane(poles3d.as_slice())? else {
        return Ok(false);
    };
    *plane = found;
    let poles: Array1<'static, Point2> = projected_poles(&poles3d, plane)?;
    let projected = if bezier.is_rational() {
        bezier.weights_array(|weights| Geom2dBezierCurve::with_weights(&poles, weights))?
    } else {
        Geom2dBezierCurve::new(&poles)?
    };
    let same_rationality: bool = projected.is_rational() == bezier.is_rational();
    *curve2d = Handle::new(projected);
    Ok(same_rationality)
}

/// `ProjectBSpline`: as [`project_bezier`] for a BSpline curve (knots,
/// multiplicities, degree and periodicity kept); `project_bspline(&c, &mut p, &mut c2d)?`.
pub fn project_bspline(
    curve: &Handle<GeomBSplineCurve>,
    plane: &mut Plane3,
    curve2d: &mut Handle<Geom2dBSplineCurve>,
) -> Result<bool, Failure> {
    curve2d.nullify();
    let Some(bspline) = curve.get() else {
        return Ok(false);
    };
    if bspline.has_eval_representation() {
        return Ok(false);
    }
    let poles3d: Array1<'static, Point3> = bspline.poles();
    let Some(found) = exact_pole_plane(poles3d.as_slice())? else {
        return Ok(false);
    };
    *plane = found;
    let poles: Array1<'static, Point2> = projected_poles(&poles3d, plane)?;
    let (knots, mults) = (bspline.knots(), bspline.multiplicities());
    let (degree, periodic) = (bspline.degree(), bspline.is_periodic());
    let projected = if bspline.is_rational() {
        bspline.weights_array(|weights| {
            Geom2dBSplineCurve::with_weights(&poles, weights, &knots, &mults, degree, periodic)
        })?
    } else {
        Geom2dBSplineCurve::new(&poles, &knots, &mults, degree, periodic)?
    };
    let same_rationality: bool = projected.is_rational() == bspline.is_rational();
    *curve2d = Handle::new(projected);
    Ok(same_rationality)
}

/// `ProjectOffset`: the 2D offset curve of a 3D offset curve whose basis has an exact
/// projection in a plane normal to the offset direction; the 2D offset sign is the one
/// that reproduces the 3D point at the middle parameter exactly;
/// `project_offset(&curve, &mut plane, &mut curve2d)?`.
pub fn project_offset(
    curve: &Handle<GeomOffsetCurve>,
    plane: &mut Plane3,
    curve2d: &mut Handle<Geom2dOffsetCurve>,
) -> Result<bool, Failure> {
    curve2d.nullify();
    let Some(offset) = curve.get() else {
        return Ok(false);
    };
    if offset.has_eval_representation() {
        return Ok(false);
    }
    let mut basis2d: Handle<dyn Geom2dCurve> = Handle::default();
    if !project_basis_curve(
        &offset.basis_curve(),
        offset.direction(),
        plane,
        &mut basis2d,
    )? {
        return Ok(false);
    }
    let probe: f64 = (offset.first_parameter() + offset.last_parameter()) * 0.5;
    if !probe.is_finite() {
        return Ok(false);
    }
    for sign in [-1.0, 1.0] {
        let candidate = Geom2dOffsetCurve::new(&basis2d, sign * offset.offset(), false)?;
        let point2d: Point2 = candidate.value(probe)?;
        let point3d: Point3 = elslib::value(point2d.x(), point2d.y(), plane);
        if point3d.square_distance(&offset.value(probe)?) == 0.0 {
            *curve2d = Handle::new(candidate);
            return Ok(true);
        }
    }
    Ok(false)
}

/// `projectBasisCurve`: the exact 2D image of an offset basis in a plane normal to
/// `normal`; trimmed curves recurse on their basis.
fn project_basis_curve(
    curve: &Handle<dyn GeomCurve>,
    normal: Direction3,
    plane: &mut Plane3,
    curve2d: &mut Handle<dyn Geom2dCurve>,
) -> Result<bool, Failure> {
    curve2d.nullify();
    if curve.is_null() {
        return Ok(false);
    }
    if let Some(trimmed) = curve.cast::<GeomTrimmedCurve>().get() {
        let mut basis2d: Handle<dyn Geom2dCurve> = Handle::default();
        if !project_basis_curve(&trimmed.basis_curve(), normal, plane, &mut basis2d)? {
            return Ok(false);
        }
        let (first, last) = (trimmed.first_parameter(), trimmed.last_parameter());
        *curve2d = Handle::new(Geom2dTrimmedCurve::new(&basis2d, first, last, true, true)?).cast();
        return Ok(true);
    }
    if let Some(line) = curve.cast::<GeomLine>().get() {
        return project_basis_line(line, normal, plane, curve2d);
    }
    if let Some(projected) = project_basis_conic(curve, normal, plane)? {
        *curve2d = projected;
        return Ok(true);
    }
    project_basis_spline(curve, normal, plane, curve2d)
}

// The line case: the line must be exactly orthogonal to the normal.
fn project_basis_line(
    line: &GeomLine,
    normal: Direction3,
    plane: &mut Plane3,
    curve2d: &mut Handle<dyn Geom2dCurve>,
) -> Result<bool, Failure> {
    let lin = line.lin();
    if Vector3::from(lin.direction()).dot(Vector3::from(normal)) != 0.0 {
        return Ok(false);
    }
    *plane = Plane3::from_position(Axis3::new(lin.location(), normal, lin.direction())?);
    let projected = projlib::project_line(plane, &lin)?;
    *curve2d = Handle::new(Geom2dLine::new(projected.position())).cast();
    Ok(true)
}

// The conic cases: the conic axis must be exactly parallel to the normal; the plane is
// the conic's own.
fn project_basis_conic(
    curve: &Handle<dyn GeomCurve>,
    normal: Direction3,
    plane: &mut Plane3,
) -> Result<Option<Handle<dyn Geom2dCurve>>, Failure> {
    if let Some(circle) = curve.cast::<GeomCircle>().get() {
        let circ = circle.circ()?;
        if is_parallel(circ.axis().direction(), normal) {
            *plane = Plane3::from_position(Axis3::from_axis2(circ.position()));
            let c = projlib::project_circle(plane, &circ)?;
            return Ok(Some(
                Handle::new(Geom2dCircle::new(c.position(), c.radius())?).cast(),
            ));
        }
    }
    if let Some(ellipse) = curve.cast::<GeomEllipse>().get() {
        let elips = ellipse.elips()?;
        if is_parallel(elips.axis().direction(), normal) {
            *plane = Plane3::from_position(Axis3::from_axis2(elips.position()));
            let e = projlib::project_ellipse(plane, &elips)?;
            let geom = Geom2dEllipse::new(e.axis(), e.major_radius(), e.minor_radius())?;
            return Ok(Some(Handle::new(geom).cast()));
        }
    }
    project_basis_open_conic(curve, normal, plane)
}

// The hyperbola and parabola cases of project_basis_conic.
fn project_basis_open_conic(
    curve: &Handle<dyn GeomCurve>,
    normal: Direction3,
    plane: &mut Plane3,
) -> Result<Option<Handle<dyn Geom2dCurve>>, Failure> {
    if let Some(hyperbola) = curve.cast::<GeomHyperbola>().get() {
        let hypr = hyperbola.hypr()?;
        if is_parallel(hypr.axis().direction(), normal) {
            *plane = Plane3::from_position(Axis3::from_axis2(hypr.position()));
            let h = projlib::project_hyperbola(plane, &hypr)?;
            let geom = Geom2dHyperbola::new(h.axis(), h.major_radius(), h.minor_radius())?;
            return Ok(Some(Handle::new(geom).cast()));
        }
    }
    if let Some(parabola) = curve.cast::<GeomParabola>().get() {
        let parab = parabola.parab()?;
        if is_parallel(parab.axis().direction(), normal) {
            *plane = Plane3::from_position(Axis3::from_axis2(parab.position()));
            let p = projlib::project_parabola(plane, &parab)?;
            return Ok(Some(
                Handle::new(Geom2dParabola::new(p.axis(), p.focal())?).cast(),
            ));
        }
    }
    Ok(None)
}

// The Bezier and BSpline cases: an exact pole plane normal to `normal`.
fn project_basis_spline(
    curve: &Handle<dyn GeomCurve>,
    normal: Direction3,
    plane: &mut Plane3,
    curve2d: &mut Handle<dyn Geom2dCurve>,
) -> Result<bool, Failure> {
    let bezier: Handle<GeomBezierCurve> = curve.cast();
    let mut bezier2d: Handle<Geom2dBezierCurve> = Handle::default();
    if !bezier.is_null()
        && project_bezier(&bezier, plane, &mut bezier2d)?
        && is_parallel(plane.position().direction(), normal)
    {
        *curve2d = bezier2d.cast();
        return Ok(true);
    }
    let bspline: Handle<GeomBSplineCurve> = curve.cast();
    let mut bspline2d: Handle<Geom2dBSplineCurve> = Handle::default();
    if !bspline.is_null()
        && project_bspline(&bspline, plane, &mut bspline2d)?
        && is_parallel(plane.position().direction(), normal)
    {
        *curve2d = bspline2d.cast();
        return Ok(true);
    }
    Ok(false)
}
