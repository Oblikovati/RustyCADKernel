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

//! The `ax_fit` and `ax_mcurves` requests.
use super::super::app_par_curves::curve;
use super::super::app_par_curves::read_points;
use super::super::common::{self as p, Reader, attempt, end};
use super::line::{source_d2, source_d3, source_p2, source_p3};
use super::{bspline, constraint};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::app_cont::{Function, FunctionBase};
use rustycad_tkgeombase::app_par_curves::MultiCurve;
use rustycad_tkgeombase::approx::{ComputeCLine, ComputeCLineOptions, MCurvesToBSpCurve};
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};

/// GbAxFunction: the source curves over `[u0, u1]`.
struct AxFunction {
    base: FunctionBase,
    range: [f64; 2],
    d1_fail: bool,
    periodic_dim: i32,
    period: f64,
}

impl Function for AxFunction {
    fn base(&self) -> &FunctionBase {
        &self.base
    }
    fn first_parameter(&self) -> f64 {
        self.range[0]
    }
    fn last_parameter(&self) -> f64 {
        self.range[1]
    }
    fn value(&self, u: f64, p2: &mut Array1<'_, Point2>, p3: &mut Array1<'_, Point3>) -> bool {
        for k in 1..=self.get_nb_of_3d_points() {
            p3[k] = source_p3(k, u);
        }
        for k in 1..=self.get_nb_of_2d_points() {
            p2[k] = source_p2(k, u);
        }
        true
    }
    fn d1(&self, u: f64, v2: &mut Array1<'_, Vector2>, v3: &mut Array1<'_, Vector3>) -> bool {
        if self.d1_fail && (u == self.range[0] || u == self.range[1]) {
            return false;
        }
        for k in 1..=self.get_nb_of_3d_points() {
            v3[k] = source_d3(k, u);
        }
        for k in 1..=self.get_nb_of_2d_points() {
            v2[k] = source_d2(k, u);
        }
        true
    }
    fn period_information(&self, dim_idx: i32) -> (bool, f64) {
        let periodic: bool = dim_idx == self.periodic_dim;
        (periodic, if periodic { self.period } else { 0. })
    }
}

fn fit_results(f: &ComputeCLine) -> Result<(), Failure> {
    p::flag(f.is_all_approximated());
    p::flag(f.is_tolerance_reached());
    let n: i32 = f.nb_multi_curves();
    p::int(n);
    for i in 1..=n {
        curve(&f.value(i)?)?;
        let (t3, t2) = f.error(i)?;
        let (p1, p2) = f.parameters(i)?;
        [t3, t2, p1, p2].into_iter().for_each(p::real);
    }
    attempt(|| curve(&f.value(n + 1)?));
    Ok(())
}

fn fit_run(r: &mut Reader<'_>, function: &AxFunction, variant: i32) -> Result<(), Failure> {
    let (degree_min, degree_max) = (r.integer(), r.integer());
    let (tolerance_3d, tolerance_2d, cutting) = (r.real(), r.real(), r.flag());
    let (first, last) = (constraint(r.integer()), constraint(r.integer()));
    let (max_segments, inv, hang) = (r.integer(), r.flag(), r.flag());
    let options = ComputeCLineOptions {
        degree_min,
        degree_max,
        tolerance_3d,
        tolerance_2d,
        cutting,
        first_constraint: first,
        last_constraint: last,
    };
    if variant == 0 {
        return fit_results(&ComputeCLine::with_line(function, options)?);
    }
    let mut f = ComputeCLine::new(options);
    if max_segments > 0 {
        f.set_max_segments(max_segments);
    }
    f.set_inv_order(inv);
    f.set_hang_checking(hang);
    f.perform(function)?;
    fit_results(&f)?;
    f.set_degrees(degree_min + 1, degree_max);
    f.set_tolerances(tolerance_3d * 0.5, tolerance_2d * 0.5);
    f.set_constraints(last, first);
    f.perform(function)?;
    fit_results(&f)
}

pub fn fit_case(values: &[f64]) {
    let mut r = Reader::new(values);
    // FitAndDivide and FitAndDivide2d are the same instantiation.
    let _class = r.integer();
    let (nb3d, nb2d) = (r.integer(), r.integer());
    let range = [r.real(), r.real()];
    let d1_fail = r.flag();
    let periodic_dim = r.integer();
    let period = r.real();
    let variant = r.integer();
    let mut base = FunctionBase::new();
    base.set_number_of_points(nb3d, nb2d);
    let function = AxFunction {
        base,
        range,
        d1_fail,
        periodic_dim,
        period,
    };
    attempt(|| fit_run(&mut r, &function, variant));
    end();
}

pub fn mcurves_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let count = r.integer();
        let mut sequence: Vec<MultiCurve> = Vec::new();
        for _ in 0..count {
            sequence.push(MultiCurve::from_multi_points(&read_points(&mut r)?));
        }
        let variant = r.integer();
        let mut join = MCurvesToBSpCurve::new();
        if variant == 1 {
            join.perform_sequence(&sequence)?;
        } else {
            sequence.iter().for_each(|c| join.append(c));
            if variant == 2 {
                join.reset();
                if let Some(last) = sequence.last() {
                    join.append(last);
                }
            }
            join.perform()?;
        }
        bspline(join.value())?;
        bspline(join.change_value())
    });
    end();
}
