// Copyright (c) 1991-1999 Matra Datavision
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
// Rust translation of ModelingData/TKGeomBase/GeomConvert/GeomConvert_1.cxx
// (SurfaceToBSplineSurface: dispatch, direct case, BSplineSurfaceBuilder and the
// shared builders) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomConvert::SurfaceToBSplineSurface`: the B-spline of a bounded Geom surface.
use super::approx_surface::ApproxSurface;
use super::curve_to_bspline::curve_to_bspline_curve;
use super::live_bspline;
use super::surface_to_bspline_trimmed::trimmed_to_bspline;
use crate::array_access::{grid, put};
use crate::native_failure::failure;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBSplineSurface, GeomBezierSurface, GeomCurve, GeomOffsetSurface,
    GeomRectangularTrimmedSurface, GeomSphericalSurface, GeomSurface, GeomSurfaceOfRevolution,
    GeomSweptSurface, GeomToroidalSurface,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkmath::convert::{ElementarySurfaceToBSplineSurface, ParameterisationType};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Axis1, GeneralTransform3, Point3, Transform3, Xyz};
use std::f64::consts::PI;

/// `GeomConvert::SurfaceToBSplineSurface(S)`: the B-spline of a bounded surface.
///
/// An offset surface is converted through its equivalent surface when it has one;
/// a rectangular trimmed surface converts its basis on the trimmed domain; a sphere,
/// torus, revolution, Bezier or B-spline surface is converted exactly; any other
/// surface is approximated (tolerance 1e-4, degree 14, `4 (NbUIntervals + 1)
/// (NbVIntervals + 1)` segments). Standard_DomainError for an infinite surface
/// ("GeomConvert::SurfaceToBSplineSurface() - infinite surface");
/// `geom_convert::surface_to_bspline_surface(&trimmed_plane)?`.
pub fn surface_to_bspline_surface(
    sr: &Handle<dyn GeomSurface>,
) -> Result<Handle<GeomBSplineSurface>, Failure> {
    let source: &dyn GeomSurface = sr.get().ok_or_else(|| {
        failure(
            "Standard_NullObject",
            "GeomConvert::SurfaceToBSplineSurface: null surface; expected a Geom_Surface",
        )
    })?;
    let (u1, u2, v1, v2) = source.bounds();
    let (u_first, u_last) = (u1.min(u2), u1.max(u2));
    let (v_first, v_last) = (v1.min(v2), v1.max(v2));
    // If the surface Sr is infinite stop the computation.
    if precision::is_negative_infinite(u_first)
        || precision::is_positive_infinite(u_last)
        || precision::is_negative_infinite(v_first)
        || precision::is_positive_infinite(v_last)
    {
        return Err(failure(
            "Standard_DomainError",
            "GeomConvert::SurfaceToBSplineSurface() - infinite surface",
        ));
    }
    if let Some(offset) = sr.cast::<GeomOffsetSurface>().get() {
        let equivalent: Handle<dyn GeomSurface> = offset.surface()?;
        if !equivalent.is_null() {
            // Convert the equivalent surface.
            return surface_to_bspline_surface(&equivalent);
        }
    }
    let trimmed: Handle<GeomRectangularTrimmedSurface> = sr.cast();
    match trimmed.get() {
        Some(strim) => trimmed_to_bspline(sr, strim, [u1, u2, v1, v2]),
        None => direct_to_bspline(sr, u_first),
    }
}

// The untrimmed sphere, torus, revolution, Bezier and B-spline surfaces; any other
// surface is approximated.
fn direct_to_bspline(
    s: &Handle<dyn GeomSurface>,
    u_first: f64,
) -> Result<Handle<GeomBSplineSurface>, Failure> {
    if let Some(sphere) = s.cast::<GeomSphericalSurface>().get() {
        return bspline_surface_builder(&ElementarySurfaceToBSplineSurface::from_sphere(
            &sphere.sphere()?,
        )?);
    }
    if let Some(torus) = s.cast::<GeomToroidalSurface>().get() {
        return bspline_surface_builder(&ElementarySurfaceToBSplineSurface::from_torus(
            &torus.torus()?,
        )?);
    }
    if let Some(revol) = s.cast::<GeomSurfaceOfRevolution>().get() {
        let meridian: Handle<GeomBSplineCurve> =
            curve_to_bspline_curve(&revol.basis_curve(), ParameterisationType::TgtThetaOver2)?;
        let sweep = RevolutionSweep {
            axis: revol.axis(),
            u_first,
            alfa_u: PI / 3.0,
            nb_u_poles: 6,
            nb_u_knots: 4,
            periodic: true,
        };
        return sweep.build(&meridian);
    }
    if s.cast::<GeomBezierSurface>().is_some() {
        return bezier_surface_to_bspline(&s.cast());
    }
    if s.cast::<GeomBSplineSurface>().is_some() {
        // Just a copy.
        return Ok(s.copy()?.cast());
    }
    approximate_direct(s)
}

// In other cases => Approx: the continuity in each direction is the highest of C2,
// C1 and C0 the surface has.
fn approximate_direct(sr: &Handle<dyn GeomSurface>) -> Result<Handle<GeomBSplineSurface>, Failure> {
    let surface: &dyn GeomSurface = sr
        .get()
        .ok_or_else(|| failure("Standard_NullObject", "null surface"))?;
    let continuity = |cn: &dyn Fn(i32) -> Result<bool, Failure>| -> Result<Shape, Failure> {
        if cn(2)? {
            return Ok(Shape::C2);
        }
        Ok(if cn(1)? { Shape::C1 } else { Shape::C0 })
    };
    let ucont: Shape = continuity(&|n| surface.is_cn_u(n))?;
    let vcont: Shape = continuity(&|n| surface.is_cn_v(n))?;
    approximate(sr, ucont, vcont)
}

/// The approximation fallback of SurfaceToBSplineSurface: tolerance 1e-4, degree 14
/// in both directions, `4 (NbUIntervals(CN) + 1) (NbVIntervals(CN) + 1)` segments,
/// precision code 1; the surface of the result (null when there is none).
pub(super) fn approximate(
    sr: &Handle<dyn GeomSurface>,
    ucont: Shape,
    vcont: Shape,
) -> Result<Handle<GeomBSplineSurface>, Failure> {
    let (tol3d, max_degree): (f64, i32) = (1.0e-4, 14);
    let adaptor = GeomAdaptorSurface::with_surface(sr)?;
    let max_seg: i32 =
        4 * (adaptor.nb_u_intervals(Shape::CN)? + 1) * (adaptor.nb_v_intervals(Shape::CN)? + 1);
    let approx = ApproxSurface::from_surface(
        sr,
        tol3d,
        (ucont, vcont),
        (max_degree, max_degree),
        max_seg,
        1,
    )?;
    Ok(approx.surface())
}

/// `BSplineSurfaceBuilder`: the rational B-spline of an elementary-surface
/// conversion.
pub(super) fn bspline_surface_builder(
    convert: &ElementarySurfaceToBSplineSurface,
) -> Result<Handle<GeomBSplineSurface>, Failure> {
    Ok(Handle::new(GeomBSplineSurface::with_weights(
        convert.poles(),
        convert.weights(),
        (convert.u_knots(), convert.u_multiplicities()),
        (convert.v_knots(), convert.v_multiplicities()),
        (convert.u_degree(), convert.v_degree()),
        (convert.is_u_periodic(), convert.is_v_periodic()),
    )?))
}

/// A Bezier surface as a single-patch B-spline on [0, 1] x [0, 1], rational when
/// the Bezier is in either direction.
pub(super) fn bezier_surface_to_bspline(
    bezier: &Handle<GeomBezierSurface>,
) -> Result<Handle<GeomBSplineSurface>, Failure> {
    let sbez: &GeomBezierSurface = bezier.get().ok_or_else(|| {
        failure(
            "Standard_NullObject",
            "GeomConvert: null Geom_BezierSurface; expected a surface",
        )
    })?;
    let (udegree, vdegree): (i32, i32) = (sbez.u_degree(), sbez.v_degree());
    let (u_knots, u_mults) = single_span(udegree)?;
    let (v_knots, v_mults) = single_span(vdegree)?;
    let poles: Array2<'static, Point3> = sbez.poles();
    let u = (&u_knots, &u_mults);
    let v = (&v_knots, &v_mults);
    let surface = match sbez.weights() {
        Some(weights) if sbez.is_u_rational() || sbez.is_v_rational() => {
            GeomBSplineSurface::with_weights(
                &poles,
                &weights,
                u,
                v,
                (udegree, vdegree),
                (false, false),
            )?
        }
        _ => GeomBSplineSurface::new(&poles, u, v, (udegree, vdegree), (false, false))?,
    };
    Ok(Handle::new(surface))
}

// The knots 0 and 1 with multiplicity Degree + 1.
fn single_span(degree: i32) -> Result<(Array1<'static, f64>, Array1<'static, i32>), Failure> {
    let mut knots: Array1<'static, f64> = one_based(2)?;
    (knots[1], knots[2]) = (0.0, 1.0);
    let mut mults: Array1<'static, i32> = one_based(2)?;
    (mults[1], mults[2]) = (degree + 1, degree + 1);
    Ok((knots, mults))
}

/// The U structure of the B-spline of a surface of revolution: spans of opening
/// `2 AlfaU` from `u_first`, each span two rational poles of degree 2 (the odd poles
/// are the meridian's rotated, the even ones its affinity by `1 / cos(AlfaU)`
/// rotated, with the weights times `cos(AlfaU)`).
pub(super) struct RevolutionSweep {
    pub axis: Axis1,
    pub u_first: f64,
    pub alfa_u: f64,
    pub nb_u_poles: i32,
    pub nb_u_knots: i32,
    pub periodic: bool,
}

impl RevolutionSweep {
    /// The B-spline surface of the meridian `c` swept around the axis.
    pub fn build(
        &self,
        c: &Handle<GeomBSplineCurve>,
    ) -> Result<Handle<GeomBSplineSurface>, Failure> {
        let meridian: &GeomBSplineCurve = live_bspline(c)?;
        let (u_knots, u_mults) = self.u_knots()?;
        let (new_poles, new_weights) = self.poles(meridian)?;
        Ok(Handle::new(GeomBSplineSurface::with_weights(
            &new_poles,
            &new_weights,
            (&u_knots, &u_mults),
            (&meridian.knots(), &meridian.multiplicities()),
            (2, meridian.degree()),
            (self.periodic, meridian.is_periodic()),
        )?))
    }

    // Knots UFirst + (i - 1) 2 AlfaU of multiplicity 2, 3 at the ends when not
    // periodic.
    fn u_knots(&self) -> Result<(Array1<'static, f64>, Array1<'static, i32>), Failure> {
        let mut u_knots: Array1<'static, f64> = one_based(self.nb_u_knots)?;
        let mut u_mults: Array1<'static, i32> = one_based(self.nb_u_knots)?;
        for i in 1..=self.nb_u_knots {
            u_knots[i] = self.u_first + f64::from(i - 1) * 2.0 * self.alfa_u;
            u_mults[i] = 2;
        }
        if !self.periodic {
            u_mults[1] += 1;
            u_mults[self.nb_u_knots] += 1;
        }
        Ok((u_knots, u_mults))
    }

    // Compute the poles: the odd rows first, then the affinity of the meridian poles
    // and the even rows.
    fn poles(
        &self,
        meridian: &GeomBSplineCurve,
    ) -> Result<(Array2<'static, Point3>, Array2<'static, f64>), Failure> {
        let nb_v_poles: i32 = meridian.nb_poles();
        let mut poles: Array1<'static, Point3> = meridian.poles();
        let weights: Array1<'static, f64> = meridian.weights_array(|w| w.clone());
        let mut new_poles: Array2<'static, Point3> = grid(self.nb_u_poles, nb_v_poles)?;
        let mut new_weights: Array2<'static, f64> = grid(self.nb_u_poles, nb_v_poles)?;
        self.fill_rows(
            1,
            &poles,
            |j| weights[j],
            (&mut new_poles, &mut new_weights),
        )?;
        let mut aff = GeneralTransform3::default();
        aff.set_affinity_axis(self.axis, 1.0 / self.alfa_u.cos());
        for j in 1..=nb_v_poles {
            let mut coord: Xyz = poles[j].xyz();
            aff.transforms(&mut coord);
            poles[j].set_xyz(coord);
        }
        let cos_alfa: f64 = self.alfa_u.cos();
        self.fill_rows(
            2,
            &poles,
            |j| weights[j] * cos_alfa,
            (&mut new_poles, &mut new_weights),
        )?;
        Ok((new_poles, new_weights))
    }

    // Rows first_row, first_row + 2, ...: the poles rotated by UFirst + (i - 1) AlfaU.
    fn fill_rows(
        &self,
        first_row: i32,
        poles: &Array1<'static, Point3>,
        weight: impl Fn(i32) -> f64,
        (new_poles, new_weights): (&mut Array2<'static, Point3>, &mut Array2<'static, f64>),
    ) -> Result<(), Failure> {
        let mut trsf = Transform3::default();
        for i in (first_row..=self.nb_u_poles).step_by(2) {
            trsf.set_rotation(self.axis, self.u_first + f64::from(i - 1) * self.alfa_u)?;
            for j in 1..=poles.length() {
                put(new_poles, i, j, poles[j].transformed(trsf), "NewPoles")?;
                put(new_weights, i, j, weight(j), "NewWeights")?;
            }
        }
        Ok(())
    }
}

/// The meridian of a surface of revolution or extrusion, whole or trimmed to
/// `[first, last]`, converted to a B-spline.
pub(super) fn swept_meridian(
    meridian: &Handle<dyn GeomCurve>,
    whole: bool,
    (first, last): (f64, f64),
) -> Result<Handle<GeomBSplineCurve>, Failure> {
    if whole {
        return curve_to_bspline_curve(meridian, ParameterisationType::TgtThetaOver2);
    }
    let ct = rustycad_tkg3d::geom::GeomTrimmedCurve::new(meridian, first, last, true, true)?;
    curve_to_bspline_curve(
        &Handle::new(ct).cast::<dyn GeomCurve>(),
        ParameterisationType::TgtThetaOver2,
    )
}
