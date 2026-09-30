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
// Rust translation of the solution points of BoundedArc, SolInfo and MinFunction of
// ModelingAlgorithms/TKGeomAlgo/IntStart/IntStart_SearchOnBoundaries.gxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The isolated solutions of an arc: sorted, merged where the function is almost
//! tangent (the "middle" point of close zeros), refined by a Brent minimization of the
//! squared function and processed as points.
use super::bounded_arc::{ArcRoots, BoundedArc};
use super::treat_lc::treat_lc;
use crate::int_start::tools::{SearchOnBoundariesTypes, TheArcFunction};
use crate::int_surf::IntSurfQuadric;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::real_to_int;
use rustycad_tkernel::stl::sort_by;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tkmath::math::{BrentMinimum, Function};

/// `SolInfo`: a solution parameter and its index in the solver result.
#[derive(Clone, Copy, Debug)]
struct SolInfo {
    math_index: i32,
    value: f64,
}

/// `MinFunction`: the square of the arc function, minimized by Brent.
struct MinFunction<'f, F: TheArcFunction> {
    func: &'f mut F,
}

impl<F: TheArcFunction> Function for MinFunction<'_, F> {
    fn value(&mut self, x: f64, result: &mut f64) -> bool {
        if !Function::value(self.func, x, result) {
            return false;
        }
        *result *= *result;
        true
    }
}

/// The parameters of the arc function at a middle point and at the two zeros around it,
/// while the tangent test reads them (`param`, `para`, `parap1`).
struct TangentPair {
    para: f64,
    parap1: f64,
    param: f64,
}

impl<T: SearchOnBoundariesTypes> BoundedArc<'_, T> {
    /// The isolated solutions (`if (Nbp)` of BoundedArc): sorted, the tangent line on a
    /// cylinder handled by TreatLC, else the tangent pairs merged and every solution
    /// processed as a point.
    pub(super) fn solution_points(
        &mut self,
        func: &mut T::Function,
        roots: &ArcRoots,
        quadric: &IntSurfQuadric,
    ) -> Result<(), Failure> {
        let mut a_si: Vec<SolInfo> = sorted_solutions(roots)?;
        // PKV, 21 March 2001: an arc (line) tangent to a cylinder gives several
        // solutions where one is expected; TreatLC finds that one.
        let ip: i32 = treat_lc(self.arc, self.domain, quadric, self.tol_boundary, self.pnt)?;
        if ip == 0 {
            return Ok(());
        }
        // Laurent's usual way: the almost tangent function with almost equal zeros takes
        // the middle point.
        for i in 0..a_si.len().saturating_sub(1) {
            self.merge_tangent_pair(func, roots, &mut a_si, i)?;
        }
        for i in 0..a_si.len() {
            self.solution_point(func, roots, &a_si, i)?;
        }
        Ok(())
    }

    /// The tangent test of the zeros `i` and `i + 1`: when the function stays small and
    /// of one sign between them, the pair becomes one solution at its smallest value.
    fn merge_tangent_pair(
        &self,
        func: &mut T::Function,
        roots: &ArcRoots,
        a_si: &mut [SolInfo],
        i: usize,
    ) -> Result<(), Failure> {
        let mut pair = TangentPair {
            para: a_si[i].value,
            parap1: a_si[i + 1].value,
            param: (a_si[i].value + a_si[i + 1].value) * 0.5,
        };
        let Some(ym) = func.checked_value(pair.param)? else {
            return Ok(());
        };
        // `abs(ym) < maxdist` natively: false on NaN.
        let small: bool = ym.abs() < roots.maxdist;
        if !small {
            return Ok(());
        }
        if !tangent_ends(func, roots, &pair, ym)? {
            return Ok(());
        }
        if self.minimize_on_nodes(func, roots, &mut pair)? {
            a_si[i].value = self.range[0] - 1.;
            a_si[i + 1].value = pair.param;
        }
        Ok(())
    }

    /// skv, OCC569 (31 August 2004): the interval is tangent; `param` becomes the node of
    /// the lowest function value. Returns whether the interval stays tangent.
    fn minimize_on_nodes(
        &self,
        func: &mut T::Function,
        roots: &ArcRoots,
        pair: &mut TangentPair,
    ) -> Result<bool, Failure> {
        let mut a_tol: f64 = self.tol_boundary * 1000.0;
        if a_tol > 0.001 {
            a_tol = 0.001;
        }
        // Fix of floating point exception 569, chl-922-e9.
        pair.parap1 = clamp_1e9(pair.parap1);
        pair.para = clamp_1e9(pair.para);
        let (para, parap1) = (pair.para, pair.parap1);
        let a_nb_nodes: i32 = real_to_int(((parap1 - para) / a_tol).ceil());
        let mut a_val: f64 = f64::MAX;
        let mut a_val_max: f64 = 0.;
        let a_delta: f64 = (parap1 - para) / (f64::from(a_nb_nodes) + 1.);
        for ii in 0..=a_nb_nodes.saturating_add(1) {
            let a_cur_par: f64 = if ii < a_nb_nodes.saturating_add(1) {
                para + f64::from(ii) * a_delta
            } else {
                parap1
            };
            let Some(a_cur_val) = func.checked_value(a_cur_par)? else {
                continue;
            };
            let an_abs_val: f64 = a_cur_val.abs();
            if an_abs_val < a_val {
                a_val = an_abs_val;
                pair.param = a_cur_par;
            }
            if an_abs_val > a_val_max {
                a_val_max = an_abs_val;
            }
        }
        // An interval of the exact intersection is tangent only when the minimum is
        // inside it and the minimum and maximum values are almost the same.
        if roots.is_int_cs_done && a_nb_nodes > 1 {
            return Ok((pair.param - para).abs() > roots.eps_x
                && (parap1 - pair.param).abs() > roots.eps_x
                && 0.01 * a_val_max <= a_val);
        }
        Ok(true)
    }

    /// The isolated solution `i`: refined by Brent when the function is not null there,
    /// then processed as a point of the arc.
    fn solution_point(
        &mut self,
        func: &mut T::Function,
        roots: &ArcRoots,
        a_si: &[SolInfo],
        i: usize,
    ) -> Result<(), Failure> {
        let [pdeb, pfin] = self.range;
        let mut para: f64 = a_si[i].value;
        if (para - pdeb) < roots.eps_x || (pfin - para) < roots.eps_x {
            return Ok(());
        }
        let Some(value) = func.checked_value(para)? else {
            return Ok(());
        };
        let dist: f64 = value.abs();
        let an_indx: i32 = solver_state(roots, a_si[i], dist, self.range)?;
        let mut a_pnt: Point3 = if an_indx < 0 {
            func.last_computed_point()
        } else {
            func.valpoint(an_indx)?
        };
        if dist > 0.1 * precision::confusion() {
            // Precise the point: moves the vertex nearer to the intersection line (#27252)
            // and merges two near vertices. The sorted order is kept by the bounds.
            let a_f_par: f64 = if i == 0 {
                pdeb
            } else {
                (para + a_si[i - 1].value) / 2.0
            };
            let a_l_par: f64 = if i + 1 == a_si.len() {
                pfin
            } else {
                (para + a_si[i + 1].value) / 2.0
            };
            if let Some(location) = brent_location(func, [a_f_par, para, a_l_par])? {
                para = location;
                let a_p2d: Point2 = self.arc.value(para)?;
                a_pnt = func.surface().value(a_p2d.x(), a_p2d.y())?;
            }
        }
        self.point_process(a_pnt, para, self.tol_boundary)?;
        Ok(())
    }
}

/// The solutions of the roots, sorted by parameter with the native `std::sort`.
fn sorted_solutions(roots: &ArcRoots) -> Result<Vec<SolInfo>, Failure> {
    let mut a_si: Vec<SolInfo> = Vec::with_capacity(roots.nbp.max(0) as usize);
    for i in 1..=roots.nbp {
        let value: f64 = match roots.p_sol.as_ref().filter(|_| !roots.is_int_cs_done) {
            Some(p_sol) => p_sol.point(i)?,
            None => roots.int_cs.point(i)?.w(),
        };
        a_si.push(SolInfo {
            math_index: i,
            value,
        });
    }
    sort_by(&mut a_si, |a, b| a.value < b.value);
    Ok(a_si)
}

/// The tangent test at the ends of a pair: the function is small at both ends and, for
/// a line of the exact intersection, of one sign (the line stays on one side).
fn tangent_ends<F: TheArcFunction>(
    func: &mut F,
    roots: &ArcRoots,
    pair: &TangentPair,
    ym: f64,
) -> Result<bool, Failure> {
    let sm: f64 = 1.0_f64.copysign(ym);
    let Some(yf) = func.checked_value(pair.para)? else {
        return Ok(false);
    };
    let Some(yl) = func.checked_value(pair.parap1)? else {
        return Ok(false);
    };
    // A line can be tangent to the surface when all the distances are below maxdist.
    let mut a_tang: bool = yf.abs() < roots.maxdist && yl.abs() < roots.maxdist;
    if a_tang && roots.is_int_cs_done && roots.type_con_s == CurveType::Line {
        let (sf, sl) = (1.0_f64.copysign(yf), 1.0_f64.copysign(yl));
        a_tang = sm == sf && sm == sl;
    }
    Ok(a_tang)
}

/// `(|x| < 1e9) ? x : (x >= 0 ? 1e9 : -1e9)`.
fn clamp_1e9(x: f64) -> f64 {
    if x.abs() < 1.0e9 {
        return x;
    }
    if x >= 0. { 1.0e9 } else { -1.0e9 }
}

/// The solver state of a solution at an end of the range, for the stored point of
/// `math_FunctionAllRoots`; -1 otherwise.
fn solver_state(
    roots: &ArcRoots,
    sol: SolInfo,
    dist: f64,
    [pdeb, pfin]: [f64; 2],
) -> Result<i32, Failure> {
    let Some(p_sol) = roots.p_sol.as_ref().filter(|_| !roots.is_int_cs_done) else {
        return Ok(-1);
    };
    let at_end: bool = (sol.value - pdeb).abs() <= precision::p_confusion()
        || (sol.value - pfin).abs() <= precision::p_confusion();
    if dist < roots.maxdist && at_end {
        return p_sol.point_state(sol.math_index);
    }
    Ok(-1)
}

/// The Brent minimum of the squared function on `[first, last]` from `middle`, when
/// the minimization is done.
fn brent_location<F: TheArcFunction>(
    func: &mut F,
    [first, middle, last]: [f64; 3],
) -> Result<Option<f64>, Failure> {
    let mut a_min = BrentMinimum::new(precision::confusion());
    a_min.perform(&mut MinFunction { func: &mut *func }, first, middle, last);
    func.take_failure()?;
    if !a_min.is_done() {
        return Ok(None);
    }
    Ok(Some(a_min.location()?))
}
