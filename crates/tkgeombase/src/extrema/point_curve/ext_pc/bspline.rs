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
// Rust translation of the GeomAbs_BSplineCurve case of Extrema_GGExtPC::Perform at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The B-spline case of Extrema_GGExtPC: the knot spans of the range (shifted by
//! whole periods), searched span by span for degree 1 and around the local extrema
//! of `Degree + 1` samples per span otherwise.
use super::GGExtPC;
use crate::extrema::curve_tool::{ExtremaCurveTool, SplineKnots};
use crate::extrema::failures::null_object;
use crate::extrema::gp_space::{GpPoint, GpVector};
use crate::extrema::pon::POnCurve;
use crate::gcpnts::PoleCurve;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::std_min;

/// The knot spans of the range: the knots, the period jump and the first and last
/// used knot indexes.
struct KnotSpans {
    knots: Array1<'static, f64>,
    jump: f64,
    first: i32,
    last: i32,
}

impl KnotSpans {
    fn knot(&self, index: i32) -> Result<f64, Failure> {
        let value: &f64 = self
            .knots
            .value(index)
            .map_err(|error| error.into_failure("Extrema_GGExtPC: Knots()"))?;
        Ok(value + self.jump)
    }
    // The bounds of span `index`, the first starting at `uinf` and the last (or,
    // with `else_if`, a later one only) ending at `usup`.
    fn span(
        &self,
        index: i32,
        (uinf, usup): (f64, f64),
        else_if: bool,
    ) -> Result<(f64, f64), Failure> {
        let (mut f, mut l) = (self.knot(index)?, self.knot(index + 1)?);
        if index == self.first {
            f = uinf;
            if else_if {
                return Ok((f, l));
            }
        }
        if index == self.last - 1 {
            l = usup;
        }
        Ok((f, l))
    }
}

impl<'a, T: ExtremaCurveTool> GGExtPC<'a, T> {
    // The knot spans covering `[myuinf, myusup]`, with the period shift of a
    // periodic curve.
    fn knot_spans(&self, spline: &T::BSpline, curve: &T::Curve) -> Result<KnotSpans, Failure> {
        let (first_idx, last_idx) = (spline.first_knot_index()?, spline.last_knot_index()?);
        let mut spans = KnotSpans {
            knots: spline.knot_values(),
            jump: 0.0,
            first: first_idx,
            last: last_idx,
        };
        let tol_coeff: f64 = (self.usup - self.uinf) * precision::p_confusion();
        if T::is_periodic(curve)? {
            let first_knot: f64 = spans.knot(first_idx)?;
            let mut shift: i32 = ((self.uinf - first_knot) / T::period(curve)?) as i32;
            if self.uinf < first_knot - tol_coeff {
                shift -= 1;
            }
            spans.jump = T::period(curve)? * f64::from(shift);
        }
        for index in first_idx..=last_idx {
            if self.uinf >= spans.knot(index)? - tol_coeff {
                spans.first = index;
            } else {
                break;
            }
        }
        for index in (first_idx..=last_idx).rev() {
            if self.usup <= spans.knot(index)? + tol_coeff {
                spans.last = index;
            } else {
                break;
            }
        }
        if spans.first == spans.last {
            spans.first = first_idx;
            spans.last = first_idx + 1;
        }
        Ok(spans)
    }

    /// The GeomAbs_BSplineCurve case of `Perform`.
    pub(super) fn perform_bspline(
        &mut self,
        point: T::Point,
        curve: &'a T::Curve,
    ) -> Result<(), Failure> {
        let handle = T::bspline(curve)?;
        let spline: &T::BSpline = handle
            .get()
            .ok_or_else(|| null_object("Extrema_GGExtPC: BSpline()"))?;
        let spans: KnotSpans = self.knot_spans(spline, curve)?;
        self.sample = spline.pole_degree() + 1;
        if self.sample == 2 {
            self.perform_linear_spans(point, curve, &spans)?;
        } else {
            self.perform_sampled_spans(point, curve, &spans)?;
        }
        self.done = true;
        Ok(())
    }

    // Degree 1: each span whose end projections change sign, or that nearly touches
    // the point, is searched; the nearest inner knot is added as a minimum when no
    // found extremum is closer or at the same parameter.
    fn perform_linear_spans(
        &mut self,
        point: T::Point,
        curve: &'a T::Curve,
        spans: &KnotSpans,
    ) -> Result<(), Failure> {
        let (mut p_min, mut t_min, mut dist_min) = (T::Point::default(), 0.0, f64::MAX);
        let mut min2: f64 = 0.0;
        self.ext_pc.initialize_curve(curve)?;
        for index in spans.first..spans.last {
            let (f, l) = spans.span(index, (self.uinf, self.usup), true)?;
            let (p1, p2) = (T::d0(curve, f)?, T::d0(curve, l)?);
            let (base1, base2, v) = (
                point.vector_to(&p1),
                point.vector_to(&p2),
                p2.vector_to(&p1),
            );
            let (val1, val2) = (v.dot_with(&base1), v.dot_with(&base2));
            let min1: f64 = if index == spans.first {
                point.square_distance_to(&p1)
            } else {
                min2
            };
            if index != spans.first && dist_min > min1 {
                (dist_min, t_min, p_min) = (min1, f, p1);
            }
            min2 = point.square_distance_to(&p2);
            let min_sq_dist: f64 = std_min(min1, min2);
            let min_der: f64 = std_min(val1.abs(), val2.abs());
            if precision::is_infinite(val1) || precision::is_infinite(val2) {
                continue;
            }
            let near: bool = min_sq_dist < 100. * precision::square_confusion();
            if val1 * val2 <= 0.0 || near || 2. * min_der < precision::confusion() {
                (self.int_uinf, self.int_usup) = (f, l);
                self.interval_perform(point)?;
            }
        }
        self.add_nearest_knot(dist_min, t_min, p_min);
        Ok(())
    }

    fn add_nearest_knot(&mut self, dist_min: f64, t_min: f64, p_min: T::Point) {
        if precision::is_infinite(dist_min) {
            return;
        }
        let mut is_to_add: bool = true;
        for (point, sq_dist) in self.points.iter().zip(&self.sq_dist) {
            is_to_add = (dist_min < *sq_dist) && ((point.parameter() - t_min).abs() > self.tol_u);
            if !is_to_add {
                break;
            }
        }
        if is_to_add {
            self.push_extremum(dist_min, true, POnCurve::new(t_min, p_min));
        }
    }
}

impl<'a, T: ExtremaCurveTool> GGExtPC<'a, T> {
    // Degree > 1: `mysample` distances per span and the last end; every local extremum
    // of the samples is searched between its neighbours, a sample within
    // Precision::SquareConfusion() is kept as is, and the end spans are searched when
    // the ends do not touch the point but their projections change sign.
    fn perform_sampled_spans(
        &mut self,
        point: T::Point,
        curve: &'a T::Curve,
        spans: &KnotSpans,
    ) -> Result<(), Failure> {
        let (values, params) = self.span_samples(point, curve, spans)?;
        self.ext_pc.initialize_curve(curve)?;
        let sq_conf: f64 = precision::square_confusion();
        for i in 1..values.len() - 1 {
            if values[i] <= sq_conf {
                let on_curve = POnCurve::new(params[i], T::value(curve, params[i])?);
                self.push_extremum(values[i], true, on_curve);
            }
            let (prev, v, next) = (values[i - 1], values[i], values[i + 1]);
            if (v >= next && v >= prev) || (v <= next && v <= prev) {
                (self.int_uinf, self.int_usup) = (params[i - 1], params[i + 1]);
                self.interval_perform(point)?;
            }
        }
        let count: usize = params.len();
        if self.dist1 > sq_conf && !precision::is_positive_infinite(self.dist1) {
            self.end_span(point, curve, (params[0], params[1]), true)?;
        }
        if self.dist2 > sq_conf && !precision::is_positive_infinite(self.dist2) {
            self.end_span(point, curve, (params[count - 2], params[count - 1]), false)?;
        }
        Ok(())
    }

    // The square distances and parameters of the samples (`aVal`, `aParam`).
    fn span_samples(
        &self,
        point: T::Point,
        curve: &T::Curve,
        spans: &KnotSpans,
    ) -> Result<(Vec<f64>, Vec<f64>), Failure> {
        let (mut values, mut params): (Vec<f64>, Vec<f64>) = (Vec::new(), Vec::new());
        for index in spans.first..spans.last {
            let (f, l) = spans.span(index, (self.uinf, self.usup), false)?;
            let step: f64 = (l - f) / f64::from(self.sample);
            for pnt_idx in 0..self.sample {
                let param: f64 = f + step * f64::from(pnt_idx);
                values.push(T::value(curve, param)?.square_distance_to(&point));
                params.push(param);
            }
        }
        values.push(T::value(curve, self.usup)?.square_distance_to(&point));
        params.push(self.usup);
        Ok((values, params))
    }

    // One end span, searched when the projections of its ends change sign, point
    // apart, or the end projection vanishes.
    fn end_span(
        &mut self,
        point: T::Point,
        curve: &'a T::Curve,
        (a, b): (f64, f64),
        first: bool,
    ) -> Result<(), Failure> {
        let (p1, v1) = T::d1(curve, a)?;
        let (p2, v2) = T::d1(curve, b)?;
        let (base1, base2) = (point.vector_to(&p1), point.vector_to(&p2));
        let (val1, val2) = (v1.dot_with(&base1), v2.dot_with(&base2));
        if precision::is_infinite(val1) || precision::is_infinite(val2) {
            return Ok(());
        }
        let end_value: f64 = if first { val1 } else { val2 };
        if val1 * val2 <= 0.0
            || base1.dot_with(&base2) <= 0.0
            || 2.0 * end_value.abs() < precision::confusion()
        {
            (self.int_uinf, self.int_usup) = (a, b);
            self.interval_perform(point)?;
        }
        Ok(())
    }
}
