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
// Rust translation of IntImpParGen_Intersector.gxx (the null intervals and
// Calcule_Toutes_Transitions) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The segments of `IntImpParGen_Intersector`: the null intervals of the signed
//! distance, mapped onto the implicit curve, clipped by its domain and classified.
use super::intersector::{Intersector, ParCurveOf, PerformInput, PerformState};
use super::package::{determine_position, normalize_on_domain};
use super::traits::{ImpTool, IntersectorTypes, ParTool};
use crate::int_res2d::{IntersectionPoint, IntersectionSegment, Position};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{self, Point2, Vector2};
use rustycad_tkmath::math::FunctionAllRoots;

// A fixed-size native array of 1-based bounds `[1, size]`.
pub(super) fn real_array(size: i32) -> Result<Array1<'static, f64>, Failure> {
    Array1::with_bounds(1, size).map_err(|e| e.into_failure("NCollection_Array1<double>"))
}

pub(super) fn set(array: &mut Array1<'_, f64>, index: i32, value: f64) -> Result<(), Failure> {
    array
        .set_value(index, value)
        .map_err(|e| e.into_failure("NCollection_Array1<double>::SetValue"))
}

pub(super) fn get(array: &Array1<'_, f64>, index: i32) -> Result<f64, Failure> {
    array
        .value(index)
        .copied()
        .map_err(|e| e.into_failure("NCollection_Array1<double>::Value"))
}

/// The interval tables of Perform: the bounds of each null interval as parameters
/// of the parametric curve (`Inter2_and_Domaine2`) and of the implicit one
/// (`Inter1`), 1-based.
pub(super) struct IntervalTables {
    pub(super) inter2: Array1<'static, f64>,
    pub(super) inter1: Array1<'static, f64>,
}

impl<T: IntersectorTypes> Intersector<T> {
    // The segments of Perform: the null intervals (shifted copies on a closed
    // implicit curve), clipped to the implicit domain, then classified.
    pub(super) fn add_null_intervals(
        &mut self,
        input: &PerformInput<'_, T>,
        sol: &FunctionAllRoots,
        state: &mut PerformState,
    ) -> Result<(), Failure> {
        let nb_segments_solution: i32 = sol.number_of_intervals()?;
        let mut tables = IntervalTables {
            inter2: real_array(2 + 8 * nb_segments_solution)?,
            inter1: real_array(2 + 8 * nb_segments_solution)?,
        };
        let mut nb_segments_crees: i32 = 0;
        let mut j2: i32 = 1;
        for j in 1..=nb_segments_solution {
            let [param2_inf, param2_sup] = sol.interval(j)?;
            let mut ptemp: Point2 = Point2::default();
            let mut param1_inf: f64 =
                self.find_u(param2_inf, &mut ptemp, input.par_curve, input.imp_tool)?;
            let mut param1_sup: f64 =
                self.find_u(param2_sup, &mut ptemp, input.par_curve, input.imp_tool)?;
            if input.imp_domain.is_closed() {
                (param1_inf, param1_sup) =
                    orient_on_period(input, (param2_inf, param1_inf, param1_sup))?;
                nb_segments_crees += add_shifted_copies(
                    input,
                    &mut tables,
                    &mut j2,
                    [param2_inf, param2_sup, param1_inf, param1_sup],
                )?;
            }
            store_bounds(
                &mut tables,
                j2,
                [param2_inf, param2_sup, param1_inf, param1_sup],
            )?;
            j2 += 2;
        }
        let mut resultat1: Array1<'static, f64> = real_array(2 + (1 + nb_segments_solution) * 2)?;
        let mut resultat2: Array1<'static, f64> = real_array(2 + (1 + nb_segments_solution) * 2)?;
        let mut nb_resultats: i32 = (nb_segments_solution + nb_segments_crees) * 2;
        self.and_domaine_objet1_intersections(
            input.imp_tool,
            input.par_curve,
            input.imp_domain,
            input.par_domain,
            &mut nb_resultats,
            (&mut tables.inter2, &mut tables.inter1),
            (&mut resultat1, &mut resultat2),
            input.eps_nul,
        )?;
        for k in (1..=nb_resultats).step_by(2) {
            self.add_result_pair(input, (&resultat1, &resultat2), k, state)?;
        }
        Ok(())
    }
}

// Writes one interval at `j2` and `j2 + 1` of the tables.
fn store_bounds(
    tables: &mut IntervalTables,
    j2: i32,
    [p2inf, p2sup, p1inf, p1sup]: [f64; 4],
) -> Result<(), Failure> {
    set(&mut tables.inter2, j2, p2inf)?;
    set(&mut tables.inter1, j2, p1inf)?;
    set(&mut tables.inter2, j2 + 1, p2sup)?;
    set(&mut tables.inter1, j2 + 1, p1sup)
}

// On a closed implicit curve: both parameters are moved above the period origin,
// then the interval is oriented by the tangents at its first bound (T1 towards the
// matter: an entering point).
fn orient_on_period<T: IntersectorTypes>(
    input: &PerformInput<'_, T>,
    (param2_inf, mut param1_inf, mut param1_sup): (f64, f64, f64),
) -> Result<(f64, f64), Failure> {
    let (param1_origine, param1_fin) = input.imp_domain.equivalent_parameters();
    let periode: f64 = param1_fin - param1_origine;
    while param1_inf < param1_origine {
        param1_inf += periode;
    }
    while param1_sup < param1_origine {
        param1_sup += periode;
    }
    let (mut ptemp, mut t1, mut t2, mut n1, mut n2) = (
        Point2::default(),
        Vector2::default(),
        Vector2::default(),
        Vector2::default(),
        Vector2::default(),
    );
    T::Par::d2(input.par_curve, param2_inf, &mut ptemp, &mut t2, &mut n2)?;
    input.imp_tool.d2(param1_inf, &mut ptemp, &mut t1, &mut n1);
    if t1.magnitude() <= gp::resolution() {
        t1 = n1;
    }
    if t2.magnitude() <= gp::resolution() {
        t2 = n2;
    }
    if t1.dot(t2) >= 0.0 {
        if param1_inf >= param1_sup {
            param1_sup += periode;
        }
    } else if param1_inf <= param1_sup {
        param1_inf += periode;
    }
    Ok((param1_inf, param1_sup))
}

// The copies of an interval shifted by one period up and down when they still
// reach into the implicit domain (e.g. domain PI/4..PI/2 or 1.5PI..2.5PI against
// the interval 0..PI); returns the number of copies written.
fn add_shifted_copies<T: IntersectorTypes>(
    input: &PerformInput<'_, T>,
    tables: &mut IntervalTables,
    j2: &mut i32,
    [p2inf, p2sup, p1inf, p1sup]: [f64; 4],
) -> Result<i32, Failure> {
    let (origine, fin) = input.imp_domain.equivalent_parameters();
    let periode: f64 = fin - origine;
    let mut created: i32 = 0;
    let upper: f64 = if p1inf > p1sup {
        p1sup + periode
    } else {
        p1inf + periode
    };
    if input.imp_domain.last_parameter()? > upper {
        store_bounds(
            tables,
            *j2,
            [p2inf, p2sup, p1inf + periode, p1sup + periode],
        )?;
        *j2 += 2;
        created += 1;
    }
    let lower: f64 = if p1inf < p1sup {
        p1sup - periode
    } else {
        p1inf - periode
    };
    if input.imp_domain.first_parameter()? < lower {
        store_bounds(
            tables,
            *j2,
            [p2inf, p2sup, p1inf - periode, p1sup - periode],
        )?;
        *j2 += 2;
        created += 1;
    }
    Ok(created)
}

impl<T: IntersectorTypes> Intersector<T> {
    // One pair of results (`Calcule_Toutes_Transitions`): a point when both ends
    // coincide on an open implicit curve, a segment otherwise.
    fn add_result_pair(
        &mut self,
        input: &PerformInput<'_, T>,
        (resultat1, resultat2): (&Array1<'_, f64>, &Array1<'_, f64>),
        k: i32,
        state: &mut PerformState,
    ) -> Result<(), Failure> {
        let dist_mini_imp_curve: f64 = input.eps_nul;
        let (p1_on1, p1_on2) = (get(resultat1, k)?, get(resultat2, k)?);
        let (p2_on1, p2_on2) = (get(resultat1, k + 1)?, get(resultat2, k + 1)?);
        let pt1_on1: Point2 = input.imp_tool.value(p1_on1);
        let pt2_on1: Point2 = input.imp_tool.value(p2_on1);
        let pt1_on2: Point2 = T::Par::value(input.par_curve, p1_on2)?;
        let pt2_on2: Point2 = T::Par::value(input.par_curve, p2_on2)?;
        let only_one_point: bool = !input.imp_domain.is_closed()
            && pt1_on1.distance(pt2_on1) <= dist_mini_imp_curve
            && pt1_on2.distance(pt2_on2) <= dist_mini_imp_curve;
        let new_p1: IntersectionPoint = self.bound_point(input, (p1_on1, p1_on2), state)?;
        if only_one_point {
            return self.base.insert(new_p1);
        }
        let new_p2: IntersectionPoint = self.bound_point(input, (p2_on1, p2_on2), state)?;
        let segopposite: bool = state.tan1.dot(state.tan2) < 0.0;
        let reversed: bool = self.base.reversed_parameters();
        self.base.append_segment(IntersectionSegment::bounded(
            new_p1,
            new_p2,
            segopposite,
            reversed,
        ));
        Ok(())
    }

    // One bound of a result pair: normalized parameters, positions, transitions,
    // and the mid point when the bound is at an end of both domains.
    fn bound_point(
        &self,
        input: &PerformInput<'_, T>,
        (param_on1, param_on2): (f64, f64),
        state: &mut PerformState,
    ) -> Result<IntersectionPoint, Failure> {
        let param_on1: f64 = normalize_on_domain(param_on1, input.imp_domain)?;
        let param_on2: f64 = normalize_on_domain(param_on2, input.par_domain)?;
        let mut pt_on1: Point2 = Point2::default();
        let mut pt_on2: Point2 = Point2::default();
        input
            .imp_tool
            .d2(param_on1, &mut pt_on1, &mut state.tan1, &mut state.norm1);
        T::Par::d2(
            input.par_curve,
            param_on2,
            &mut pt_on2,
            &mut state.tan2,
            &mut state.norm2,
        )?;
        let pos1: Position = determine_position(input.imp_domain, pt_on1, param_on1)?;
        let pos2: Position = determine_position(input.par_domain, pt_on2, param_on2)?;
        state.record(pos1, pos2);
        state.transition(pos1, pos2, input.eps_nul);
        if pos1 != Position::Middle && pos2 != Position::Middle {
            pt_on1.set_x(0.5 * (pt_on1.x() + pt_on2.x()));
            pt_on1.set_y(0.5 * (pt_on1.y() + pt_on2.y()));
        }
        let reversed: bool = self.base.reversed_parameters();
        Ok(IntersectionPoint::new(
            pt_on1,
            param_on1,
            param_on2,
            state.trans1,
            state.trans2,
            reversed,
        ))
    }

    /// `FindV(parameter, point, ImpTool, ParCurve, ParDomain, V0, V1, Tolerance)`:
    /// the point of the implicit curve at `parameter` (written to `point`) and its
    /// parameter on the parametric curve, normalized on a closed domain, clamped to
    /// `[min(V0, V1), max(V0, V1)]` otherwise;
    /// `inter.find_v(0.5, &mut point, &itool, &pconic, &d2, 0.0, 1.0, 1e-9)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn find_v(
        &self,
        parameter: f64,
        point: &mut Point2,
        imp_tool: &T::Imp,
        par_curve: &ParCurveOf<T>,
        par_domain: &crate::int_res2d::Domain,
        v0: f64,
        v1: f64,
        tolerance: f64,
    ) -> Result<f64, Failure> {
        use super::traits::ProjectOnPCurveTool;
        *point = imp_tool.value(parameter);
        if par_domain.is_closed() {
            let v: f64 = T::Project::find_parameter(par_curve, *point, tolerance)?;
            return normalize_on_domain(v, par_domain);
        }
        let (vv0, vv1) = if v1 < v0 { (v1, v0) } else { (v0, v1) };
        let x: f64 = T::Project::find_parameter_in(par_curve, *point, vv0, vv1, tolerance)?;
        if x > vv1 {
            return Ok(vv1);
        }
        if x < vv0 {
            return Ok(vv0);
        }
        Ok(x)
    }
}
