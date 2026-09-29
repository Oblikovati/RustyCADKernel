// Copyright (c) 1992-1999 Matra Datavision
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
// Rust translation of IntCurve_IntCurveCurveGen.gxx/.lxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntCurve_IntCurveCurveGen`: the intersection of two curves of any type, or of
//! a curve with itself, dispatched on the curve types and split per continuity
//! interval of composite curves.
use super::int_poly_poly_zone::CurveOf;
use super::user_int_conic_curve_gen::merge_result;
use super::{GpConic, IConicTool, IntConicConic, IntConicCurveGen, IntPolyPolyGen, PCurveTool};
use crate::int_imp_par_gen::{IntersectorTypes, ParTool};
use crate::int_res2d::{Domain, Intersection};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use std::f64::consts::PI;
use std::ops::{Deref, DerefMut};

/// `IntCurve_IntCurveCurveGen` (`Geom2dInt_GInter`): an [`Intersection`] reached
/// through `Deref`. Two conics go to [`IntConicConic`] (in its pair order, the
/// parameters swapped when the second kind comes first), a conic and another curve
/// to [`IntConicCurveGen`], two other curves to [`IntPolyPolyGen`]; composite
/// curves are intersected per pair of continuity intervals and the results merged.
/// The self intersection of a conic is empty.
/// `let inter = GInter::between(&a1, &a2, 1e-9, 1e-9)?;`.
pub struct IntCurveCurveGen<T: IntersectorTypes<Imp = IConicTool>>
where
    T::Par: PCurveTool,
{
    base: Intersection,
    param1inf: f64,
    param1sup: f64,
    param2inf: f64,
    param2sup: f64,
    intconiconi: IntConicConic,
    intconicurv: IntConicCurveGen<T>,
    intcurvcurv: IntPolyPolyGen<T>,
}

impl<T: IntersectorTypes<Imp = IConicTool>> Default for IntCurveCurveGen<T>
where
    T::Par: PCurveTool,
{
    /// `IntCurve_IntCurveCurveGen()`: not done, the composite bounds infinite.
    fn default() -> Self {
        Self {
            base: Intersection::new(),
            param1inf: -precision::infinite(),
            param1sup: precision::infinite(),
            param2inf: -precision::infinite(),
            param2sup: precision::infinite(),
            intconiconi: IntConicConic::new(),
            intconicurv: IntConicCurveGen::new(),
            intcurvcurv: IntPolyPolyGen::new(),
        }
    }
}

impl<T: IntersectorTypes<Imp = IConicTool>> Deref for IntCurveCurveGen<T>
where
    T::Par: PCurveTool,
{
    type Target = Intersection;
    fn deref(&self) -> &Intersection {
        &self.base
    }
}

impl<T: IntersectorTypes<Imp = IConicTool>> DerefMut for IntCurveCurveGen<T>
where
    T::Par: PCurveTool,
{
    fn deref_mut(&mut self) -> &mut Intersection {
        &mut self.base
    }
}

/// One curve of the composite split: the curve, its domain, its interval count
/// and its interval bounds.
type CompositeCurve<'c, T> = (&'c CurveOf<T>, &'c Domain, i32, &'c Array1<'static, f64>);

impl<T: IntersectorTypes<Imp = IConicTool>> IntCurveCurveGen<T>
where
    T::Par: PCurveTool,
{
    /// `IntCurve_IntCurveCurveGen()`. `GInter::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `IntCurve_IntCurveCurveGen(C, TolConf, Tol)`: the self intersection.
    /// `GInter::of_self(&adaptor, 1e-9, 1e-9)?`.
    pub fn of_self(c: &CurveOf<T>, tol_conf: f64, tol: f64) -> Result<Self, Failure> {
        let mut inter: Self = Self::new();
        inter.perform_self(c, tol_conf, tol)?;
        Ok(inter)
    }

    /// `IntCurve_IntCurveCurveGen(C, D, TolConf, Tol)`: the self intersection on `d`.
    /// `GInter::of_self_on(&adaptor, &domain, 1e-9, 1e-9)?`.
    pub fn of_self_on(
        c: &CurveOf<T>,
        d: &Domain,
        tol_conf: f64,
        tol: f64,
    ) -> Result<Self, Failure> {
        let mut inter: Self = Self::new();
        inter.perform_self_on(c, d, tol_conf, tol)?;
        Ok(inter)
    }

    /// `IntCurve_IntCurveCurveGen(C1, C2, TolConf, Tol)`: on the domains of both
    /// curves ([`IntCurveCurveGen::compute_domain`]).
    /// `GInter::between(&a1, &a2, 1e-9, 1e-9)?`.
    pub fn between(
        c1: &CurveOf<T>,
        c2: &CurveOf<T>,
        tol_conf: f64,
        tol: f64,
    ) -> Result<Self, Failure> {
        let mut inter: Self = Self::new();
        inter.perform_between(c1, c2, tol_conf, tol)?;
        Ok(inter)
    }

    /// `IntCurve_IntCurveCurveGen(C1, D1, C2, TolConf, Tol)`.
    /// `GInter::with_first_domain(&a1, &d1, &a2, 1e-9, 1e-9)?`.
    pub fn with_first_domain(
        c1: &CurveOf<T>,
        d1: &Domain,
        c2: &CurveOf<T>,
        tol_conf: f64,
        tol: f64,
    ) -> Result<Self, Failure> {
        let mut inter: Self = Self::new();
        inter.perform_first_domain(c1, d1, c2, tol_conf, tol)?;
        Ok(inter)
    }

    /// `IntCurve_IntCurveCurveGen(C1, C2, D2, TolConf, Tol)`.
    /// `GInter::with_second_domain(&a1, &a2, &d2, 1e-9, 1e-9)?`.
    pub fn with_second_domain(
        c1: &CurveOf<T>,
        c2: &CurveOf<T>,
        d2: &Domain,
        tol_conf: f64,
        tol: f64,
    ) -> Result<Self, Failure> {
        let mut inter: Self = Self::new();
        inter.perform_second_domain(c1, c2, d2, tol_conf, tol)?;
        Ok(inter)
    }

    /// `IntCurve_IntCurveCurveGen(C1, D1, C2, D2, TolConf, Tol)`.
    /// `GInter::with_domains(&a1, &d1, &a2, &d2, 1e-9, 1e-9)?`.
    pub fn with_domains(
        c1: &CurveOf<T>,
        d1: &Domain,
        c2: &CurveOf<T>,
        d2: &Domain,
        tol_conf: f64,
        tol: f64,
    ) -> Result<Self, Failure> {
        let mut inter: Self = Self::new();
        inter.perform(c1, d1, c2, d2, tol_conf, tol)?;
        Ok(inter)
    }

    /// `Perform(C1, C2, TolConf, Tol)`: on the computed domains, with the larger
    /// tolerance at their ends. `inter.perform_between(&a1, &a2, 1e-9, 1e-9)?`.
    pub fn perform_between(
        &mut self,
        c1: &CurveOf<T>,
        c2: &CurveOf<T>,
        tol_conf: f64,
        tol: f64,
    ) -> Result<(), Failure> {
        let tol_domain: f64 = domain_tolerance(tol_conf, tol);
        let d1: Domain = self.compute_domain(c1, tol_domain)?;
        let d2: Domain = self.compute_domain(c2, tol_domain)?;
        self.perform(c1, &d1, c2, &d2, tol_conf, tol)
    }

    /// `Perform(C1, D1, C2, TolConf, Tol)`: the second domain computed.
    /// `inter.perform_first_domain(&a1, &d1, &a2, 1e-9, 1e-9)?`.
    pub fn perform_first_domain(
        &mut self,
        c1: &CurveOf<T>,
        d1: &Domain,
        c2: &CurveOf<T>,
        tol_conf: f64,
        tol: f64,
    ) -> Result<(), Failure> {
        let tol_domain: f64 = domain_tolerance(tol_conf, tol);
        let d2: Domain = self.compute_domain(c2, tol_domain)?;
        self.perform(c1, d1, c2, &d2, tol_conf, tol)
    }

    /// `Perform(C1, C2, D2, TolConf, Tol)`: the first domain computed.
    /// `inter.perform_second_domain(&a1, &a2, &d2, 1e-9, 1e-9)?`.
    pub fn perform_second_domain(
        &mut self,
        c1: &CurveOf<T>,
        c2: &CurveOf<T>,
        d2: &Domain,
        tol_conf: f64,
        tol: f64,
    ) -> Result<(), Failure> {
        let tol_domain: f64 = domain_tolerance(tol_conf, tol);
        let d1: Domain = self.compute_domain(c1, tol_domain)?;
        self.perform(c1, &d1, c2, d2, tol_conf, tol)
    }

    /// `Perform(C, TolConf, Tol)`: the self intersection on the parameter range of
    /// the curve (semi-infinite when one bound is infinite, not done when both
    /// are); empty and done for a conic. `inter.perform_self(&adaptor, 1e-9, 1e-9)?`.
    pub fn perform_self(&mut self, c: &CurveOf<T>, tol_conf: f64, tol: f64) -> Result<(), Failure> {
        let mut d1: Domain = Domain::new();
        let tol_domain: f64 = if tol < tol_conf { tol_conf } else { tol };
        if GpConic::of_curve::<T::Par>(c)?.is_some() {
            self.base.reset_fields();
            self.base.done = true;
            return Ok(());
        }
        let paraminf: f64 = <T::Par as PCurveTool>::first_parameter(c)?;
        let paramsup: f64 = <T::Par as PCurveTool>::last_parameter(c)?;
        if precision::is_infinite(paraminf) && precision::is_infinite(paramsup) {
            self.base.done = false;
            return Ok(());
        }
        set_parameter_domain::<T>(&mut d1, c, (paraminf, paramsup), tol_domain)?;
        self.perform_self_curve(c, &d1, tol_conf, tol)
    }

    /// `Perform(C, D, TolConf, Tol)`: the self intersection on `d`; empty and done
    /// for a conic. `inter.perform_self_on(&adaptor, &domain, 1e-9, 1e-9)?`.
    pub fn perform_self_on(
        &mut self,
        c: &CurveOf<T>,
        d: &Domain,
        tol_conf: f64,
        tol: f64,
    ) -> Result<(), Failure> {
        if GpConic::of_curve::<T::Par>(c)?.is_some() {
            self.base.reset_fields();
            self.base.done = true;
            return Ok(());
        }
        self.perform_self_curve(c, d, tol_conf, tol)
    }

    // The self intersection of a non-conic curve through IntPolyPolyGen.
    fn perform_self_curve(
        &mut self,
        c: &CurveOf<T>,
        d: &Domain,
        tol_conf: f64,
        tol: f64,
    ) -> Result<(), Failure> {
        self.base.reset_fields();
        self.intcurvcurv.set_reversed_parameters(false);
        self.intcurvcurv.perform_self(c, d, tol_conf, tol)?;
        self.base.set_values(&self.intcurvcurv);
        self.base.done = true;
        Ok(())
    }

    /// `ComputeDomain(C1, TolDomain)`: the domain of the curve's parameter range
    /// with `tol_domain` at its ends; a circle or an ellipse has the period 2 Pi, an
    /// unbounded end is left open. `inter.compute_domain(&adaptor, 1e-9)?`.
    pub fn compute_domain(&self, c1: &CurveOf<T>, tol_domain: f64) -> Result<Domain, Failure> {
        let mut d1: Domain = Domain::new();
        let closed: bool = matches!(
            GpConic::of_curve::<T::Par>(c1)?,
            Some(GpConic::Circle(_) | GpConic::Ellipse(_))
        );
        let firstparameter: f64 = <T::Par as PCurveTool>::first_parameter(c1)?;
        let lastparameter: f64 = <T::Par as PCurveTool>::last_parameter(c1)?;
        if !closed {
            set_parameter_domain::<T>(&mut d1, c1, (firstparameter, lastparameter), tol_domain)?;
            return Ok(d1);
        }
        // A trimmed curve gives its first and last parameters.
        let p1 = T::Par::value(c1, firstparameter)?;
        let p2 = T::Par::value(c1, lastparameter)?;
        d1.set_values(
            p1,
            firstparameter,
            tol_domain,
            p2,
            lastparameter,
            tol_domain,
        );
        d1.set_equivalent_parameters(firstparameter, firstparameter + PI + PI)?;
        Ok(d1)
    }

    /// `Perform(C1, D1, C2, D2, TolConf, Tol)`: a composite curve (more than one C1
    /// interval) is intersected per pair of intervals clipped by the domains, the
    /// results merged by the composite Append over the curves' parameter ranges.
    /// `inter.perform(&a1, &d1, &a2, &d2, 1e-9, 1e-9)?`.
    pub fn perform(
        &mut self,
        c1: &CurveOf<T>,
        d1: &Domain,
        c2: &CurveOf<T>,
        d2: &Domain,
        tol_conf: f64,
        tol: f64,
    ) -> Result<(), Failure> {
        self.base.reset_fields();
        let nbi1: i32 = <T::Par as PCurveTool>::nb_intervals(c1)?;
        (self.param1inf, self.param1sup) = composite_bounds::<T>(c1, d1, nbi1)?;
        let nbi2: i32 = <T::Par as PCurveTool>::nb_intervals(c2)?;
        (self.param2inf, self.param2sup) = composite_bounds::<T>(c2, d2, nbi2)?;
        if nbi1 <= 1 && nbi2 <= 1 {
            return self.internal_perform(c1, d1, c2, d2, (tol_conf, tol), false);
        }
        let tab1: Array1<'static, f64> = interval_bounds::<T>(c1, nbi1)?;
        let tab2: Array1<'static, f64> = interval_bounds::<T>(c2, nbi2)?;
        self.internal_composite_perform(
            (c1, d1, nbi1, &tab1),
            (c2, d2, nbi2, &tab2),
            (1, 1),
            (tol_conf, tol),
            true,
        )
    }

    /// `SetMinNbSamples(theMinNbSamples)`: of the curve-curve intersector.
    /// `inter.set_min_nb_samples(40)`.
    pub fn set_min_nb_samples(&mut self, the_min_nb_samples: i32) {
        self.intcurvcurv.set_min_nb_samples(the_min_nb_samples);
    }

    /// `GetMinNbSamples()`. `inter.get_min_nb_samples()`.
    pub fn get_min_nb_samples(&self) -> i32 {
        self.intcurvcurv.get_min_nb_samples()
    }

    // `InternalPerform(C1, D1, C2, D2, TolConf, Tol, Composite)`: the dispatch on
    // the curve types; the result appended (composite) or copied.
    fn internal_perform(
        &mut self,
        c1: &CurveOf<T>,
        d1: &Domain,
        c2: &CurveOf<T>,
        d2: &Domain,
        (tol_conf, tol): (f64, f64),
        composite: bool,
    ) -> Result<(), Failure> {
        let bounds = [
            self.param1inf,
            self.param1sup,
            self.param2inf,
            self.param2sup,
        ];
        match (
            GpConic::of_curve::<T::Par>(c1)?,
            GpConic::of_curve::<T::Par>(c2)?,
        ) {
            (Some(conic1), Some(conic2)) => {
                if conic2.rank() < conic1.rank() {
                    self.intconiconi.set_reversed_parameters(true);
                    self.intconiconi
                        .perform_ordered(&conic2, d2, &conic1, d1, tol_conf, tol)?;
                } else {
                    self.intconiconi.set_reversed_parameters(false);
                    self.intconiconi
                        .perform_ordered(&conic1, d1, &conic2, d2, tol_conf, tol)?;
                }
                merge_result(&mut self.base, &self.intconiconi, bounds, composite)
            }
            (Some(conic1), None) => {
                self.intconicurv.set_reversed_parameters(false);
                self.intconicurv
                    .perform_any_conic(&conic1, d1, c2, d2, tol_conf, tol)?;
                merge_result(&mut self.base, &self.intconicurv, bounds, composite)
            }
            (None, Some(conic2)) => {
                self.intconicurv.set_reversed_parameters(true);
                self.intconicurv
                    .perform_any_conic(&conic2, d2, c1, d1, tol_conf, tol)?;
                merge_result(&mut self.base, &self.intconicurv, bounds, composite)
            }
            (None, None) => {
                self.intcurvcurv.set_reversed_parameters(false);
                self.intcurvcurv.perform(c1, d1, c2, d2, tol_conf, tol)?;
                merge_result(&mut self.base, &self.intcurvcurv, bounds, composite)?;
                self.base.done = true;
                Ok(())
            }
        }
    }

    // `InternalCompositePerform(C1, D1, N1, NB1, Tab1, C2, D2, N2, NB2, Tab2, TolConf,
    // Tol, RecursOnC2)`: every interval of the first curve from `n1` against interval
    // `n2` of the second, then the next interval of the second from the first of the
    // first (the native tail recursion, as a loop).
    fn internal_composite_perform(
        &mut self,
        side1: CompositeCurve<'_, T>,
        side2: CompositeCurve<'_, T>,
        (xxx_num_inter_c1, xxx_num_inter_c2): (i32, i32),
        tols: (f64, f64),
        recurs_on_c2: bool,
    ) -> Result<(), Failure> {
        let mut num_inter_c2: i32 = xxx_num_inter_c2;
        let mut num_inter_c1: i32 = xxx_num_inter_c1;
        loop {
            if num_inter_c2 > side2.2 {
                return Ok(());
            }
            if !recurs_on_c2 {
                return self.internal_composite_perform_no_recurs(
                    (side1, num_inter_c1),
                    (side2, num_inter_c2),
                    tols,
                );
            }
            for i in num_inter_c1..=side1.2 {
                self.internal_composite_perform_no_recurs((side2, num_inter_c2), (side1, i), tols)?;
            }
            if num_inter_c2 >= side2.2 {
                return Ok(());
            }
            num_inter_c2 += 1;
            num_inter_c1 = 1;
        }
    }

    // `InternalCompositePerform_noRecurs(NbInterC1, C1, NumInterC1, Tab1, D1,
    // NbInterC2, C2, NumInterC2, Tab2, D2, TolConf, Tol)`: the domains of both
    // intervals (the whole domain of a curve with one interval), then
    // InternalPerform with the curves swapped back.
    fn internal_composite_perform_no_recurs(
        &mut self,
        ((c1, d1, nb_inter_c1, tab1), num_inter_c1): (CompositeCurve<'_, T>, i32),
        ((c2, d2, nb_inter_c2, tab2), num_inter_c2): (CompositeCurve<'_, T>, i32),
        tols: (f64, f64),
    ) -> Result<(), Failure> {
        if num_inter_c2 > nb_inter_c2 {
            return Ok(());
        }
        let domain_c1 = interval_domain::<T>(c1, d1, (nb_inter_c1, num_inter_c1), tab1)?;
        let domain_c2 = interval_domain::<T>(c2, d2, (nb_inter_c2, num_inter_c2), tab2)?;
        let (Some(domain_c1), Some(domain_c2)) = (domain_c1, domain_c2) else {
            return Ok(());
        };
        self.internal_perform(c2, &domain_c2, c1, &domain_c1, tols, true)
    }
}

// The domain tolerance of the Perform overloads that compute domains: the larger
// of both tolerances (`TolConf` when it is greater).
fn domain_tolerance(tol_conf: f64, tol: f64) -> f64 {
    if tol_conf > tol { tol_conf } else { tol }
}

// The domain of a parameter range: bounded, or semi-infinite from the finite end;
// left as it is when both ends are infinite.
fn set_parameter_domain<T: IntersectorTypes>(
    d1: &mut Domain,
    c: &CurveOf<T>,
    (paraminf, paramsup): (f64, f64),
    tol_domain: f64,
) -> Result<(), Failure> {
    if paraminf > -precision::infinite() {
        if paramsup < precision::infinite() {
            //--         paraminf-----------paramsup
            let (pinf, psup) = (T::Par::value(c, paraminf)?, T::Par::value(c, paramsup)?);
            d1.set_values(pinf, paraminf, tol_domain, psup, paramsup, tol_domain);
        } else {
            //--        paraminf------------...
            d1.set_semi_infinite_values(T::Par::value(c, paraminf)?, paraminf, tol_domain, true);
        }
    } else if paramsup < precision::infinite() {
        //--    ...-----------------paramsup
        d1.set_semi_infinite_values(T::Par::value(c, paramsup)?, paramsup, tol_domain, false);
    }
    Ok(())
}

// The composite bounds of a curve: its parameter range when it has several
// intervals, else the bounds of its domain (infinite where it is open).
fn composite_bounds<T: IntersectorTypes>(
    c: &CurveOf<T>,
    d: &Domain,
    nbi: i32,
) -> Result<(f64, f64), Failure>
where
    T::Par: PCurveTool,
{
    if nbi > 1 {
        return Ok((
            <T::Par as PCurveTool>::first_parameter(c)?,
            <T::Par as PCurveTool>::last_parameter(c)?,
        ));
    }
    let inf: f64 = if d.has_first_point() {
        d.first_parameter()?
    } else {
        -precision::infinite()
    };
    let sup: f64 = if d.has_last_point() {
        d.last_parameter()?
    } else {
        precision::infinite()
    };
    Ok((inf, sup))
}

// `Tab(1, nbi + 1)` filled by `Intervals(C, Tab)`.
fn interval_bounds<T: IntersectorTypes>(
    c: &CurveOf<T>,
    nbi: i32,
) -> Result<Array1<'static, f64>, Failure>
where
    T::Par: PCurveTool,
{
    let mut tab: Array1<'static, f64> = Array1::with_bounds(1, nbi + 1)
        .map_err(|e| e.into_failure("IntCurve_IntCurveCurveGen::Perform"))?;
    <T::Par as PCurveTool>::intervals(c, &mut tab)?;
    Ok(tab)
}

// The domain of interval `num` of a composite curve clipped by `d` (None when the
// clipped interval is at most 1e-10 long), or `d` itself for a single interval.
fn interval_domain<T: IntersectorTypes>(
    c: &CurveOf<T>,
    d: &Domain,
    (nb_inter, num_inter): (i32, i32),
    tab: &Array1<'static, f64>,
) -> Result<Option<Domain>, Failure>
where
    T::Par: PCurveTool,
{
    if nb_inter <= 1 {
        return Ok(Some(*d));
    }
    let (mut param_inf, mut param_sup) = <T::Par as PCurveTool>::get_interval(c, num_inter, tab)?;
    let first: f64 = d.first_parameter()?;
    if param_inf < first {
        param_inf = first;
    }
    let last: f64 = d.last_parameter()?;
    if param_sup > last {
        param_sup = last;
    }
    // `(ParamSup - ParamInf) > 1e-10` natively: a NaN length is not OK either.
    if !matches!(
        (param_sup - param_inf).partial_cmp(&1e-10),
        Some(std::cmp::Ordering::Greater)
    ) {
        return Ok(None);
    }
    Ok(Some(Domain::bounded(
        T::Par::value(c, param_inf)?,
        param_inf,
        d.first_tolerance()?,
        T::Par::value(c, param_sup)?,
        param_sup,
        d.last_tolerance()?,
    )))
}
