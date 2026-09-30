// Copyright (c) 1994-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
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
// Rust translation of GeomFill::Surface of
// ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill::Surface(Curve1, Curve2)`: the ruled surface between two curves — a
//! plane between two matching segments, a cylinder or a cone between two coaxial
//! full circles, else the skinning of `GeomFill_Generator`.
use super::frame_math::{std_max, std_min};
use super::generator::GeomFillGenerator;
use super::sweep_failures::live;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{
    GeomCircle, GeomConicalSurface, GeomCurve, GeomCylindricalSurface, GeomLine, GeomPlane,
    GeomRectangularTrimmedSurface, GeomSurface, GeomTrimmedCurve,
};
use rustycad_tkmath::gp::{Axis3, Direction3, Line3, Vector3};

/// A curve with the bounds of its trim, if it was trimmed.
type BoundedCurve = (Handle<dyn GeomCurve>, Option<(f64, f64)>);

/// The basis and the bounds of a trimmed curve, or a copy of another curve.
fn untrimmed(curve: &Handle<dyn GeomCurve>) -> Result<BoundedCurve, Failure> {
    let c = live(curve, "GeomFill::Surface", "Curve")?;
    let trimmed: Handle<GeomTrimmedCurve> = curve.cast();
    if let Some(t) = trimmed.get() {
        return Ok((
            t.basis_curve(),
            Some((t.first_parameter(), t.last_parameter())),
        ));
    }
    Ok((c.copy()?.cast(), None))
}

/// The plane between two matching segments of parallel lines (`None` otherwise).
fn plane_between(
    l1: Line3,
    l2: Line3,
    (a1, b1): (f64, f64),
    (a2, b2): (f64, f64),
) -> Result<Option<Handle<dyn GeomSurface>>, Failure> {
    let (d1, d2) = (l1.direction(), l2.direction());
    let ang = precision::angular();
    if !d1.is_parallel(d2, ang) {
        return Ok(None);
    }
    let p1p2 = Vector3::between(l1.location(), l2.location());
    let proj = p1p2.dot(Vector3::from(d1));
    let conf = precision::confusion();
    let mut surf = None;
    let same = (a1 - proj - a2).abs() <= conf && (b1 - proj - b2).abs() <= conf;
    if d1.is_equal(&d2, ang) && same {
        surf = Some(plane_surface(l1, d1, p1p2, (a1, b1))?);
    }
    let reversed = (a1 - proj + b2).abs() <= conf && (b1 - proj + a2).abs() <= conf;
    if d1.is_opposite(d2, ang) && reversed {
        surf = Some(plane_surface(l1, d1, p1p2, (a1, b1))?);
    }
    Ok(surf)
}

/// The plane through the first line and the vector to the second, trimmed to the
/// first segment and to the band between the lines.
fn plane_surface(
    l1: Line3,
    d1: Direction3,
    p1p2: Vector3,
    (a1, b1): (f64, f64),
) -> Result<Handle<dyn GeomSurface>, Failure> {
    let normal = d1.crossed(Direction3::from_vector(p1p2)?)?;
    let ax = Axis3::new(l1.location(), normal, d1)?;
    let p: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(ax)).cast();
    let v = p1p2.dot(Vector3::from(ax.y_direction()));
    let trimmed = GeomRectangularTrimmedSurface::new(
        &p,
        (a1, b1),
        (std_min(0.0, v), std_max(0.0, v)),
        (true, true),
    )?;
    Ok(Handle::new(trimmed).cast())
}

/// A cylinder (equal radii) or a cone between two coaxial full circles, trimmed in
/// V to the distance between them (`None` when they are not coaxial).
fn cylinder_or_cone(
    c1: &GeomCircle,
    c2: &GeomCircle,
) -> Result<Option<Handle<dyn GeomSurface>>, Failure> {
    let (c1, c2) = (c1.circ()?, c2.circ()?);
    let a1 = Axis3::from_axis2(c1.position());
    let a2 = Axis3::from_axis2(c2.position());
    if !a1
        .axis()
        .is_coaxial(a2.axis(), precision::angular(), precision::confusion())
    {
        return Ok(None);
    }
    let v = Vector3::between(a1.location(), a2.location()).dot(Vector3::from(a1.direction()));
    let (c, v) = revolved_between(a1, (c1.radius(), c2.radius()), v)?;
    let trimmed = GeomRectangularTrimmedSurface::with_one_direction(
        &c,
        std_min(0.0, v),
        std_max(0.0, v),
        false,
        true,
    )?;
    Ok(Some(Handle::new(trimmed).cast()))
}

/// The cylinder (equal radii) or the cone on `a1` between circles of radii `r` at
/// the axial distance `v`, and the V length on it.
fn revolved_between(
    mut a1: Axis3,
    r: (f64, f64),
    mut v: f64,
) -> Result<(Handle<dyn GeomSurface>, f64), Failure> {
    if (r.0 - r.1).abs() < precision::confusion() {
        return Ok((Handle::new(GeomCylindricalSurface::new(a1, r.0)?).cast(), v));
    }
    let rad = r.1 - r.0;
    let mut ang = (rad / v).atan();
    if ang < 0.0 {
        a1.z_reverse();
        v = -v;
        ang = -ang;
    }
    let cone: Handle<dyn GeomSurface> = Handle::new(GeomConicalSurface::new(a1, ang, r.0)?).cast();
    v /= ang.cos();
    Ok((cone, v))
}

/// The elementary ruled surface of two trimmed lines or two full circles, or
/// `None` (trimmed circles are left to the generator, as natively).
fn elementary(
    (c1, trim1): (&Handle<dyn GeomCurve>, Option<(f64, f64)>),
    (c2, trim2): (&Handle<dyn GeomCurve>, Option<(f64, f64)>),
) -> Result<Option<Handle<dyn GeomSurface>>, Failure> {
    let (l1, l2): (Handle<GeomLine>, Handle<GeomLine>) = (c1.cast(), c2.cast());
    if let (Some(l1), Some(l2), Some(t1), Some(t2)) = (l1.get(), l2.get(), trim1, trim2) {
        return plane_between(l1.lin(), l2.lin(), t1, t2);
    }
    let (k1, k2): (Handle<GeomCircle>, Handle<GeomCircle>) = (c1.cast(), c2.cast());
    if let (Some(k1), Some(k2)) = (k1.get(), k2.get())
        && trim1.is_none()
        && trim2.is_none()
    {
        return cylinder_or_cone(k1, k2);
    }
    Ok(None)
}

/// `GeomFill::Surface(Curve1, Curve2)`: the ruled surface between two curves;
/// `let s = geom_fill::surface(&c1, &c2)?;`.
pub fn surface(
    curve1: &Handle<dyn GeomCurve>,
    curve2: &Handle<dyn GeomCurve>,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    let first = untrimmed(curve1)?;
    let second = untrimmed(curve2)?;
    if let Some(surf) = elementary((&first.0, first.1), (&second.0, second.1))? {
        return Ok(surf);
    }
    let mut generator = GeomFillGenerator::new();
    generator.add_curve(curve1)?;
    generator.add_curve(curve2)?;
    generator.perform(precision::p_confusion())?;
    Ok(generator.surface())
}
