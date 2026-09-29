// Created on: 1997-04-29
// Created by: Stagiaire Francois DUMONT
// Copyright (c) 1997-1999 Matra Datavision
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
// Rust translation of ModelingData/TKGeomBase/Geom2dConvert/Geom2dConvert_CompCurveToBSplineCurve.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dConvert_CompCurveToBSplineCurve`: joins bounded curves end to end into one
//! B-spline.
use super::curve_to_bspline::curve_to_bspline_curve;
use super::{copy_bspline, live_bspline};
use crate::native_failure::failure;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dBoundedCurve, Geom2dCurve};
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::gp::Point2;

/// The composite-curve builder: a B-spline that grows as curves are added at either
/// end, with a C1 reparametrization of each junction when possible.
///
/// `let mut comp = CompCurveToBSplineCurve::with_curve(&arc, ParameterisationType::TgtThetaOver2)?;
/// comp.add(&next, 1e-7, false)?;`
#[derive(Clone)]
pub struct CompCurveToBSplineCurve {
    my_curve: Handle<Geom2dBSplineCurve>,
    my_tol: f64,
    my_type: ParameterisationType,
}

impl CompCurveToBSplineCurve {
    /// The empty algorithm; `parameterisation` converts the circles and ellipses added
    /// later (`Geom2dConvert_CompCurveToBSplineCurve(Parameterisation =
    /// Convert_TgtThetaOver2)`); `CompCurveToBSplineCurve::new(ParameterisationType::TgtThetaOver2)`.
    pub fn new(parameterisation: ParameterisationType) -> Self {
        Self {
            my_curve: Handle::default(),
            my_tol: precision::confusion(),
            my_type: parameterisation,
        }
    }

    /// The algorithm started with a copy of `basis_curve`, converted to a B-spline
    /// unless it is one (`Geom2dConvert_CompCurveToBSplineCurve(BasisCurve,
    /// Parameterisation = Convert_TgtThetaOver2)`);
    /// `CompCurveToBSplineCurve::with_curve(&arc, ParameterisationType::TgtThetaOver2)?`.
    pub fn with_curve(
        basis_curve: &Handle<dyn Geom2dBoundedCurve>,
        parameterisation: ParameterisationType,
    ) -> Result<Self, Failure> {
        let mut result: Self = Self::new(parameterisation);
        result.my_curve = to_bspline(basis_curve, parameterisation)?;
        Ok(result)
    }

    /// Appends `new_curve` at the end of the B-spline it touches within `tolerance`
    /// (reversing it when its other end touches), and returns false when neither of
    /// its ends touches either end of the B-spline. When both ends touch (the result
    /// would be closed), `after` chooses the end
    /// (`Add(NewCurve, Tolerance, After = false)`); `comp.add(&next, 1e-7, false)?`.
    pub fn add(
        &mut self,
        new_curve: &Handle<dyn Geom2dBoundedCurve>,
        tolerance: f64,
        after: bool,
    ) -> Result<bool, Failure> {
        let bs: Handle<Geom2dBSplineCurve> = to_bspline(new_curve, self.my_type)?;
        if self.my_curve.is_null() {
            self.my_curve = bs;
            return Ok(true);
        }
        self.my_tol = tolerance;
        let joint: Joint = Joint::find(
            live_bspline(&self.my_curve)?,
            live_bspline(&bs)?,
            tolerance,
            after,
        )?;
        if joint.is_after {
            if joint.is_after_reversed {
                bs.reverse()?;
            }
            let first: Handle<Geom2dBSplineCurve> = self.my_curve.clone();
            self.concat(&first, &bs, true)?;
            return Ok(true);
        }
        if joint.is_before {
            if joint.is_before_reversed {
                bs.reverse()?;
            }
            let second: Handle<Geom2dBSplineCurve> = self.my_curve.clone();
            self.concat(&bs, &second, false)?;
            return Ok(true);
        }
        Ok(false)
    }

    /// The composite B-spline, null before the first curve (`BSplineCurve()`);
    /// `comp.bspline_curve()`.
    pub fn bspline_curve(&self) -> Handle<Geom2dBSplineCurve> {
        self.my_curve.clone()
    }

    /// Forgets the composite B-spline (`Clear()`); `comp.clear()`.
    pub fn clear(&mut self) {
        self.my_curve.nullify();
    }

    /// The private `Add(FirstCurve, SecondCurve, After)`: the concatenation of two
    /// B-splines at a common knot, reparametrizing the moving one by the ratio of
    /// the junction speeds (C1 when possible), then removing the junction knot as
    /// far as `my_tol` allows. Both curves are raised to the higher degree in place.
    fn concat(
        &mut self,
        first_curve: &Handle<Geom2dBSplineCurve>,
        second_curve: &Handle<Geom2dBSplineCurve>,
        after: bool,
    ) -> Result<(), Failure> {
        let (first, second) = (live_bspline(first_curve)?, live_bspline(second_curve)?);
        let deg: i32 = first.degree().max(second.degree());
        if first.degree() < deg {
            first.increase_degree(deg)?;
        }
        if second.degree() < deg {
            second.increase_degree(deg)?;
        }
        let shift: JunctionShift = JunctionShift::new(first, second, after)?;
        let (knots, mults) = joined_knots(first, second, &shift)?;
        let (poles, weights) = joined_poles(first, second)?;
        let junction: i32 = first.nb_knots();
        self.my_curve = Handle::new(Geom2dBSplineCurve::with_weights(
            &poles, &weights, &knots, &mults, deg, false,
        )?);
        self.reduce_junction(junction, mults[junction])
    }

    // Optionally reduce the multiplicity of the junction knot.
    fn reduce_junction(&self, junction: i32, mult: i32) -> Result<(), Failure> {
        let curve: &Geom2dBSplineCurve = live_bspline(&self.my_curve)?;
        let (mut m, mut ok): (i32, bool) = (mult, true);
        while m > 0 && ok {
            m -= 1;
            ok = curve.remove_knot(junction, m, self.my_tol)?;
        }
        Ok(())
    }
}

// The conversion of an added curve: a copy of a B-spline, CurveToBSplineCurve of
// anything else.
fn to_bspline(
    curve: &Handle<dyn Geom2dBoundedCurve>,
    parameterisation: ParameterisationType,
) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
    if curve.is_null() {
        return Err(failure(
            "Standard_NullObject",
            "Geom2dConvert_CompCurveToBSplineCurve: null curve; expected a Geom2d_BoundedCurve",
        ));
    }
    match curve.cast::<Geom2dBSplineCurve>().get() {
        Some(bspline) => copy_bspline(bspline),
        None => curve_to_bspline_curve(&curve.cast::<dyn Geom2dCurve>(), parameterisation),
    }
}

// Where an added curve touches the composite curve, from the end poles.
struct Joint {
    is_before: bool,
    is_before_reversed: bool,
    is_after: bool,
    is_after_reversed: bool,
}

impl Joint {
    fn find(
        my_curve: &Geom2dBSplineCurve,
        bs: &Geom2dBSplineCurve,
        tolerance: f64,
        after: bool,
    ) -> Result<Self, Failure> {
        let sq_tol: f64 = tolerance * tolerance;
        let (lbs, lcb): (i32, i32) = (bs.nb_poles(), my_curve.nb_poles());
        let [head, tail]: [Point2; 2] = [my_curve.pole(1)?, my_curve.pole(lcb)?];
        let [start, end]: [Point2; 2] = [bs.pole(1)?, bs.pole(lbs)?];
        let (d1, d2) = (head.square_distance(start), head.square_distance(end));
        let is_before_reversed: bool = head.square_distance(start) < sq_tol && d1 < d2;
        let mut is_before: bool = head.square_distance(end) < sq_tol || is_before_reversed;
        let (d1, d2) = (tail.square_distance(start), tail.square_distance(end));
        let is_after_reversed: bool = tail.square_distance(end) < sq_tol && d2 < d1;
        let mut is_after: bool = tail.square_distance(start) < sq_tol || is_after_reversed;
        // Both ends touch: the two curves form a closed curve.
        if is_before && is_after {
            if after {
                is_before = false;
            } else {
                is_after = false;
            }
        }
        Ok(Self {
            is_before,
            is_before_reversed,
            is_after,
            is_after_reversed,
        })
    }
}

// The affine reparametrizations `Ratio * u - Delta` of the two curves and the
// parameter of their junction; the fixed curve keeps its parameters.
struct JunctionShift {
    ratio1: f64,
    delta1: f64,
    ratio2: f64,
    delta2: f64,
    u_de_raccord: f64,
}

impl JunctionShift {
    fn new(
        first: &Geom2dBSplineCurve,
        second: &Geom2dBSplineCurve,
        after: bool,
    ) -> Result<Self, Failure> {
        let ratio: f64 = speed_ratio(first, second)?;
        let nb_k1: i32 = first.nb_knots();
        if after {
            // Do not move the first curve.
            let ratio2: f64 = 1.0 / ratio;
            return Ok(Self {
                ratio1: 1.0,
                delta1: 0.0,
                ratio2,
                delta2: ratio2 * second.knot(1)? - first.knot(nb_k1)?,
                u_de_raccord: first.last_parameter(),
            });
        }
        // Do not move the second curve.
        Ok(Self {
            ratio1: ratio,
            delta1: ratio * first.knot(nb_k1)? - second.knot(1)?,
            ratio2: 1.0,
            delta2: 0.0,
            u_de_raccord: second.first_parameter(),
        })
    }
}

// The reparametrization ratio (C1 if possible): the ratio of the junction speeds,
// 1 when a speed is below Precision::Confusion() or the ratio is out of range.
fn speed_ratio(first: &Geom2dBSplineCurve, second: &Geom2dBSplineCurve) -> Result<f64, Failure> {
    let l1: f64 = first.dn(first.last_parameter(), 1)?.magnitude();
    let l2: f64 = second.dn(second.first_parameter(), 1)?.magnitude();
    let mut ratio: f64 = 1.0;
    if l1 > precision::confusion() && l2 > precision::confusion() {
        ratio = l1 / l2;
    }
    if ratio < precision::confusion() || ratio > 1.0 / precision::confusion() {
        ratio = 1.0;
    }
    Ok(ratio)
}

// The knots of the first curve but the last, the junction knot of multiplicity
// Degree, then the knots of the second curve but the first.
fn joined_knots(
    first: &Geom2dBSplineCurve,
    second: &Geom2dBSplineCurve,
    shift: &JunctionShift,
) -> Result<(Array1<'static, f64>, Array1<'static, i32>), Failure> {
    let (nb_k1, nb_k2): (i32, i32) = (first.nb_knots(), second.nb_knots());
    let mut noeuds: Array1<'static, f64> = one_based(nb_k1 + nb_k2 - 1)?;
    let mut mults: Array1<'static, i32> = one_based(nb_k1 + nb_k2 - 1)?;
    for ii in 1..nb_k1 {
        noeuds[ii] = shift.ratio1 * first.knot(ii)? - shift.delta1;
        mults[ii] = first.multiplicity(ii)?;
    }
    noeuds[nb_k1] = shift.u_de_raccord;
    mults[nb_k1] = first.degree();
    for (ii, jj) in (2..=nb_k2).zip(nb_k1 + 1..) {
        noeuds[jj] = shift.ratio2 * second.knot(ii)? - shift.delta2;
        mults[jj] = second.multiplicity(ii)?;
    }
    Ok((noeuds, mults))
}

// The poles of both curves sharing the junction pole of the second one; the second
// curve's weights are scaled so that they meet the first one's (the weights may not
// connect with C0 continuity).
fn joined_poles(
    first: &Geom2dBSplineCurve,
    second: &Geom2dBSplineCurve,
) -> Result<(Array1<'static, Point2>, Array1<'static, f64>), Failure> {
    let (nb_p1, nb_p2): (i32, i32) = (first.nb_poles(), second.nb_poles());
    let mut poles: Array1<'static, Point2> = one_based(nb_p1 + nb_p2 - 1)?;
    let mut poids: Array1<'static, f64> = one_based(nb_p1 + nb_p2 - 1)?;
    let mut ratio: f64 = first.weight(nb_p1)?;
    ratio /= second.weight(1)?;
    for ii in 1..nb_p1 {
        poles[ii] = first.pole(ii)?;
        poids[ii] = first.weight(ii)?;
    }
    for (ii, jj) in (1..=nb_p2).zip(nb_p1..) {
        poles[jj] = second.pole(ii)?;
        poids[jj] = ratio * second.weight(ii)?;
    }
    Ok((poles, poids))
}
