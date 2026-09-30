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
// Rust translation of Geom2dAPI_Interpolate.cxx and GeomAPI_Interpolate.cxx
// (PerformPeriodic, PerformNonPeriodic) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The two interpolations of Geom2dAPI_Interpolate and GeomAPI_Interpolate: a cubic
//! B-spline through the points with tangent conditions at the ends (and where
//! requested), or a closed C1 one through the first point.
use super::interpolate::{InterpolateArrays, InterpolateGen};
use super::interpolate_build::{array, at, build_periodic_tangent, build_tangents, put};
use super::interpolate_space::{InterpolateSpace, vector_as_point};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bsplclib;

/// The linear system of BSplCLib::Interpolate being assembled.
struct System<S: InterpolateSpace> {
    degree: i32,
    parameters: Array1<'static, f64>,
    flatknots: Array1<'static, f64>,
    mults: Array1<'static, i32>,
    contact_order_array: Array1<'static, i32>,
    poles: Array1<'static, S::Point>,
}

impl<S: InterpolateSpace> System<S> {
    fn new(num_poles: i32, degree: i32, num_distinct_knots: i32) -> Result<Self, Failure> {
        let mut contact_order_array: Array1<'static, i32> = array(1, num_poles)?;
        contact_order_array.init(0);
        let mut mults: Array1<'static, i32> = array(1, num_distinct_knots)?;
        for ii in 2..num_distinct_knots {
            put(&mut mults, ii, 1)?;
        }
        Ok(Self {
            degree,
            parameters: array(1, num_poles)?,
            flatknots: array(1, num_poles + degree + 1)?,
            mults,
            contact_order_array,
            poles: array(1, num_poles)?,
        })
    }
    fn num_poles(&self) -> i32 {
        self.poles.len() as i32
    }
    // `BSplCLib::Interpolate` in place on `parameters`; `true` without inversion
    // problem.
    fn interpolate(&mut self, parameters: &Array1<'_, f64>) -> Result<bool, Failure> {
        let mut inversion_problem: i32 = 0;
        bsplclib::interpolate(
            self.degree,
            &self.flatknots,
            parameters,
            &self.contact_order_array,
            &mut self.poles,
            &mut inversion_problem,
        )?;
        Ok(inversion_problem == 0)
    }
    // A pole and its derivative condition: the tangent as a pole, contact order 1.
    fn add_tangent(&mut self, index: i32, parameter: f64, tangent: S::Vector) -> Result<(), Failure> {
        put(&mut self.contact_order_array, index, 1)?;
        put(&mut self.parameters, index, parameter)?;
        put(&mut self.poles, index, vector_as_point::<S>(&tangent))
    }
}

/// The contents of the arrays once the tangents are built.
struct Inputs<'g, S: InterpolateSpace> {
    points: &'g Array1<'static, S::Point>,
    parameters: &'g Array1<'static, f64>,
    tangents: &'g Array1<'static, S::Vector>,
    flags: &'g Array1<'static, bool>,
}

// The number of flagged points in `flags(from..=to)`.
fn count_flags(flags: &Array1<'_, bool>, from: i32, to: i32) -> Result<i32, Failure> {
    let mut count: i32 = 0;
    for ii in from..=to {
        count += i32::from(at(flags, ii)?);
    }
    Ok(count)
}

impl<S: InterpolateSpace> InterpolateGen<S> {
    /// `PerformPeriodic()`: a periodic curve of degree 1 through two points without
    /// tangents, otherwise a C1 periodic cubic closing on the first point, with the
    /// tangent of the first point (built by BuildPeriodicTangent when missing).
    pub(super) fn perform_periodic(&mut self) -> Result<(), Failure> {
        let h: InterpolateArrays<S> = self.arrays();
        let num_points: i32 = h.points()?.array1().len() as i32;
        if num_points == 2 && !self.tangent_request() {
            // build a periodic curve of degree 1
            let mut deg1_mults: Array1<'static, i32> = array(1, num_points + 1)?;
            deg1_mults.init(1);
            let (points, parameters) = (h.points()?.array1(), h.parameters()?.array1());
            let curve = S::new_curve(&points, &parameters, &deg1_mults, 1, self.periodic())?;
            self.set_result(curve);
            return Ok(());
        }
        let mut num_poles: i32 = num_points + 3;
        if self.tangent_request() {
            let flags = h.flags()?.array1();
            num_poles += count_flags(&flags, flags.lower() + 1, flags.upper())?;
        }
        if num_points >= 3 {
            // only here with 3 points or more, otherwise the tangent is given
            let (points, parameters) = (h.points()?.array1(), h.parameters()?.array1());
            build_periodic_tangent::<S>(
                &points,
                &mut h.tangents()?.change_array1(),
                &mut h.flags()?.change_array1(),
                &parameters,
            )?;
        }
        let (points, parameters) = (h.points()?.array1(), h.parameters()?.array1());
        let (tangents, flags) = (h.tangents()?.array1(), h.flags()?.array1());
        let inputs = Inputs {
            points: &points,
            parameters: &parameters,
            tangents: &tangents,
            flags: &flags,
        };
        let mut s: System<S> = System::new(num_poles, 3, num_points + 1)?;
        self.fill_periodic(&inputs, &mut s)?;
        let system_parameters: Array1<'static, f64> = s.parameters.clone();
        if !s.interpolate(&system_parameters)? {
            return Ok(());
        }
        let mut newpoles: Array1<'static, S::Point> = array(1, num_poles - 2)?;
        for i in 1..=num_poles - 2 {
            put(&mut newpoles, i, at(&s.poles, i)?)?;
        }
        let curve = S::new_curve(&newpoles, &parameters, &s.mults, 3, self.periodic())?;
        self.set_result(curve);
        Ok(())
    }

    fn fill_periodic(&self, a: &Inputs<'_, S>, s: &mut System<S>) -> Result<(), Failure> {
        let (half_order, num_poles) = (2, s.num_poles());
        let (p_lower, p_upper) = (a.parameters.lower(), a.parameters.upper());
        let period: f64 = at(a.parameters, p_upper)? - at(a.parameters, p_lower)?;
        for ii in 1..=half_order {
            put(
                &mut s.flatknots,
                ii,
                at(a.parameters, p_upper - 1)? - period,
            )?;
            put(
                &mut s.flatknots,
                ii + half_order,
                at(a.parameters, p_lower)?,
            )?;
            put(&mut s.flatknots, num_poles + ii, at(a.parameters, p_upper)?)?;
            put(
                &mut s.flatknots,
                num_poles + half_order + ii,
                at(a.parameters, half_order)? + period,
            )?;
        }
        let last_mult: i32 = s.mults.len() as i32;
        put(&mut s.mults, 1, half_order)?;
        put(&mut s.mults, last_mult, half_order)?;
        s.add_tangent(2, at(a.parameters, 1)?, at(a.tangents, 1)?)?;
        put(&mut s.parameters, 1, at(a.parameters, 1)?)?;
        put(&mut s.poles, 1, at(a.points, 1)?)?;
        if self.tangent_request() {
            fill_periodic_requested(a, s)?;
        } else {
            fill_periodic_plain(a, s)?;
        }
        // for the periodic curve ONLY the tangent of the first point is used since
        // the curve closes itself at the first point (see BuildPeriodicTangent)
        s.add_tangent(
            num_poles - 1,
            at(a.parameters, p_upper)?,
            at(a.tangents, 1)?,
        )?;
        put(&mut s.parameters, num_poles, at(a.parameters, p_upper)?)?;
        put(&mut s.poles, num_poles, at(a.points, 1)?)
    }
}

// The periodic system without requested tangents: every parameter as a knot, every
// point after the first as a pole (the last pole is the first point again).
fn fill_periodic_plain<S: InterpolateSpace>(
    a: &Inputs<'_, S>,
    s: &mut System<S>,
) -> Result<(), Failure> {
    let (mut index, mut index1) = (s.degree + 1, 2);
    for ii in a.parameters.lower()..=a.parameters.upper() {
        put(&mut s.parameters, index1, at(a.parameters, ii)?)?;
        put(&mut s.flatknots, index, at(a.parameters, ii)?)?;
        index += 1;
        index1 += 1;
    }
    // The poles from 3 on (the last one is the first point again, set later).
    for (index, ii) in (3..).zip(a.points.lower() + 1..=a.points.upper()) {
        put(&mut s.poles, index, at(a.points, ii)?)?;
    }
    Ok(())
}

// The periodic system with requested tangents: every flagged point after the first
// doubles its knot and adds a derivative condition.
fn fill_periodic_requested<S: InterpolateSpace>(
    a: &Inputs<'_, S>,
    s: &mut System<S>,
) -> Result<(), Failure> {
    let (mut mult_index, mut index, mut index1) = (2, 3, s.degree + 2);
    for ii in a.flags.lower() + 1..=a.flags.upper() {
        put(&mut s.parameters, index, at(a.parameters, ii)?)?;
        put(&mut s.flatknots, index1, at(a.parameters, ii)?)?;
        put(&mut s.poles, index, at(a.points, ii)?)?;
        index += 1;
        index1 += 1;
        if at(a.flags, ii)? {
            let mult: i32 = at(&s.mults, mult_index)?;
            put(&mut s.mults, mult_index, mult + 1)?;
            s.add_tangent(index, at(a.parameters, ii)?, at(a.tangents, ii)?)?;
            put(&mut s.flatknots, index1, at(a.parameters, ii)?)?;
            index += 1;
            index1 += 1;
        }
        mult_index += 1;
    }
    Ok(())
}

impl<S: InterpolateSpace> InterpolateGen<S> {
    /// `PerformNonPeriodic()`: degree 1 for two points and degree 2 for three
    /// without tangents, otherwise a cubic with end tangents (built by BuildTangents
    /// when missing) and the requested inner ones.
    pub(super) fn perform_non_periodic(&mut self) -> Result<(), Failure> {
        let h: InterpolateArrays<S> = self.arrays();
        let num_points: i32 = h.points()?.array1().len() as i32;
        let (degree, num_distinct_knots, mut num_poles) = match num_points {
            2 if !self.tangent_request() => (1, num_points, num_points),
            3 if !self.tangent_request() => (2, 2, num_points),
            _ => (3, num_points, num_points + 2),
        };
        if degree == 3 && self.tangent_request() {
            let flags = h.flags()?.array1();
            num_poles += count_flags(&flags, flags.lower() + 1, flags.upper() - 1)?;
        }
        let mut s: System<S> = System::new(num_poles, degree, num_distinct_knots)?;
        {
            let parameters = h.parameters()?.array1();
            for ii in 1..=degree + 1 {
                put(&mut s.flatknots, ii, at(&parameters, 1)?)?;
                put(
                    &mut s.flatknots,
                    ii + num_poles,
                    at(&parameters, num_points)?,
                )?;
            }
        }
        put(&mut s.mults, 1, degree + 1)?;
        put(&mut s.mults, num_distinct_knots, degree + 1)?;
        match degree {
            1 => self.perform_degree1(&h, &mut s),
            2 => self.perform_degree2(&h, &mut s),
            _ => self.perform_cubic(&h, &mut s, num_points),
        }
    }

    fn perform_degree1(
        &mut self,
        h: &InterpolateArrays<S>,
        s: &mut System<S>,
    ) -> Result<(), Failure> {
        let (points, parameters) = (h.points()?.array1(), h.parameters()?.array1());
        for ii in 1..=s.num_poles() {
            put(&mut s.poles, ii, at(&points, ii)?)?;
        }
        let curve = S::new_curve(&s.poles, &parameters, &s.mults, 1, false)?;
        drop((points, parameters));
        self.set_result(curve);
        Ok(())
    }

    fn perform_degree2(
        &mut self,
        h: &InterpolateArrays<S>,
        s: &mut System<S>,
    ) -> Result<(), Failure> {
        let (points, parameters) = (h.points()?.array1(), h.parameters()?.array1());
        let mut knots: Array1<'static, f64> = array(1, 2)?;
        put(&mut knots, 1, at(&parameters, 1)?)?;
        put(&mut knots, 2, at(&parameters, 3)?)?;
        for ii in 1..=s.num_poles() {
            put(&mut s.poles, ii, at(&points, ii)?)?;
        }
        if !s.interpolate(&parameters)? {
            return Ok(());
        }
        let curve = S::new_curve(&s.poles, &knots, &s.mults, 2, false)?;
        drop((points, parameters));
        self.set_result(curve);
        Ok(())
    }

    fn perform_cubic(
        &mut self,
        h: &InterpolateArrays<S>,
        s: &mut System<S>,
        num_points: i32,
    ) -> Result<(), Failure> {
        if num_points >= 3 {
            // the tangents of a cubic cannot be built from only 2 points if they
            // were not given in advance
            let (points, parameters) = (h.points()?.array1(), h.parameters()?.array1());
            build_tangents::<S>(
                &points,
                &mut h.tangents()?.change_array1(),
                &mut h.flags()?.change_array1(),
                &parameters,
            )?;
        }
        let (points, parameters) = (h.points()?.array1(), h.parameters()?.array1());
        let (tangents, flags) = (h.tangents()?.array1(), h.flags()?.array1());
        let a = Inputs {
            points: &points,
            parameters: &parameters,
            tangents: &tangents,
            flags: &flags,
        };
        self.fill_cubic(&a, s, num_points)?;
        let system_parameters: Array1<'static, f64> = s.parameters.clone();
        if !s.interpolate(&system_parameters)? {
            return Ok(());
        }
        let curve = S::new_curve(&s.poles, &parameters, &s.mults, 3, false)?;
        drop((points, parameters, tangents, flags));
        self.set_result(curve);
        Ok(())
    }

    fn fill_cubic(
        &self,
        a: &Inputs<'_, S>,
        s: &mut System<S>,
        num_points: i32,
    ) -> Result<(), Failure> {
        let num_poles: i32 = s.num_poles();
        s.add_tangent(2, at(a.parameters, 1)?, at(a.tangents, 1)?)?;
        put(&mut s.parameters, 1, at(a.parameters, 1)?)?;
        put(&mut s.poles, 1, at(a.points, 1)?)?;
        if self.tangent_request() {
            fill_cubic_requested(a, s)?;
        } else {
            fill_cubic_plain(a, s)?;
        }
        let p_upper: i32 = a.parameters.upper();
        s.add_tangent(
            num_poles - 1,
            at(a.parameters, p_upper)?,
            at(a.tangents, num_points)?,
        )?;
        put(&mut s.parameters, num_poles, at(a.parameters, p_upper)?)?;
        put(&mut s.poles, num_poles, at(a.points, num_points)?)
    }
}

// The cubic system with requested tangents: each flagged inner point doubles its
// knot and adds a derivative condition. The flag is read at a running index from 2
// and the tangent at the parameter index, as natively.
fn fill_cubic_requested<S: InterpolateSpace>(
    a: &Inputs<'_, S>,
    s: &mut System<S>,
) -> Result<(), Failure> {
    let (mut mult_index, mut index, mut index1) = (2, 3, 2);
    let (mut index2, mut index3) = (a.points.lower() + 1, s.degree + 2);
    for ii in a.parameters.lower() + 1..a.parameters.upper() {
        put(&mut s.parameters, index, at(a.parameters, ii)?)?;
        put(&mut s.poles, index, at(a.points, index2)?)?;
        put(&mut s.flatknots, index3, at(a.parameters, ii)?)?;
        index += 1;
        index3 += 1;
        if at(a.flags, index1)? {
            // set the multiplicities, the order of the contact, the flatknots
            let mult: i32 = at(&s.mults, mult_index)?;
            put(&mut s.mults, mult_index, mult + 1)?;
            s.add_tangent(index, at(a.parameters, ii)?, at(a.tangents, ii)?)?;
            put(&mut s.flatknots, index3, at(a.parameters, ii)?)?;
            index += 1;
            index3 += 1;
        }
        mult_index += 1;
        index1 += 1;
        index2 += 1;
    }
    Ok(())
}

// The cubic system without requested tangents: every parameter a knot and a
// condition, every inner point a pole.
fn fill_cubic_plain<S: InterpolateSpace>(
    a: &Inputs<'_, S>,
    s: &mut System<S>,
) -> Result<(), Failure> {
    for (index1, ii) in (2..).zip(a.parameters.lower()..=a.parameters.upper()) {
        put(&mut s.parameters, index1, at(a.parameters, ii)?)?;
    }
    for (index, ii) in (3..).zip(a.points.lower() + 1..=a.points.upper() - 1) {
        put(&mut s.poles, index, at(a.points, ii)?)?;
    }
    for (index, ii) in (s.degree + 1..).zip(a.parameters.lower()..=a.parameters.upper()) {
        put(&mut s.flatknots, index, at(a.parameters, ii)?)?;
    }
    Ok(())
}
