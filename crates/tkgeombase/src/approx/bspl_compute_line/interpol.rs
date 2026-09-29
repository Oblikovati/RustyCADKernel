// Copyright (c) 1995-1999 Matra Datavision
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
// Rust translation of ModelingData/TKGeomBase/Approx/Approx_BSplComputeLine.gxx (Interpol, TangencyVector)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Approx_BSplComputeLine::Interpol`: the C2 cubic interpolation of the line.
use super::BSplComputeLine;
use crate::app_par_curves::arrays::scratch;
use crate::app_par_curves::{Constraint, LeastSquare, LineTool, MultiBSpCurve};
use crate::approx::compute_line::owned_array;
use crate::approx::tangency::{LineEnd, curve_tangents, search_lambda, tangency_vector};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::Vector;

/// The degree of the interpolation.
const INTERPOLATION_DEGREE: i32 = 3;
/// `nnp`: the most points of the Bezier fits that give the end tangents.
const TANGENT_FIT_POINTS: i32 = 9;

/// The end tangents and their lambdas.
struct EndTangents {
    v1: Vector<'static>,
    v2: Vector<'static>,
    lambda1: f64,
    lambda2: f64,
}

impl BSplComputeLine {
    /// `Interpol(Line)`: the interpolation of every point by a C2 cubic B-spline with
    /// a knot at each parameter and the end tangents of the line (or of Bezier fits
    /// of the first and last points); a straight segment for two points. Sets the
    /// continuity to 2; `compute.interpol(&line)?`.
    pub fn interpol<L: LineTool + Clone>(&mut self, line: &L) -> Result<(), Failure> {
        self.continuity = 2;
        let (first, last) = (line.first_point(), line.last_point()?);
        let the_param: Vector<'static> = self.initial_parameters(line, first, last)?;
        if last - first + 1 == 2 {
            self.interpolate_segment(line, [first, last], &the_param)?;
        } else {
            self.interpolate_points(line, [first, last], &the_param)?;
        }
        self.my_parameters = Some(owned_array(&the_param)?);
        Ok(())
    }

    // Two points: the degree 1 fit over the knots [TheParam(first), TheParam(last)].
    fn interpolate_segment<L: LineTool>(
        &mut self,
        line: &L,
        [first, last]: [i32; 2],
        the_param: &Vector<'static>,
    ) -> Result<(), Failure> {
        let cons: Constraint = Constraint::NoConstraint;
        let mut lsq = LeastSquare::new(line, first, last, cons, cons, the_param, 2)?;
        self.alldone = lsq.is_done();
        let mut knots: Array1<'static, f64> = scratch(2)?;
        let mut mults: Array1<'static, i32> = scratch(2)?;
        (knots[1], knots[2]) = (*the_param.value(first)?, *the_param.value(last)?);
        (mults[1], mults[2]) = (2, 2);
        self.the_multi_bsp_curve =
            MultiBSpCurve::from_multi_curve(&lsq.bezier_value()?, &knots, &mults);
        (_, self.currenttol3d, self.currenttol2d) = lsq.error()?;
        Ok(())
    }

    fn interpolate_points<L: LineTool>(
        &mut self,
        line: &L,
        [first, last]: [i32; 2],
        the_param: &Vector<'static>,
    ) -> Result<(), Failure> {
        let nbpoints: i32 = last - first + 1;
        let (knots, mults) = interpolation_knots(the_param, nbpoints)?;
        let mut ends: EndTangents = if nbpoints == 3 || nbpoints == 4 {
            self.line_end_tangents(line, [first, last], the_param, &knots)?
        } else {
            fitted_end_tangents(line, [first, last], the_param, &knots)?
        };
        if self.periodic {
            let mean: Vector<'static> = ends.v1.added(&ends.v2)?.t_multiplied(0.5);
            ends.v1 = mean.clone();
            ends.v2 = mean;
        }
        let cons: Constraint = Constraint::TangencyPoint;
        let mut sq = LeastSquare::with_knots_without_parameters(
            line,
            &knots,
            &mults,
            first,
            last,
            cons,
            cons,
            nbpoints + 2,
        )?;
        sq.perform_tangents(the_param, &ends.v1, &ends.v2, ends.lambda1, ends.lambda2)?;
        self.alldone = sq.is_done();
        self.the_multi_bsp_curve = sq.bspline_value()?.clone();
        (_, self.currenttol3d, self.currenttol2d) = sq.error()?;
        self.tolreached = true;
        Ok(())
    }

    // Three or four points: the line tangents (or parabola tangents) with the chord
    // lambdas scaled by the first and last spans.
    fn line_end_tangents<L: LineTool>(
        &self,
        line: &L,
        [first, last]: [i32; 2],
        the_param: &Vector<'static>,
        knots: &Array1<'static, f64>,
    ) -> Result<EndTangents, Failure> {
        let size: i32 = 3 * line.nb_p3d()? + 2 * line.nb_p2d()?;
        let (mut v1, mut v2) = (Vector::with_bounds(1, size)?, Vector::with_bounds(1, size)?);
        let params =
            |l: &L, f: i32, t: i32, p: &mut Vector<'static>| self.compute_parameters(l, f, t, p);
        let n: i32 = knots.length();
        let span = |i: i32| knots[i + 1] - knots[i];
        let whole: f64 = knots[n] - knots[1];
        tangency_vector(line, first, LineEnd::First, &mut v1, &params)?;
        let lambda1: f64 =
            search_lambda(line, the_param, &v1, [first, first + 1])? * span(1) / whole;
        tangency_vector(line, last, LineEnd::Last, &mut v2, &params)?;
        let lambda2: f64 =
            search_lambda(line, the_param, &v2, [last - 1, last])? * span(n - 1) / whole;
        let deg: f64 = f64::from(INTERPOLATION_DEGREE);
        Ok(EndTangents {
            v1,
            v2,
            lambda1: lambda1 / deg,
            lambda2: lambda2 / deg,
        })
    }
}

// The knots TheParam(first..=last) with the end multiplicities 4 and the inner 1.
fn interpolation_knots(
    the_param: &Vector<'static>,
    nbknots: i32,
) -> Result<(Array1<'static, f64>, Array1<'static, i32>), Failure> {
    let mut knots: Array1<'static, f64> = scratch(nbknots)?;
    let mut mults: Array1<'static, i32> = scratch(nbknots)?;
    knots[1] = *the_param.value(the_param.lower())?;
    knots[nbknots] = *the_param.value(the_param.upper())?;
    mults[1] = INTERPOLATION_DEGREE + 1;
    mults[nbknots] = INTERPOLATION_DEGREE + 1;
    let low: i32 = the_param.lower();
    for i in 2..=nbknots - 1 {
        knots[i] = *the_param.value(i + low - 1)?;
        mults[i] = 1;
    }
    Ok((knots, mults))
}

// Five points or more: the tangents of Bezier fits of the first and last nine (at
// most) points, with the lambdas of the first and last spans.
fn fitted_end_tangents<L: LineTool>(
    line: &L,
    [first, last]: [i32; 2],
    the_param: &Vector<'static>,
    knots: &Array1<'static, f64>,
) -> Result<EndTangents, Failure> {
    let size: i32 = 3 * line.nb_p3d()? + 2 * line.nb_p2d()?;
    let (mut v1, mut v2) = (Vector::with_bounds(1, size)?, Vector::with_bounds(1, size)?);
    let nnp: i32 = (last - first + 1).min(TANGENT_FIT_POINTS);
    let lastp: i32 = last.min(first + nnp - 1);
    let fit1 = bezier_fit(line, [first, lastp], the_param, nnp)?;
    curve_tangents(line, &fit1, 0.0, &mut v1)?;
    let firstp: i32 = first.max(last - nnp + 1);
    if firstp == first && lastp == last {
        curve_tangents(line, &fit1, 1.0, &mut v2)?;
    } else {
        let fit2 = bezier_fit(line, [firstp, last], the_param, nnp)?;
        curve_tangents(line, &fit2, 1.0, &mut v2)?;
    }
    let n: i32 = knots.length();
    let deg: f64 = f64::from(INTERPOLATION_DEGREE);
    let lambda1: f64 = 1. / deg * (knots[2] - knots[1]) / (knots[n] - knots[1]);
    let lambda2: f64 = 1. / deg * (knots[n] - knots[n - 1]) / (knots[n] - knots[1]);
    Ok(EndTangents {
        v1,
        v2,
        lambda1,
        lambda2,
    })
}

// The Bezier curve of `nnpol` poles fitted with tangency ends to the points
// `first..=last` at their parameters.
fn bezier_fit<L: LineTool>(
    line: &L,
    [first, last]: [i32; 2],
    the_param: &Vector<'static>,
    nnpol: i32,
) -> Result<crate::app_par_curves::MultiCurve, Failure> {
    let cons: Constraint = Constraint::TangencyPoint;
    let mut sq = LeastSquare::without_parameters(line, first, last, cons, cons, nnpol)?;
    let mut p: Vector<'static> = Vector::with_bounds(first, last)?;
    for i in first..=last {
        *p.change_value(i)? = *the_param.value(i)?;
    }
    sq.perform(&p)?;
    sq.bezier_value()
}
