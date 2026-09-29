// Copyright (c) 1996-1999 Matra Datavision
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
// Rust translation of ModelingData/TKGeomBase/AppDef/AppDef_Variational.cxx (InitCutting, SplitCurve,
// GettingKnots, NearIndex)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The element knots of the variational curve: the initial cutting at the
//! constraints and the insertion of new knots.
use super::{Variational, construction};
use crate::fem_tool::Curve;
use crate::native_failure::array_failure;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::plib::HermitJacobi;

fn at(array: &Array1<'_, f64>, index: i32) -> Result<f64, Failure> {
    array
        .value(index)
        .copied()
        .map_err(|error| array_failure(error, "AppDef_Variational"))
}

fn put(array: &mut Array1<'_, f64>, index: i32, value: f64) -> Result<(), Failure> {
    array
        .set_value(index, value)
        .map_err(|error| array_failure(error, "AppDef_Variational"))
}

/// `NearIndex(T, TabPar, Eps, Flag)`: the index `i` with `TabPar(i) <= T <
/// TabPar(i + 1)` by bisection (the upper one when `T` is within `eps` of it), and
/// the flag -1 / 1 with the first / last index outside the table.
pub(super) fn near_index(
    t: f64,
    tab_par: &Array1<'_, f64>,
    eps: f64,
) -> Result<(i32, i32), Failure> {
    let (loi, upi) = (tab_par.lower(), tab_par.upper());
    if t < at(tab_par, loi)? {
        return Ok((loi, -1));
    }
    if t > at(tab_par, upi)? {
        return Ok((upi, 1));
    }
    let (mut ibeg, mut ifin) = (loi, upi);
    while ibeg + 1 != ifin {
        let imidl: i32 = (ibeg + ifin) / 2;
        if t >= at(tab_par, ibeg)? && t <= at(tab_par, imidl)? {
            ifin = imidl;
        } else {
            ibeg = imidl;
        }
    }
    if (t - at(tab_par, ifin)?).abs() < eps {
        return Ok((ifin, 0));
    }
    Ok((ibeg, 0))
}

/// `GettingKnots(TabPar, InCurve, Deg, NbElm, NewKnots)`: a new knot inside each
/// element of degree `deg`, from its two knots and the parameters between them,
/// while `NewKnots` has room.
fn getting_knots(
    tab_par: &Array1<'_, f64>,
    in_curve: &Curve,
    deg: i32,
    nb_elm: &mut i32,
    new_knots: &mut Array1<'_, f64>,
) -> Result<(), Failure> {
    let eps: f64 = 1.0e-12;
    let old_knots: Array1<'static, f64> = in_curve.knots().clone();
    let (nb_max_old, nb_max) = (in_curve.nb_elements(), new_knots.upper());
    let (mut el, mut i1) = (0, old_knots.lower());
    let mut i0: i32 = i1 - 1;
    while *nb_elm < nb_max && el < nb_max_old {
        el += 1;
        i0 += 1;
        i1 += 1;
        if in_curve.degree(el)? != deg {
            continue;
        }
        *nb_elm += 1;
        let (k0, k1) = (at(&old_knots, i0)?, at(&old_knots, i1)?);
        put(new_knots, *nb_elm, element_knot(tab_par, (k0, k1), eps)?)?;
    }
    Ok(())
}

// The new knot of the element `[k0, k1]`: a quarter of `k0 + k1` plus twice the
// middle parameter between them (the mean of the two middle ones), or its middle
// without a parameter inside.
fn element_knot(tab_par: &Array1<'_, f64>, (k0, k1): (f64, f64), eps: f64) -> Result<f64, Failure> {
    let (ipt1, flag1) = near_index(k0, tab_par, eps)?;
    let ipt1: i32 = if flag1 != 0 { tab_par.lower() } else { ipt1 };
    let (ipt2, flag2) = near_index(k1, tab_par, eps)?;
    let ipt2: i32 = if flag2 != 0 { tab_par.upper() } else { ipt2 };
    if ipt2 - ipt1 < 1 {
        return Ok((k0 + k1) / 2.);
    }
    let ipt: i32 = (ipt1 + ipt2) / 2;
    let t_par: f64 = if 2 * ipt == ipt1 + ipt2 {
        2. * at(tab_par, ipt)?
    } else {
        at(tab_par, ipt)? + at(tab_par, ipt + 1)?
    };
    Ok((k0 + k1 + t_par) / 4.)
}

// `out_knots` as the knots of `in_curve` followed by the new knots
// `nb_elm_old + 1..=nb_elm` of `new_knots`, sorted.
fn merge_knots(
    out_knots: &mut Array1<'_, f64>,
    in_curve: &Curve,
    new_knots: &Array1<'_, f64>,
    (nb_elm_old, nb_elm): (i32, i32),
) -> Result<(), Failure> {
    let in_knots: Array1<'static, f64> = in_curve.knots().clone();
    let i0: i32 = out_knots.lower();
    for i in in_knots.lower()..=in_knots.upper() {
        put(out_knots, i, at(&in_knots, i)?)?;
    }
    for i in nb_elm_old + 1..=nb_elm {
        put(out_knots, i + i0, at(new_knots, i)?)?;
    }
    out_knots
        .as_mut_slice()
        .sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    Ok(())
}

impl Variational {
    /// `SplitCurve(InCurve, Ti, CurveTol, OutCurve, iscut)`: below MaxSegment
    /// elements, a new knot in each element of WorkDegree, then of WorkDegree - 1;
    /// the cut curve (sorted knots, elements to be recomputed) or None.
    pub(super) fn split_curve(
        &self,
        in_curve: &Curve,
        ti: &Array1<'_, f64>,
        curve_tol: f64,
    ) -> Result<Option<Handle<Curve>>, Failure> {
        let nb_elm_old: i32 = in_curve.nb_elements();
        if nb_elm_old >= self.options.max_segment {
            return Ok(None);
        }
        let (new_knots, nb_elm) = self.split_knots(in_curve, ti)?;
        if nb_elm <= nb_elm_old {
            return Ok(None);
        }
        let out_curve: Curve =
            Curve::new(in_curve.dimension(), nb_elm, in_curve.base(), curve_tol)?;
        merge_knots(
            &mut out_curve.knots(),
            in_curve,
            &new_knots,
            (nb_elm_old, nb_elm),
        )?;
        Ok(Some(Handle::new(out_curve)))
    }

    // The new knots (after the current elements, up to MaxSegment) of the elements
    // of WorkDegree, then of WorkDegree - 1, and the element count they make.
    fn split_knots(
        &self,
        in_curve: &Curve,
        ti: &Array1<'_, f64>,
    ) -> Result<(Array1<'static, f64>, i32), Failure> {
        let mut nb_elm: i32 = in_curve.nb_elements();
        let mut new_knots: Array1<'static, f64> =
            Array1::with_bounds(nb_elm + 1, self.options.max_segment)
                .map_err(|error| array_failure(error, "AppDef_Variational::SplitCurve"))?;
        let work_degree: i32 = in_curve.base().work_degree();
        getting_knots(ti, in_curve, work_degree, &mut nb_elm, &mut new_knots)?;
        getting_knots(ti, in_curve, work_degree - 1, &mut nb_elm, &mut new_knots)?;
        Ok((new_knots, nb_elm))
    }

    // The constraint count `|kind| + 1` of each constraint and the largest one.
    fn constraint_weights(&self) -> Result<(i32, i32), Failure> {
        let (mut orc_mx, mut n_cont) = (-1, 0);
        for i in 1..=self.tables.nb_constr() {
            let kk: i32 = self.tables.typ(2 * i)?.abs() + 1;
            orc_mx = orc_mx.max(kk);
            n_cont += kk;
        }
        Ok((orc_mx, n_cont))
    }

    // Definition of number of elements: NCont constraint counts spread over
    // elements of NLibre free coefficients, NLibre raised until MaxSegment suffices.
    fn element_count(&self) -> Result<(i32, i32), Failure> {
        let (max_degree, niv_cont) = (self.options.max_degree, self.niv_cont);
        let (orc_mx, n_cont) = self.constraint_weights()?;
        if orc_mx > max_degree - niv_cont {
            return Err(construction(&format!(
                "InitCutting: constraint order {orc_mx} above MaxDegree - Continuity {}",
                max_degree - niv_cont
            )));
        }
        let nb_elem: i32 = self.spread_constraints(n_cont);
        if nb_elem > self.options.max_segment {
            return Err(construction(&format!(
                "InitCutting: {nb_elem} elements; expected <= MaxSegment {}",
                self.options.max_segment
            )));
        }
        Ok((nb_elem, n_cont))
    }

    // The element count for `n_cont` constraint counts: NLibre free coefficients
    // per element from about three quarters of the degree on, raised until the
    // elements fit MaxSegment or NLibre reaches MaxDegree - Continuity.
    fn spread_constraints(&self, n_cont: i32) -> i32 {
        let (max_degree, niv_cont) = (self.options.max_degree, self.niv_cont);
        let mut n_libre: i32 = (max_degree - niv_cont - (max_degree + 1) / 4).max(niv_cont + 1);
        let mut nb_elem: i32 = elements_of(n_cont, n_libre);
        while nb_elem > self.options.max_segment && n_libre < max_degree - niv_cont {
            n_libre += 1;
            nb_elem = elements_of(n_cont, n_libre);
        }
        nb_elem
    }

    /// `InitCutting(aBase, CurvTol, aCurve)`: a curve with an element per group of
    /// constraints, its knots placed at (or between) the constrained parameters from
    /// both ends; Standard_ConstructionError when a constraint needs more than
    /// MaxDegree - Continuity or the elements exceed MaxSegment. The native knots
    /// start uninitialized (the port's at 0); the back cutting may read the knot it
    /// is about to set.
    pub(super) fn init_cutting(
        &self,
        base: &HermitJacobi,
        curv_tol: f64,
    ) -> Result<Handle<Curve>, Failure> {
        let (nb_elem, n_cont) = self.element_count()?;
        let curve: Curve = Curve::new(self.dimension, nb_elem, base, curv_tol)?;
        let mut knot: Array1<'static, f64> = curve.knots().clone();
        let n_cnt: i32 = (n_cont - 1) / nb_elem + 1;
        let cutting = Cutting {
            n_cnt,
            n_plus: nb_elem - (n_cnt * nb_elem - n_cont),
            i_deb: 0,
            i_fin: self.tables.nb_constr() + 1,
            n_deb: 0,
            n_fin: 0,
        };
        let (lower, upper) = (knot.lower(), knot.upper());
        put(&mut knot, lower, self.parameter(self.first_point)?)?;
        put(&mut knot, upper, self.parameter(self.last_point)?)?;
        self.cut_knots(cutting, nb_elem, &mut knot)?;
        curve.knots().assign(&knot);
        Ok(Handle::new(curve))
    }

    // The parameter of constraint `i`: myParameters(myTypConstraints(2 i - 1)).
    fn constraint_parameter(&self, i: i32) -> Result<f64, Failure> {
        self.parameter(self.tables.typ(2 * i - 1)?)
    }

    fn cut_knots(
        &self,
        mut c: Cutting,
        nb_elem: i32,
        knot: &mut Array1<'_, f64>,
    ) -> Result<(), Failure> {
        let (mut ind_el, i_upper, mut nb_el) = (knot.lower(), knot.upper(), 0);
        while nb_elem - nb_el > 1 {
            ind_el += 1;
            nb_el += 1;
            if c.n_plus == 0 {
                c.n_cnt -= 1;
            }
            self.front_knot(&mut c, ind_el, knot)?;
            c.n_plus -= 1;
            if c.n_plus == 0 {
                c.n_cnt -= 1;
            }
            if nb_elem - nb_el == 1 {
                break;
            }
            nb_el += 1;
            self.back_knot(&mut c, i_upper + 1 - ind_el, knot)?;
        }
        Ok(())
    }

    // The knot `ind_el` from the front constraints.
    fn front_knot(
        &self,
        c: &mut Cutting,
        ind_el: i32,
        knot: &mut Array1<'_, f64>,
    ) -> Result<(), Failure> {
        while c.n_deb < c.n_cnt && c.i_deb < c.i_fin {
            c.i_deb += 1;
            c.n_deb += self.tables.typ(2 * c.i_deb)?.abs() + 1;
        }
        let at_deb: f64 = self.constraint_parameter(c.i_deb)?;
        let value: f64 = if c.n_deb == c.n_cnt {
            c.n_deb = 0;
            if c.n_plus == 1 && at_deb > at(knot, ind_el - 1)? {
                at_deb
            } else {
                (at_deb + self.parameter(self.tables.typ(2 * c.i_deb + 1)?)?) / 2.
            }
        } else {
            c.n_deb -= c.n_cnt;
            at_deb
        };
        put(knot, ind_el, value)
    }

    // The knot `index` (IUpper + 1 - IndEl) from the back constraints.
    fn back_knot(
        &self,
        c: &mut Cutting,
        index: i32,
        knot: &mut Array1<'_, f64>,
    ) -> Result<(), Failure> {
        while c.n_fin < c.n_cnt && c.i_deb < c.i_fin {
            c.i_fin -= 1;
            c.n_fin += self.tables.typ(2 * c.i_fin)?.abs() + 1;
        }
        let at_fin: f64 = self.constraint_parameter(c.i_fin)?;
        let middle = |this: &Self| -> Result<f64, Failure> {
            Ok((at_fin + this.parameter(this.tables.typ(2 * c.i_fin - 3)?)?) / 2.)
        };
        let value: f64 = if c.n_fin == c.n_cnt {
            c.n_fin = 0;
            middle(self)?
        } else {
            c.n_fin -= c.n_cnt;
            if at_fin < at(knot, index)? {
                at_fin
            } else {
                middle(self)?
            }
        };
        put(knot, index, value)
    }
}

// The elements of `n_libre` coefficients `n_cont` constraint counts need.
fn elements_of(n_cont: i32, n_libre: i32) -> i32 {
    if n_cont % n_libre == 0 {
        n_cont / n_libre
    } else {
        n_cont / n_libre + 1
    }
}

/// The running counters of InitCutting: the constraint counts per element (`NCnt`,
/// `NPlus` elements take one more) and the front and back scans.
struct Cutting {
    n_cnt: i32,
    n_plus: i32,
    i_deb: i32,
    i_fin: i32,
    n_deb: i32,
    n_fin: i32,
}
