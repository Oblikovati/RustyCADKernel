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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/Law/Law_BSpline.cxx (Reverse, Segment,
// the periodicity members and PeriodicNormalization) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The reversal, segmentation and periodicity members of `Law_BSpline`.
use super::{LawBSpline, LawBSplineData, split_weighted};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, real};
use rustycad_tkg2d::geom2d::arrays::{at, one_based, put};
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkg2d::geom2d::spline_data::{rotated, slice};
use rustycad_tkmath::bsplclib;

impl LawBSplineData {
    /// `PeriodicNormalization`: shifts a periodic parameter by whole periods into
    /// `[FlatKnots(Degree+1), FlatKnots(Upper-Degree)]`, one period at a time.
    pub(super) fn periodic_normalization(&self, parameter: f64) -> f64 {
        if !self.periodic {
            return parameter;
        }
        let flat: &[f64] = self.flatknots.as_slice();
        let (low, high) = (
            flat[self.deg as usize],
            flat[flat.len() - 1 - self.deg as usize],
        );
        let period: f64 = high - low;
        let mut u: f64 = parameter;
        while u > high {
            u -= period;
        }
        while u < low {
            u += period;
        }
        u
    }

    /// `LocateParameter` on the distinct knots over their whole range.
    fn locate(&self, u: f64) -> Result<(i32, f64), Failure> {
        let (mut index, mut new_u): (i32, f64) = (0, 0.0);
        bsplclib::locate_parameter(
            self.deg,
            &self.knots,
            &self.mults,
            u,
            self.periodic,
            self.knots.lower(),
            self.knots.upper(),
            &mut index,
            &mut new_u,
        )?;
        Ok((index, new_u))
    }

    fn reverse(&mut self) -> Result<(), Failure> {
        bsplclib::reverse_knots(&mut self.knots)?;
        bsplclib::reverse_mults(&mut self.mults);
        let last: i32 = if self.periodic {
            self.flatknots.upper() - self.deg - 1
        } else {
            self.poles.upper()
        };
        bsplclib::reverse_poles(&mut self.poles, last)?;
        if self.rational
            && let Some(weights) = self.weights.as_mut()
        {
            bsplclib::reverse_poles(weights, last)?;
        }
        self.update_knots()
    }

    fn segment(&mut self, u1: f64, u2: f64) -> Result<(), Failure> {
        if u2 < u1 {
            return Err(errors::domain(&format!(
                "Law_BSpline::Segment: U1 = {u1}, U2 = {u2}; expected U1 <= U2"
            )));
        }
        let eps: f64 = real::epsilon(u1.abs().max(u2.abs()));
        let delta: f64 = u2 - u1;
        let (_, new_u1) = self.locate(u1)?;
        let (_, new_u2) = self.locate(u2)?;
        let (mut knots, mut mults) = (one_based::<f64>(2)?, one_based::<i32>(2)?);
        put(&mut knots, 1, new_u1.min(new_u2))?;
        put(&mut knots, 2, new_u1.max(new_u2))?;
        mults.init(self.deg);
        self.insert_knots(&knots, &mults, eps, false)?;
        if self.periodic {
            // Set the origin at NewU1.
            let (mut index0, u) = self.locate(u1)?;
            if (at(&self.knots, index0 + 1)? - u).abs() < eps {
                index0 += 1;
            }
            self.set_origin(index0)?;
            self.set_not_periodic()?;
        }
        self.keep_segment(new_u1, new_u1 + delta, eps)
    }

    /// The last step of `Segment`: keeps the knots and poles between `from` and `to`,
    /// with end multiplicities `Degree + 1`.
    fn keep_segment(&mut self, from: f64, to: f64, eps: f64) -> Result<(), Failure> {
        let (index1, _) = self.locate(from)?;
        let (mut index2, u) = self.locate(to)?;
        if (at(&self.knots, index2 + 1)? - u).abs() < eps {
            index2 += 1;
        }
        let nb_knots: i32 = index2 - index1 + 1;
        let knots: Array1<'static, f64> = slice(&self.knots, index1, index2)?;
        let mut mults: Array1<'static, i32> = slice(&self.mults, index1, index2)?;
        put(&mut mults, 1, self.deg + 1)?;
        put(&mut mults, nb_knots, self.deg + 1)?;
        let pindex1: i32 = bsplclib::pole_index(self.deg, index1, self.periodic, &self.mults)? + 1;
        let pindex2: i32 = (bsplclib::pole_index(self.deg, index2, self.periodic, &self.mults)?
            + 1)
        .min(self.poles.len() as i32);
        let poles: Array1<'static, f64> = slice(&self.poles, pindex1, pindex2)?;
        if self.rational
            && let Some(weights) = self.weights.as_ref()
        {
            self.weights = Some(slice(weights, pindex1, pindex2)?);
        }
        self.knots = knots;
        self.mults = mults;
        self.poles = poles;
        self.update_knots()
    }

    fn set_periodic(&mut self) -> Result<(), Failure> {
        let (first, last) = (self.first_u_knot_index()?, self.last_u_knot_index()?);
        let knots: Array1<'static, f64> = slice(&self.knots, first, last)?;
        let mut mults: Array1<'static, i32> = slice(&self.mults, first, last)?;
        let count: i32 = mults.len() as i32;
        let end_mult: i32 = at(&mults, 1)?.max(at(&mults, count)?);
        put(&mut mults, 1, end_mult)?;
        put(&mut mults, count, end_mult)?;
        let nbp: i32 = bsplclib::nb_poles(self.deg, true, &mults)?;
        // The native array views poles(1) with nbp values; reading past the poles is
        // undefined natively and Standard_OutOfRange here.
        self.poles = slice(&self.poles, 1, nbp)?;
        if self.rational
            && let Some(weights) = self.weights.as_ref()
        {
            self.weights = Some(slice(weights, 1, nbp)?);
        }
        self.knots = knots;
        self.mults = mults;
        self.periodic = true;
        self.update_knots()
    }

    fn set_origin(&mut self, index: i32) -> Result<(), Failure> {
        if !self.periodic {
            return Err(errors::no_such_object(
                "Law_BSpline::SetOrigin: the law is not periodic; expected a periodic law",
            ));
        }
        let (first, last) = (self.first_u_knot_index()?, self.last_u_knot_index()?);
        if index < first || index > last {
            return Err(errors::domain(&format!(
                "Law_BSpline::SetOrigin: Index = {index}; expected in [{first}, {last}]"
            )));
        }
        let period: f64 = at(&self.knots, last)? - at(&self.knots, first)?;
        let mut knots: Array1<'static, f64> = one_based(self.knots.len() as i32)?;
        let mut mults: Array1<'static, i32> = one_based(self.knots.len() as i32)?;
        // The knots from Index keep their values; the ones before it move one period.
        let tail = (index..=last).map(|i| (i, None));
        let head = (first + 1..=index).map(|i| (i, Some(period)));
        for (k, (i, shift)) in (1..).zip(tail.chain(head)) {
            let knot: f64 = at(&self.knots, i)?;
            put(&mut knots, k, shift.map_or(knot, |period| knot + period))?;
            put(&mut mults, k, at(&self.mults, i)?)?;
        }
        let mut pole_origin: i32 = 1;
        for i in first + 1..=index {
            pole_origin += at(&self.mults, i)?;
        }
        self.poles = rotated(&self.poles, pole_origin)?;
        if self.rational
            && let Some(weights) = self.weights.as_ref()
        {
            self.weights = Some(rotated(weights, pole_origin)?);
        }
        self.knots = knots;
        self.mults = mults;
        self.update_knots()
    }

    fn set_not_periodic(&mut self) -> Result<(), Failure> {
        if !self.periodic {
            return Ok(());
        }
        let (nb_knots, nb_poles) = bsplclib::prepare_unperiodize(self.deg, &self.mults)?;
        let mut knots: Array1<'static, f64> = one_based(nb_knots)?;
        let mut mults: Array1<'static, i32> = one_based(nb_knots)?;
        let weighted: Option<Array1<'static, f64>> = self.weighted_if_any()?;
        let input: &Array1<'static, f64> = weighted.as_ref().unwrap_or(&self.poles);
        let dimension: i32 = if weighted.is_some() { 2 } else { 1 };
        let mut output: Array1<'static, f64> = one_based(nb_poles * dimension)?;
        // The native call passes Dimension 1 for the weighted pairs too; BSplCLib
        // ignores it and copies the flat array cyclically, which keeps the pairs.
        bsplclib::unperiodize_flat(
            self.deg,
            1,
            &self.mults,
            &self.knots,
            input,
            &mut mults,
            &mut knots,
            &mut output,
        )?;
        if weighted.is_some() {
            let (mut poles, mut weights) = (one_based(nb_poles)?, one_based(nb_poles)?);
            split_weighted(&output, &mut poles, &mut weights)?;
            self.poles = poles;
            self.weights = Some(weights);
        } else {
            self.poles = output;
            self.weights = None;
        }
        self.knots = knots;
        self.mults = mults;
        self.periodic = false;
        self.update_knots()
    }

    /// The weighted pairs of a law with weights (`IsRational`), or `None`.
    fn weighted_if_any(&self) -> Result<Option<Array1<'static, f64>>, Failure> {
        match self.weights.as_ref() {
            Some(weights) => Ok(Some(super::weighted_poles(&self.poles, weights)?)),
            None => Ok(None),
        }
    }
}

impl LawBSpline {
    /// Reverses the orientation (`Reverse`); `law.reverse()?`.
    pub fn reverse(&self) -> Result<(), Failure> {
        self.state.write().reverse()
    }

    /// The parameter of the reversed law that matches `u`: `First + Last - U`
    /// (`ReversedParameter`); `law.reversed_parameter(0.25)`.
    pub fn reversed_parameter(&self, u: f64) -> f64 {
        let data = self.state.read();
        let flat: &[f64] = data.flatknots.as_slice();
        let (first, last) = (
            flat[data.deg as usize],
            flat[flat.len() - 1 - data.deg as usize],
        );
        first + last - u
    }

    /// Keeps the part between `u1` and `u2` (`Segment`); Standard_DomainError when
    /// `u2 < u1`. A periodic law becomes non-periodic. `law.segment(0.25, 0.75)?`.
    pub fn segment(&self, u1: f64, u2: f64) -> Result<(), Failure> {
        self.state.write().segment(u1, u2)
    }

    /// Makes the law periodic on its bound knots (`SetPeriodic`); `law.set_periodic()?`.
    pub fn set_periodic(&self) -> Result<(), Failure> {
        self.state.write().set_periodic()
    }

    /// Moves the origin of a periodic law to knot `index` (`SetOrigin`);
    /// Standard_NoSuchObject for a non-periodic law, Standard_DomainError for an index
    /// outside the knots. `law.set_origin(2)?`.
    pub fn set_origin(&self, index: i32) -> Result<(), Failure> {
        self.state.write().set_origin(index)
    }

    /// Makes a periodic law non-periodic without changing its shape (`SetNotPeriodic`);
    /// `law.set_not_periodic()?`.
    pub fn set_not_periodic(&self) -> Result<(), Failure> {
        self.state.write().set_not_periodic()
    }

    /// `U` shifted by whole periods into the parameter range of a periodic law
    /// (`PeriodicNormalization`); `law.periodic_normalization(7.0)`.
    pub fn periodic_normalization(&self, u: f64) -> f64 {
        self.state.read().periodic_normalization(u)
    }
}
