// Copyright (c) 1993-1999 Matra Datavision
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
// Rust translation of ModelingData/TKG3d/Geom/Geom_OffsetSurface.cxx (UIso, VIso and their AdvApprox evaluators) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Iso curves of an offset surface: through the equivalent surface when there is
//! one, a translated line for an extrusion basis, otherwise a B-spline
//! approximation of the offset iso by AdvApprox.
use super::GeomOffsetSurface;
use crate::adv_approx::AdvApproxApproxAFunction;
use crate::geom::errors;
use crate::geom::{
    GeomBSplineCurve, GeomCurve, GeomRectangularTrimmedSurface, GeomSurface,
    GeomSurfaceOfLinearExtrusion,
};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point3, Vector3, resolution};

// Whether GeomAdaptor_Surface classifies `surface` as a surface of extrusion: the
// class itself, or a rectangular trimmed surface whose basis is one (the adaptor
// loads the basis of a trimmed surface).
fn adaptor_is_extrusion(surface: &Handle<dyn GeomSurface>) -> bool {
    if surface.cast::<GeomSurfaceOfLinearExtrusion>().is_some() {
        return true;
    }
    match surface.cast::<GeomRectangularTrimmedSurface>().get() {
        Some(trimmed) => trimmed
            .basis_surface()
            .cast::<GeomSurfaceOfLinearExtrusion>()
            .is_some(),
        None => false,
    }
}

// The iso direction of one approximation: the fixed parameter and whether it is U.
#[derive(Clone, Copy)]
struct IsoLine {
    parameter: f64,
    fixed_u: bool,
}

// The Geom_OffsetSurface_UIsoEvaluator / VIsoEvaluator body: the point, or the
// derivative along the free parameter. A surface failure stops the approximation
// (error code 1) and is returned by the caller, as the native exception would be.
fn iso_values(
    surface: &GeomOffsetSurface,
    iso: IsoLine,
    parameter: f64,
    derivative: i32,
) -> Result<[f64; 3], Failure> {
    let (u, v) = if iso.fixed_u {
        (iso.parameter, parameter)
    } else {
        (parameter, iso.parameter)
    };
    if derivative == 0 {
        let p: Point3 = surface.value(u, v)?;
        return Ok([p.x(), p.y(), p.z()]);
    }
    let d1 = surface.d1(u, v)?;
    let d: Vector3 = if iso.fixed_u { d1.d1v } else { d1.d1u };
    Ok([d.x(), d.y(), d.z()])
}

// The AdvApprox B-spline of one iso of `surface` over `[first, last]`
// (Num3DSS = 1, tolerance Precision::Approximation(), C1, degree 14, 100 segments).
fn approximated_iso(
    surface: &GeomOffsetSurface,
    iso: IsoLine,
    range: (f64, f64),
    member: &str,
) -> Result<Handle<dyn GeomCurve>, Failure> {
    let mut tolerances: Array1<'static, f64> =
        Array1::with_bounds(1, 1).map_err(errors::array_bounds)?;
    tolerances.init(precision::approximation());
    let mut failure: Option<Failure> = None;
    let mut evaluator = |_: i32, _: [f64; 2], t: f64, derivative: i32, result: &mut [f64]| -> i32 {
        match iso_values(surface, iso, t, derivative) {
            Ok(values) => {
                result[..3].copy_from_slice(&values);
                0
            }
            Err(error) => {
                failure.get_or_insert(error);
                1
            }
        }
    };
    let approx = AdvApproxApproxAFunction::new(
        0,
        0,
        1,
        None,
        None,
        Some(&tolerances),
        range.0,
        range.1,
        Shape::C1,
        14,
        100,
        &mut evaluator,
    );
    if let Some(error) = failure {
        return Err(error);
    }
    let approx = approx?;
    if !approx.is_done() {
        return Err(errors::construction(&format!(
            " Geom_OffsetSurface : {member}: the approximation of the iso at {} failed",
            iso.parameter
        )));
    }
    bspline_from(&approx)
}

// new Geom_BSplineCurve(Poles, Knots, Mults, Degree) from the approximation.
fn bspline_from(approx: &AdvApproxApproxAFunction) -> Result<Handle<dyn GeomCurve>, Failure> {
    let mut poles: Array1<'static, Point3> =
        Array1::with_bounds(1, approx.nb_poles()?).map_err(errors::array_bounds)?;
    approx.poles_of(1, &mut poles)?;
    let (Some(knots), Some(mults)) = (approx.knots(), approx.multiplicities()) else {
        return Err(errors::construction(
            " Geom_OffsetSurface : the approximation has no knots; expected a result",
        ));
    };
    let curve = GeomBSplineCurve::new(&poles, knots, mults, approx.degree(), false)?;
    Ok(Handle::new(curve).cast())
}

impl GeomOffsetSurface {
    /// `UIso`: the iso of the equivalent surface, the basis iso translated along the
    /// normal for an extrusion basis, or an AdvApprox B-spline.
    pub(super) fn offset_u_iso(&self, u: f64) -> Result<Handle<dyn GeomCurve>, Failure> {
        let equivalent = self.direct_rep_surface();
        if let Some(equivalent) = equivalent.get() {
            return equivalent.u_iso(u);
        }
        let (basis, offset) = (self.basis_surface(), self.offset());
        if adaptor_is_extrusion(&basis) {
            return extrusion_iso(&basis, u, offset);
        }
        let (_, _, v1, v2) = self.bounds();
        approximated_iso(
            self,
            IsoLine {
                parameter: u,
                fixed_u: true,
            },
            (v1, v2),
            "UIso",
        )
    }
    /// `VIso`: the iso of the equivalent surface or an AdvApprox B-spline.
    pub(super) fn offset_v_iso(&self, v: f64) -> Result<Handle<dyn GeomCurve>, Failure> {
        let equivalent = self.direct_rep_surface();
        if let Some(equivalent) = equivalent.get() {
            return equivalent.v_iso(v);
        }
        let (u1, u2, _, _) = self.bounds();
        approximated_iso(
            self,
            IsoLine {
                parameter: v,
                fixed_u: false,
            },
            (u1, u2),
            "VIso",
        )
    }
}

// The U iso of an extrusion basis translated by the offset along the normal at
// (U, 0); the untranslated iso when that normal is null.
fn extrusion_iso(
    basis: &Handle<dyn GeomSurface>,
    u: f64,
    offset: f64,
) -> Result<Handle<dyn GeomCurve>, Failure> {
    let line: Handle<dyn GeomCurve> = basis.u_iso(u)?;
    let d1 = basis.d1(u, 0.)?;
    let mut dir: Vector3 = d1.d1u.crossed(d1.d1v);
    if dir.square_magnitude() < resolution() {
        return Ok(line);
    }
    dir.normalize()?;
    dir *= offset;
    line.translate(dir)?;
    Ok(line)
}
