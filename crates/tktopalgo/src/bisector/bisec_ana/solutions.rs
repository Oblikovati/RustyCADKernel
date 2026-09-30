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
// Rust translation of ModelingAlgorithms/TKTopAlgo/Bisector/Bisector_BisecAna.cxx (Degenerate and the trimming of a chosen solution) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The GccInt solutions of the analytic bisectors: the replacement of degenerate
//! conics by lines, and the trimmed curves built from the chosen solution.
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{
    Geom2dCircle, Geom2dCurve, Geom2dEllipse, Geom2dHyperbola, Geom2dLine, Geom2dTrimmedCurve,
};
use rustycad_tkgeomalgo::gcc_int::{BLine, Bisec, IType};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Line2, Point2};
use std::f64::consts::PI;

/// A trimmed bisector, as the native `thebisector` field holds it.
pub(super) type Trimmed = Handle<Geom2dTrimmedCurve>;

/// `Degenerate(aBisector, Tolerance)`: a hyperbola with a radius, a parabola with a
/// focal or an ellipse with a minor radius below `tolerance` becomes the line of one
/// of its axes; the solution is returned unchanged otherwise.
pub(super) fn degenerate(
    bisector: Handle<dyn Bisec>,
    tolerance: f64,
) -> Result<Handle<dyn Bisec>, Failure> {
    match bisector.arc_type() {
        IType::Hpr => degenerate_hyperbola(bisector, tolerance),
        IType::Par => {
            let parabola = bisector.parabola()?;
            if parabola.focal() < tolerance {
                return Ok(line_solution(Line2::from(parabola.axis_of_symmetry())));
            }
            Ok(bisector)
        }
        IType::Ell => {
            let ellipse = bisector.ellipse()?;
            if ellipse.minor_radius() < tolerance {
                return Ok(line_solution(Line2::from(ellipse.x_axis())));
            }
            Ok(bisector)
        }
        _ => Ok(bisector),
    }
}

// A degenerate hyperbola: the minor radius test runs after the major one and wins.
fn degenerate_hyperbola(
    bisector: Handle<dyn Bisec>,
    tolerance: f64,
) -> Result<Handle<dyn Bisec>, Failure> {
    let hyperbola = bisector.hyperbola()?;
    let mut result: Handle<dyn Bisec> = bisector;
    if hyperbola.major_radius() < tolerance {
        result = line_solution(Line2::from(hyperbola.y_axis()));
    }
    if hyperbola.minor_radius() < tolerance {
        result = line_solution(Line2::from(hyperbola.x_axis()));
    }
    Ok(result)
}

/// `new GccInt_BLine(line)` as a `GccInt_Bisec` handle.
pub(super) fn line_solution(line: Line2) -> Handle<dyn Bisec> {
    Handle::new(BLine::new(line)).cast()
}

/// `new Geom2d_TrimmedCurve(curve, u1, u2, sense)` (the periodic bounds adjusted).
pub(super) fn trimmed(
    curve: Handle<dyn Geom2dCurve>,
    u1: f64,
    u2: f64,
    sense: bool,
) -> Result<Trimmed, Failure> {
    Ok(Handle::new(Geom2dTrimmedCurve::new(
        &curve, u1, u2, sense, true,
    )?))
}

/// A line solution trimmed from `first` to its side of `center` (an extremity of the
/// segment 1e-8 past the centre, or infinite when the centre is behind): the
/// bisector of a circle and a line or point it touches.
pub(super) fn line_to_center(
    line: Line2,
    center: Point2,
    first: f64,
    sense: bool,
) -> Result<Trimmed, Failure> {
    let mut second: f64 = elclib::parameter(&line, center)?;
    let curve: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_lin2d(line)).cast();
    if !sense {
        second = if second > first {
            -precision::infinite()
        } else {
            second - 1.0E-8
        };
    } else {
        second = if second < first {
            precision::infinite()
        } else {
            second + 1.0E-8
        };
    }
    trimmed(curve, first, second, true)
}

/// A circle, hyperbola or ellipse solution trimmed from `first`: a whole turn of the
/// closed conics (in the sense `sense`), a half branch of the hyperbola. `None` for
/// any other type, which the native code leaves untrimmed.
pub(super) fn trim_conic(
    solution: &dyn Bisec,
    first: f64,
    sense: bool,
) -> Result<Option<Trimmed>, Failure> {
    let curve: Handle<dyn Geom2dCurve> = match solution.arc_type() {
        IType::Cir => Handle::new(Geom2dCircle::from_circ2d(solution.circle()?)).cast(),
        IType::Ell => Handle::new(Geom2dEllipse::from_elips2d(solution.ellipse()?)).cast(),
        IType::Hpr => {
            let curve = Handle::new(Geom2dHyperbola::from_hypr2d(solution.hyperbola()?)).cast();
            return half_line_like(curve, first, sense).map(Some);
        }
        _ => return Ok(None),
    };
    if !sense {
        return trimmed(curve, first - 2.0 * PI, first, sense).map(Some);
    }
    trimmed(curve, first, first + 2.0 * PI, sense).map(Some)
}

/// `curve` trimmed from `first` to the infinity of its side: `-Infinite` when `sense`
/// is false, `+Infinite` otherwise.
pub(super) fn half_line_like(
    curve: Handle<dyn Geom2dCurve>,
    first: f64,
    sense: bool,
) -> Result<Trimmed, Failure> {
    let last: f64 = if sense {
        precision::infinite()
    } else {
        -precision::infinite()
    };
    trimmed(curve, first, last, true)
}
