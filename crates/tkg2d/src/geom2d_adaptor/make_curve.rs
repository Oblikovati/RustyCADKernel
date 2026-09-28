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
// Rust translation of ModelingData/TKG2d/Geom2dAdaptor/Geom2dAdaptor.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dAdaptor::MakeCurve`: a Geom2d curve from an adapted curve.
use super::curve::Geom2dAdaptorCurve;
use crate::adaptor2d::Adaptor2dCurve2d;
use crate::geom2d::errors::domain;
use crate::geom2d::native::{std_max, std_min};
use crate::geom2d::{
    Geom2dCircle, Geom2dCurve, Geom2dEllipse, Geom2dHyperbola, Geom2dLine, Geom2dParabola,
    Geom2dTrimmedCurve,
};
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkmath::geom_abs::CurveType;

/// A Geom2d curve equal to the adapted curve `adaptor` (`Geom2dAdaptor::MakeCurve`).
///
/// Elementary kinds are rebuilt from their gp geometry, Bezier and B-spline curves
/// are shared, and an offset kind is the curve of a `Geom2dAdaptor_Curve`. When the
/// adaptor range differs from the curve range, the result is a trimmed curve on the
/// adaptor range, clipped to the curve range unless the curve is periodic.
/// Standard_DomainError for other kinds and for an offset that is not a
/// Geom2dAdaptor_Curve; `make_curve(&*adaptor)?`.
pub fn make_curve(adaptor: &dyn Adaptor2dCurve2d) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let curve: Handle<dyn Geom2dCurve> = curve_of_kind(adaptor)?;
    let Some(built) = curve.get() else {
        return Ok(curve);
    };
    let (first, last) = (adaptor.first_parameter()?, adaptor.last_parameter()?);
    if first == built.first_parameter() && last == built.last_parameter() {
        return Ok(curve);
    }
    let trimmed: Geom2dTrimmedCurve = if built.is_periodic()
        || (adaptor.first_parameter()? >= built.first_parameter()
            && adaptor.last_parameter()? <= built.last_parameter())
    {
        Geom2dTrimmedCurve::new(
            &curve,
            adaptor.first_parameter()?,
            adaptor.last_parameter()?,
            true,
            true,
        )?
    } else {
        let start: f64 = std_max(adaptor.first_parameter()?, built.first_parameter());
        let end: f64 = std_min(adaptor.last_parameter()?, built.last_parameter());
        Geom2dTrimmedCurve::new(&curve, start, end, true, true)?
    };
    Ok(Handle::new(trimmed).cast())
}

// The untrimmed curve of the adaptor's kind.
fn curve_of_kind(adaptor: &dyn Adaptor2dCurve2d) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    Ok(match adaptor.get_type()? {
        CurveType::Line => Handle::new(Geom2dLine::from_lin2d(adaptor.line()?)).cast(),
        CurveType::Circle => Handle::new(Geom2dCircle::from_circ2d(adaptor.circle()?)).cast(),
        CurveType::Ellipse => Handle::new(Geom2dEllipse::from_elips2d(adaptor.ellipse()?)).cast(),
        CurveType::Parabola => {
            Handle::new(Geom2dParabola::from_parab2d(adaptor.parabola()?)).cast()
        }
        CurveType::Hyperbola => {
            Handle::new(Geom2dHyperbola::from_hypr2d(adaptor.hyperbola()?)).cast()
        }
        CurveType::BezierCurve => adaptor.bezier()?.cast(),
        CurveType::BSplineCurve => adaptor.bspline()?.cast(),
        CurveType::OffsetCurve => match Geom2dAdaptorCurve::project_from(adaptor) {
            Some(geom_adaptor) => geom_adaptor.curve(),
            None => return Err(domain("Geom2dAdaptor::MakeCurve, Not Geom2dAdaptor_Curve")),
        },
        CurveType::OtherCurve => return Err(domain("Geom2dAdaptor::MakeCurve, OtherCurve")),
    })
}
