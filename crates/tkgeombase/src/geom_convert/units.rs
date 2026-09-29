// Copyright (c) 2021 OPEN CASCADE SAS
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
// Rust translation of ModelingData/TKGeomBase/GeomConvert/GeomConvert_Units.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomConvert_Units`: the conversion of pcurves between angle and length units.
use crate::geom2d_convert::curve_to_bspline_curve;
use crate::native_failure::failure;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBoundedCurve, Geom2dCircle, Geom2dConic, Geom2dCurve, Geom2dEllipse,
    Geom2dGeometry, Geom2dHyperbola, Geom2dLine, Geom2dParabola,
};
use rustycad_tkg3d::geom::{
    GeomConicalSurface, GeomCylindricalSurface, GeomPlane, GeomSphericalSurface, GeomSurface,
    GeomSurfaceOfRevolution, GeomToroidalSurface,
};
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::gp::{self, Axis2d, Direction2, GeneralTransform2, Point2, Transform2, Xy};

/// The static members of `GeomConvert_Units`.
///
/// `let pcurve = Units::radian_to_degree(&pcurve, &cylinder, 1.0, 180.0 / PI)?;`
pub struct Units;

// How a pcurve's coordinates scale on a surface: the U and V factors, or the plane
// case where circles and ellipses are scaled by the length factor.
enum Scaling {
    Factors(f64, f64),
    Plane(f64),
    Unchanged,
}

impl Units {
    /// `RadianToDegree(theCurve, theSurface, theLengthFactor, theFactorRadianDegree)`:
    /// a copy of the pcurve with its angular coordinates multiplied by
    /// `factor_radian_degree` and its length coordinates divided by `length_factor`.
    ///
    /// The factors follow the surface: both angular on a sphere or torus, U angular
    /// and V length on a cylinder, U angular on a revolution, V length times the cone
    /// cosine on a cone, both length on a plane (where a circle or ellipse is scaled
    /// about the origin instead), none elsewhere. A line gets new coordinates; a
    /// circle or ellipse becomes a B-spline; parabolas, hyperbolas and bounded curves
    /// other than B-splines keep their coordinates;
    /// `Units::radian_to_degree(&pcurve, &cylinder, 1.0, 180.0 / PI)?`.
    pub fn radian_to_degree(
        curve_2d: &Handle<dyn Geom2dCurve>,
        surf: &Handle<dyn GeomSurface>,
        length_factor: f64,
        factor_radian_degree: f64,
    ) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        let mut a_curve_2d: Handle<dyn Geom2dCurve> = copied(curve_2d)?;
        let length_fact: f64 = 1.0 / length_factor;
        let scaling: Scaling = surface_scaling(surf, factor_radian_degree, length_fact)?;
        let Some((u_fact, v_fact)) = factors_or_scaled(scaling, &a_curve_2d)? else {
            return Ok(a_curve_2d);
        };
        if let Some(line) = a_curve_2d.cast::<Geom2dLine>().get() {
            let copy: Handle<Geom2dLine> = line.copy()?.cast();
            scale_line(live(&copy)?, u_fact, v_fact)?;
            return Ok(copy.cast());
        }
        if is_elliptic(&a_curve_2d) {
            a_curve_2d =
                curve_to_bspline_curve(&a_curve_2d, ParameterisationType::TgtThetaOver2)?.cast();
        }
        let is_bounded_bspline: bool = a_curve_2d.cast::<dyn Geom2dBoundedCurve>().is_some()
            && a_curve_2d.cast::<Geom2dBSplineCurve>().is_some();
        if !is_bounded_bspline {
            return Ok(a_curve_2d);
        }
        let copy: Handle<Geom2dBSplineCurve> = a_curve_2d.copy()?.cast();
        scale_poles(
            live(&copy)?,
            &live(&a_curve_2d.cast::<Geom2dBSplineCurve>())?.poles(),
            u_fact,
            v_fact,
        )?;
        Ok(copy.cast())
    }

    /// `DegreeToRadian(thePcurve, theSurface, theLengthFactor, theFactorRadianDegree)`:
    /// the inverse conversion, with the length coordinates multiplied by
    /// `length_factor` (divided by the cone cosine on a cone); a parabola or
    /// hyperbola is returned unchanged, and any curve other than a line or B-spline
    /// keeps its coordinates. Unlike RadianToDegree, the line and B-spline copies are
    /// modified in place; `Units::degree_to_radian(&pcurve, &cylinder, 1.0, PI / 180.0)?`.
    pub fn degree_to_radian(
        pcurve: &Handle<dyn Geom2dCurve>,
        surface: &Handle<dyn GeomSurface>,
        length_factor: f64,
        factor_radian_degree: f64,
    ) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        let mut a_pcurve: Handle<dyn Geom2dCurve> = copied(pcurve)?;
        let scaling: Scaling =
            surface_scaling_inverse(surface, factor_radian_degree, length_factor)?;
        let Some((u_fact, v_fact)) = factors_or_scaled(scaling, &a_pcurve)? else {
            return Ok(a_pcurve);
        };
        if a_pcurve.cast::<dyn Geom2dConic>().is_some() {
            if is_elliptic(&a_pcurve) {
                a_pcurve =
                    curve_to_bspline_curve(&a_pcurve, ParameterisationType::TgtThetaOver2)?.cast();
            } else if a_pcurve.cast::<Geom2dParabola>().is_some()
                || a_pcurve.cast::<Geom2dHyperbola>().is_some()
            {
                return Ok(a_pcurve);
            }
        }
        if let Some(line) = a_pcurve.cast::<Geom2dLine>().get() {
            scale_line(line, u_fact, v_fact)?;
        } else if let Some(bspline) = a_pcurve.cast::<Geom2dBSplineCurve>().get() {
            scale_poles(bspline, &bspline.poles(), u_fact, v_fact)?;
        }
        Ok(a_pcurve)
    }

    /// `MirrorPCurve(theCurve)`: a copy of the curve mirrored about the X axis of the
    /// parametric plane; `Units::mirror_pcurve(&pcurve)?`.
    pub fn mirror_pcurve(
        curve: &Handle<dyn Geom2dCurve>,
    ) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        let the_mirrored: Handle<dyn Geom2dCurve> = copied(curve)?;
        let mut t = Transform2::default();
        let ax2 = Axis2d::new(Point2::new(0.0, 0.0), gp::dx2d());
        t.set_mirror_axis(ax2);
        live(&the_mirrored)?.transform(t)?;
        Ok(the_mirrored)
    }
}

fn copied(curve: &Handle<dyn Geom2dCurve>) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    Ok(live(curve)?.copy()?.cast())
}

fn live<T: ?Sized + rustycad_tkernel::standard::RuntimeTyped>(
    handle: &Handle<T>,
) -> Result<&T, Failure> {
    handle.get().ok_or_else(|| {
        failure(
            "Standard_NullObject",
            "GeomConvert_Units: null curve or surface handle",
        )
    })
}

fn is_elliptic(curve: &Handle<dyn Geom2dCurve>) -> bool {
    curve.cast::<Geom2dCircle>().is_some() || curve.cast::<Geom2dEllipse>().is_some()
}

// The factors of RadianToDegree (AngleFact, LengthFact = 1 / theLengthFactor).
fn surface_scaling(
    surf: &Handle<dyn GeomSurface>,
    angle: f64,
    length: f64,
) -> Result<Scaling, Failure> {
    live(surf)?;
    if surf.cast::<GeomSphericalSurface>().is_some() || surf.cast::<GeomToroidalSurface>().is_some()
    {
        return Ok(Scaling::Factors(angle, angle));
    }
    if surf.cast::<GeomCylindricalSurface>().is_some() {
        return Ok(Scaling::Factors(angle, length));
    }
    if surf.cast::<GeomSurfaceOfRevolution>().is_some() {
        return Ok(Scaling::Factors(angle, 1.0));
    }
    if let Some(cone) = surf.cast::<GeomConicalSurface>().get() {
        return Ok(Scaling::Factors(angle, length * cone.semi_angle().cos()));
    }
    if surf.cast::<GeomPlane>().is_some() {
        return Ok(Scaling::Plane(length));
    }
    Ok(Scaling::Unchanged)
}

// The factors of DegreeToRadian (LengthFact = theLengthFactor, a cone divides).
fn surface_scaling_inverse(
    surf: &Handle<dyn GeomSurface>,
    angle: f64,
    length: f64,
) -> Result<Scaling, Failure> {
    if let Some(cone) = surf.cast::<GeomConicalSurface>().get() {
        return Ok(Scaling::Factors(angle, length / cone.semi_angle().cos()));
    }
    surface_scaling(surf, angle, length)
}

// The U and V factors to apply, or None when the conversion is over: on a plane a
// circle or ellipse is scaled about the origin by the length factor (in place) and
// returned, any other curve goes on with both factors equal to it; on a surface
// without factors the curve is returned unchanged.
fn factors_or_scaled(
    scaling: Scaling,
    curve: &Handle<dyn Geom2dCurve>,
) -> Result<Option<(f64, f64)>, Failure> {
    match scaling {
        Scaling::Factors(u_fact, v_fact) => Ok(Some((u_fact, v_fact))),
        Scaling::Unchanged => Ok(None),
        Scaling::Plane(factor) if is_elliptic(curve) => {
            let mut t = Transform2::default();
            t.set_scale(gp::origin2d(), factor);
            live(curve)?.transform(t)?;
            Ok(None)
        }
        Scaling::Plane(factor) => Ok(Some((factor, factor))),
    }
}

// Location (x u, y v) and direction (dx u, dy v), renormalized.
fn scale_line(line: &Geom2dLine, u_fact: f64, v_fact: f64) -> Result<(), Failure> {
    let (loc, dir): (Point2, Direction2) = (line.location(), line.direction());
    line.set_location(Point2::new(loc.x() * u_fact, loc.y() * v_fact));
    line.set_direction(Direction2::new(dir.x() * u_fact, dir.y() * v_fact)?);
    Ok(())
}

// Every pole through the U affinity (about OY) then the V affinity (about OX).
fn scale_poles(
    target: &Geom2dBSplineCurve,
    poles: &rustycad_tkernel::ncollection::Array1<'static, Point2>,
    u_fact: f64,
    v_fact: f64,
) -> Result<(), Failure> {
    let (mut t_matu, mut t_matv) = (GeneralTransform2::default(), GeneralTransform2::default());
    t_matu.set_affinity(gp::oy2d(), u_fact);
    t_matv.set_affinity(gp::ox2d(), v_fact);
    for i in 1..=target.nb_poles() {
        let mut p_xy: Xy = poles[i].xy();
        t_matu.transforms(&mut p_xy);
        t_matv.transforms(&mut p_xy);
        target.set_pole(i, Point2::new(p_xy.x(), p_xy.y()))?;
    }
    Ok(())
}
