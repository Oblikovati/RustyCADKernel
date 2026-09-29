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
// Rust translation of ModelingData/TKGeomBase/GeomConvert/GeomConvert_CompCurveToBSplineCurve.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomConvert_CompCurveToBSplineCurve`: joins bounded curves end to end into one
//! B-spline.
use super::curve_to_bspline::curve_to_bspline_curve;
use super::{copy_bspline, live_bspline};
use crate::native_failure::failure;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomBoundedCurve, GeomCurve};
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::gp::Point3;

/// The composite-curve builder: a B-spline that grows as curves are added at either
/// end, with a C1 reparametrization of each junction when possible (WithRatio) and
/// an optional reduction of the junction multiplicity (MinM).
///
/// `let mut comp = CompCurveToBSplineCurve::with_curve(&arc, ParameterisationType::TgtThetaOver2)?;
/// comp.add(&next, 1e-7, false, true, 0)?;`
#[derive(Clone)]
pub struct CompCurveToBSplineCurve {
    my_curve: Handle<GeomBSplineCurve>,
    my_tol: f64,
    my_type: ParameterisationType,
}

impl CompCurveToBSplineCurve {
    /// The empty algorithm; `parameterisation` converts the circles and ellipses added
    /// later (`GeomConvert_CompCurveToBSplineCurve(Parameterisation =
    /// Convert_TgtThetaOver2)`); `CompCurveToBSplineCurve::new(ParameterisationType::TgtThetaOver2)`.
    pub fn new(parameterisation: ParameterisationType) -> Self {
        Self {
            my_curve: Handle::default(),
            my_tol: precision::confusion(),
            my_type: parameterisation,
        }
    }

    /// The algorithm started with a copy of `basis_curve`, converted to a B-spline
    /// unless it is one (`GeomConvert_CompCurveToBSplineCurve(BasisCurve,
    /// Parameterisation = Convert_TgtThetaOver2)`);
    /// `CompCurveToBSplineCurve::with_curve(&arc, ParameterisationType::TgtThetaOver2)?`.
    pub fn with_curve(
        basis_curve: &Handle<dyn GeomBoundedCurve>,
        parameterisation: ParameterisationType,
    ) -> Result<Self, Failure> {
        let mut result: Self = Self::new(parameterisation);
        result.my_curve = to_bspline(basis_curve, parameterisation)?;
        Ok(result)
    }

    /// Appends `new_curve` at the end of the B-spline whose end point it touches
    /// within `tolerance` (reversing it when needed), and returns false when neither
    /// of its end points touches either end point of the B-spline. The end points are
    /// the curves' start and end points, not their end poles (bug 0030007). When both
    /// ends touch (the result would be closed), `after` chooses the end. `with_ratio`
    /// reparametrizes the moving curve so that the junction is C1 when possible;
    /// `min_m` is the multiplicity the junction knot is reduced to within the
    /// tolerance (`Add(NewCurve, Tolerance, After = false, WithRatio = true, MinM = 0)`);
    /// `comp.add(&next, 1e-7, false, true, 0)?`.
    pub fn add(
        &mut self,
        new_curve: &Handle<dyn GeomBoundedCurve>,
        tolerance: f64,
        after: bool,
        with_ratio: bool,
        min_m: i32,
    ) -> Result<bool, Failure> {
        let bs: Handle<GeomBSplineCurve> = to_bspline(new_curve, self.my_type)?;
        if self.my_curve.is_null() {
            self.my_curve = bs;
            return Ok(true);
        }
        self.my_tol = tolerance;
        let joint = Joint::find(
            live_bspline(&self.my_curve)?,
            live_bspline(&bs)?,
            tolerance,
            after,
        )?;
        let options = JoinOptions { with_ratio, min_m };
        if joint.apres {
            if joint.apres_reversed {
                live_bspline(&bs)?.reverse()?;
            }
            let first: Handle<GeomBSplineCurve> = self.my_curve.clone();
            self.concat(&first, &bs, true, options)?;
            return Ok(true);
        }
        if joint.avant {
            if joint.avant_reversed {
                live_bspline(&bs)?.reverse()?;
            }
            let second: Handle<GeomBSplineCurve> = self.my_curve.clone();
            self.concat(&bs, &second, false, options)?;
            return Ok(true);
        }
        Ok(false)
    }

    /// The composite B-spline, null before the first curve (`BSplineCurve()`);
    /// `comp.bspline_curve()`.
    pub fn bspline_curve(&self) -> Handle<GeomBSplineCurve> {
        self.my_curve.clone()
    }

    /// Forgets the composite B-spline (`Clear()`); `comp.clear()`.
    pub fn clear(&mut self) {
        self.my_curve.nullify();
    }

    /// The private `Add(FirstCurve, SecondCurve, After, WithRatio, MinM)`: the
    /// concatenation of two B-splines at a common knot. Both curves are raised to the
    /// higher degree in place.
    fn concat(
        &mut self,
        first_curve: &Handle<GeomBSplineCurve>,
        second_curve: &Handle<GeomBSplineCurve>,
        after: bool,
        options: JoinOptions,
    ) -> Result<(), Failure> {
        let (first, second) = (live_bspline(first_curve)?, live_bspline(second_curve)?);
        let deg: i32 = first.degree().max(second.degree());
        if first.degree() < deg {
            first.increase_degree(deg)?;
        }
        if second.degree() < deg {
            second.increase_degree(deg)?;
        }
        let ratio: f64 = if options.with_ratio {
            speed_ratio(first, second)?
        } else {
            1.0
        };
        let shift: JunctionShift = JunctionShift::new(first, second, after, ratio)?;
        let (noeuds, mults) = joined_knots(first, second, &shift)?;
        let (poles, poids) = joined_poles(first, second)?;
        let junction: i32 = first.nb_knots();
        self.my_curve = Handle::new(GeomBSplineCurve::with_weights(
            &poles, &poids, &noeuds, &mults, deg, false,
        )?);
        self.reduce_junction(junction, mults[junction], options.min_m)
    }

    // Optionally reduce the multiplicity of the junction knot down to MinM.
    fn reduce_junction(&self, junction: i32, mult: i32, min_m: i32) -> Result<(), Failure> {
        let curve: &GeomBSplineCurve = live_bspline(&self.my_curve)?;
        let (mut m, mut ok): (i32, bool) = (mult, true);
        while m > min_m && ok {
            m -= 1;
            ok = curve.remove_knot(junction, m, self.my_tol)?;
        }
        Ok(())
    }
}

// The WithRatio and MinM arguments of Add.
#[derive(Clone, Copy)]
struct JoinOptions {
    with_ratio: bool,
    min_m: i32,
}

// The conversion of an added curve: a copy of a B-spline, CurveToBSplineCurve of
// anything else.
fn to_bspline(
    curve: &Handle<dyn GeomBoundedCurve>,
    parameterisation: ParameterisationType,
) -> Result<Handle<GeomBSplineCurve>, Failure> {
    if curve.is_null() {
        return Err(failure(
            "Standard_NullObject",
            "GeomConvert_CompCurveToBSplineCurve: null curve; expected a Geom_BoundedCurve",
        ));
    }
    match curve.cast::<GeomBSplineCurve>().get() {
        Some(bspline) => copy_bspline(bspline),
        None => curve_to_bspline_curve(&curve.cast::<dyn GeomCurve>(), parameterisation),
    }
}

// Where an added curve touches the composite curve, from the curve end points.
struct Joint {
    avant: bool,
    avant_reversed: bool,
    apres: bool,
    apres_reversed: bool,
}

impl Joint {
    fn find(
        my_curve: &GeomBSplineCurve,
        bs: &GeomBSplineCurve,
        tol: f64,
        after: bool,
    ) -> Result<Self, Failure> {
        let (curve_start, curve_end): (Point3, Point3) =
            (my_curve.start_point()?, my_curve.end_point()?);
        let (bs_start, bs_end): (Point3, Point3) = (bs.start_point()?, bs.end_point()?);
        let mut avant: bool =
            curve_start.distance(&bs_start) < tol || curve_start.distance(&bs_end) < tol;
        let mut apres: bool =
            curve_end.distance(&bs_start) < tol || curve_end.distance(&bs_end) < tol;
        // Will myCurve be (or become) closed? Resolve the ambiguity.
        if avant && apres {
            if after {
                avant = false;
            } else {
                apres = false;
            }
        }
        Ok(Self {
            avant,
            avant_reversed: curve_start.distance(&bs_start) < tol,
            apres,
            apres_reversed: curve_end.distance(&bs_end) < tol,
        })
    }
}

// The affine reparametrizations `Ratio * u - Delta` of the two curves; the fixed
// curve keeps its parameters.
struct JunctionShift {
    ratio1: f64,
    delta1: f64,
    ratio2: f64,
    delta2: f64,
}

impl JunctionShift {
    fn new(
        first: &GeomBSplineCurve,
        second: &GeomBSplineCurve,
        after: bool,
        ratio: f64,
    ) -> Result<Self, Failure> {
        let nb_k1: i32 = first.nb_knots();
        if after {
            // Do not move the first curve.
            let ratio2: f64 = 1.0 / ratio;
            return Ok(Self {
                ratio1: 1.0,
                delta1: 0.0,
                ratio2,
                delta2: ratio2 * second.knot(1)? - first.knot(nb_k1)?,
            });
        }
        // Do not move the second curve.
        Ok(Self {
            ratio1: ratio,
            delta1: ratio * first.knot(nb_k1)? - second.knot(1)?,
            ratio2: 1.0,
            delta2: 0.0,
        })
    }
}

// The reparametrization ratio (C1 if possible): the ratio of the junction speeds,
// 1 when a speed is below Precision::Confusion() or the ratio is out of range.
fn speed_ratio(first: &GeomBSplineCurve, second: &GeomBSplineCurve) -> Result<f64, Failure> {
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

// The knots of both curves sharing the junction knot (of multiplicity Degree); each
// knot not above its predecessor by more than max(Epsilon, 5e-10) is pushed up.
fn joined_knots(
    first: &GeomBSplineCurve,
    second: &GeomBSplineCurve,
    shift: &JunctionShift,
) -> Result<(Array1<'static, f64>, Array1<'static, i32>), Failure> {
    let (nb_k1, nb_k2): (i32, i32) = (first.nb_knots(), second.nb_knots());
    let mut noeuds: Array1<'static, f64> = one_based(nb_k1 + nb_k2 - 1)?;
    let mut mults: Array1<'static, i32> = one_based(nb_k1 + nb_k2 - 1)?;
    for ii in 1..=nb_k1 {
        noeuds[ii] = shift.ratio1 * first.knot(ii)? - shift.delta1;
        if ii > 1 {
            separate_knot(&mut noeuds, ii);
        }
        mults[ii] = first.multiplicity(ii)?;
    }
    mults[nb_k1] = first.degree();
    for (ii, jj) in (2..=nb_k2).zip(nb_k1 + 1..) {
        noeuds[jj] = shift.ratio2 * second.knot(ii)? - shift.delta2;
        separate_knot(&mut noeuds, jj);
        mults[jj] = second.multiplicity(ii)?;
    }
    Ok((noeuds, mults))
}

// Noeuds(i) += eps when it does not exceed Noeuds(i - 1) by more than eps, with
// eps = max(Epsilon(|Noeuds(i - 1)|), 5e-10).
fn separate_knot(noeuds: &mut Array1<'static, f64>, i: i32) {
    let mut eps: f64 = epsilon(noeuds[i - 1].abs());
    if eps < 5.0e-10 {
        eps = 5.0e-10;
    }
    if noeuds[i] - noeuds[i - 1] <= eps {
        noeuds[i] += eps;
    }
}

// The poles of both curves sharing the junction pole of the second one; the second
// curve's weights are scaled so that they meet the first one's (the weights may not
// connect with C0 continuity).
fn joined_poles(
    first: &GeomBSplineCurve,
    second: &GeomBSplineCurve,
) -> Result<(Array1<'static, Point3>, Array1<'static, f64>), Failure> {
    let (nb_p1, nb_p2): (i32, i32) = (first.nb_poles(), second.nb_poles());
    let mut poles: Array1<'static, Point3> = one_based(nb_p1 + nb_p2 - 1)?;
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
