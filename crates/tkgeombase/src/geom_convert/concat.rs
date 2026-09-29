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
// Rust translation of ModelingData/TKGeomBase/GeomConvert/GeomConvert.cxx
// (ConcatG1, ConcatC1) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomConvert::ConcatG1` and `GeomConvert::ConcatC1`: the concatenation of an
//! array of B-splines into as few B-splines as the G1 junctions allow.
use super::comp_curve::CompCurveToBSplineCurve;
use super::concat_plan::GroupPlan;
use super::concat_support::{cloned_at, need_2deg_repara, need_to_be_treated, pretreatment};
use super::reparametrize::{double_degree_reparametrization, mult_num_and_denom, new_curve_table};
use super::{HArrayOfBSplineCurves, live_bspline};
use crate::hermit;
use crate::native_failure::{array_failure, failure};
use rustycad_tkernel::ncollection::{Array1, ArrayError, HArray1};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dBSplineCurve;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomBoundedCurve};
use rustycad_tkmath::convert::ParameterisationType;

/// `GeomConvert::ConcatG1(ArrayOfCurves, ArrayOfToler, ArrayOfConcatenated,
/// ClosedG1Flag, ClosedTolerance)`: joins each run of G1 junctions of
/// `array_of_curves` (indexed from 0) into one B-spline of `array_of_concatenated`.
///
/// `array_of_toler[i]` is the tolerance of the junction after curve `i`; the G1 test
/// itself uses GeomLProp::Continuity with Precision::Confusion() and
/// Precision::Angular(). When `closed_flag` is set, the array is rotated so that a
/// group spans the closure, and a single group becomes one periodic curve;
/// `closed_flag` is cleared when the closing curve cannot be reparametrized. The input
/// array is rotated and the rational curves' weights are normalized in place.
/// Standard_ConstructionError when two curves do not join, Standard_Failure
/// ("Courbes non jointives") when their ends are apart;
/// `geom_convert::concat_g1(&mut curves, &tolerances, &mut concatenated, &mut closed, 1e-7)?`.
pub fn concat_g1(
    array_of_curves: &mut Array1<'_, Handle<GeomBSplineCurve>>,
    array_of_toler: &Array1<'_, f64>,
    array_of_concatenated: &mut HArrayOfBSplineCurves,
    closed_flag: &mut bool,
    closed_tolerance: f64,
) -> Result<(), Failure> {
    let mut plan: GroupPlan = GroupPlan::build(array_of_curves, array_of_toler, None)?;
    plan.rearrange(array_of_curves, *closed_flag, closed_tolerance)?;
    *array_of_concatenated = new_curve_table(plan.nb_group)?;
    let context = ConcatContext::prepare(array_of_curves, plan.nb_group, closed_flag)?;
    if plan.nb_group == 1 && *closed_flag {
        plan.reorder_at_min_degree(array_of_curves, closed_tolerance)?;
        let closed: Handle<GeomBSplineCurve> = close_group(array_of_curves, &plan, context, false)?;
        return set_curve(array_of_concatenated, 0, closed);
    }
    // The open groups append with the input tolerances, before the curve.
    let joins = GroupJoins {
        toler: array_of_toler,
        after: false,
    };
    for_each_group(&plan, |group, j| {
        let curve: Handle<GeomBSplineCurve> = cloned_at(array_of_curves, j)?;
        group.append(array_of_concatenated, curve, &joins)
    })
}

/// `GeomConvert::ConcatC1(ArrayOfCurves, ArrayOfToler, ArrayOfIndices,
/// ArrayOfConcatenated, ClosedG1Flag, ClosedTolerance)`: [`concat_c1_with_angular`]
/// with Precision::Angular();
/// `geom_convert::concat_c1(&mut curves, &tol, &mut indices, &mut result, &mut closed, 1e-7)?`.
pub fn concat_c1(
    array_of_curves: &mut Array1<'_, Handle<GeomBSplineCurve>>,
    array_of_toler: &Array1<'_, f64>,
    array_of_indices: &mut Handle<HArray1<i32>>,
    array_of_concatenated: &mut HArrayOfBSplineCurves,
    closed_flag: &mut bool,
    closed_tolerance: f64,
) -> Result<(), Failure> {
    let outputs = ConcatOutputs {
        indices: array_of_indices,
        concatenated: array_of_concatenated,
    };
    concat_c1_with_angular(
        array_of_curves,
        array_of_toler,
        outputs,
        closed_flag,
        closed_tolerance,
        precision::angular(),
    )
}

/// The two output arrays of ConcatC1: the index in the (rotated) input array of the
/// first curve of each group, then a closing index, and the concatenated curves.
pub struct ConcatOutputs<'o> {
    /// `ArrayOfIndices`, filled at `0..NbGroup`.
    pub indices: &'o mut Handle<HArray1<i32>>,
    /// `ArrayOfConcatenated`, filled at `0..NbGroup - 1`.
    pub concatenated: &'o mut HArrayOfBSplineCurves,
}

/// `GeomConvert::ConcatC1(..., ClosedTolerance, AngularTolerance)`: [`concat_g1`]
/// where the G1 test uses `array_of_toler` and `angular_tolerance`, each rational
/// curve whose end weights are not 1 is first multiplied by its Hermit law so that
/// the junctions become C1, and the open groups append each curve after the previous
/// one with the rotated tolerances (ConcatG1 appends before, with the input ones);
/// `geom_convert::concat_c1_with_angular(&mut curves, &tol, outputs, &mut closed, 1e-7, 1e-9)?`.
pub fn concat_c1_with_angular(
    array_of_curves: &mut Array1<'_, Handle<GeomBSplineCurve>>,
    array_of_toler: &Array1<'_, f64>,
    outputs: ConcatOutputs<'_>,
    closed_flag: &mut bool,
    closed_tolerance: f64,
    angular_tolerance: f64,
) -> Result<(), Failure> {
    let mut plan: GroupPlan =
        GroupPlan::build(array_of_curves, array_of_toler, Some(angular_tolerance))?;
    plan.rearrange(array_of_curves, *closed_flag, closed_tolerance)?;
    *outputs.indices = Handle::new(HArray1::with_bounds(0, plan.nb_group).map_err(table_error)?);
    *outputs.concatenated = new_curve_table(plan.nb_group)?;
    let context = ConcatContext::prepare(array_of_curves, plan.nb_group, closed_flag)?;
    if plan.nb_group == 1 && *closed_flag {
        set_index(outputs.indices, 0, 0)?;
        set_index(outputs.indices, 1, 0)?;
        plan.reorder_at_min_degree(array_of_curves, closed_tolerance)?;
        let closed: Handle<GeomBSplineCurve> = close_group(array_of_curves, &plan, context, true)?;
        return set_curve(outputs.concatenated, 0, closed);
    }
    let local_tolerance: Array1<'static, f64> = plan.local_tolerance.clone();
    let joins = GroupJoins {
        toler: &local_tolerance,
        after: true,
    };
    let (concatenated, indices) = (outputs.concatenated, outputs.indices);
    for_each_group(&plan, |group, j| {
        if j == group.index {
            record_group_indices(indices, &plan, group, *closed_flag)?;
        }
        let curve: Handle<GeomBSplineCurve> = hermit_treated(&cloned_at(array_of_curves, j)?)?;
        group.append(concatenated, curve, &joins)
    })
}

// What both members compute after the rearrangement: the weight normalization and
// the degree-doubling test of the closing curve (which clears `closed_flag` when
// the doubled degree would exceed Geom2d_BSplineCurve::MaxDegree(), the 2D bound the
// native 3D code also reads).
#[derive(Clone, Copy)]
struct ConcatContext {
    need_double_deg_repara: bool,
}

impl ConcatContext {
    fn prepare(
        curves: &Array1<'_, Handle<GeomBSplineCurve>>,
        nb_group: i32,
        closed_flag: &mut bool,
    ) -> Result<Self, Failure> {
        pretreatment(curves)?;
        let need_double_deg_repara: bool = need_2deg_repara(curves)?;
        if nb_group == 1 && *closed_flag && need_double_deg_repara {
            let closing: Handle<GeomBSplineCurve> = cloned_at(curves, curves.length() - 1)?;
            if live_bspline(&closing)?.degree() > Geom2dBSplineCurve::max_degree() / 2 {
                *closed_flag = false;
            }
        }
        Ok(Self {
            need_double_deg_repara,
        })
    }
}

// The single closed group: every curve (Hermit-treated for ConcatC1) appended to the
// previous ones with the rotated tolerances, the closing curve reparametrized first
// when needed, then made periodic.
fn close_group(
    curves: &Array1<'_, Handle<GeomBSplineCurve>>,
    plan: &GroupPlan,
    context: ConcatContext,
    treated: bool,
) -> Result<Handle<GeomBSplineCurve>, Failure> {
    let pick = |j: i32| -> Result<Handle<GeomBSplineCurve>, Failure> {
        let curve: Handle<GeomBSplineCurve> = cloned_at(curves, j)?;
        if treated {
            hermit_treated(&curve)
        } else {
            Ok(curve)
        }
    };
    let mut curve2: Handle<GeomBSplineCurve> = pick(0)?;
    for j in 1..=(plan.nb_curve - 1) {
        let mut curve1: Handle<GeomBSplineCurve> = pick(j)?;
        if j == plan.nb_curve - 1 && context.need_double_deg_repara {
            curve1 = double_degree_reparametrization(&curve1, &curve2, &cloned_at(curves, 0)?)?;
        }
        curve2 = merged(
            &curve2,
            &curve1,
            cloned_at(&plan.local_tolerance, j - 1)?,
            false,
        )?;
    }
    make_periodic(&curve2)?;
    Ok(curve2)
}

// A single C1 curve: periodic, the closure knot reduced to multiplicity 1 as far as
// Precision::Confusion() allows.
fn make_periodic(curve: &Handle<GeomBSplineCurve>) -> Result<(), Failure> {
    let curve: &GeomBSplineCurve = live_bspline(curve)?;
    curve.set_periodic()?;
    let last: i32 = curve.last_u_knot_index()?;
    curve.remove_knot(last, curve.multiplicity(last)? - 1, precision::confusion())?;
    Ok(())
}

// CompCurveToBSplineCurve(first).Add(second, tolerance, after): Standard_ConstructionError
// ("GeomConvert Concatenation Error") when the curves do not join.
fn merged(
    first: &Handle<GeomBSplineCurve>,
    second: &Handle<GeomBSplineCurve>,
    tolerance: f64,
    after: bool,
) -> Result<Handle<GeomBSplineCurve>, Failure> {
    let mut comp = CompCurveToBSplineCurve::with_curve(
        &first.cast::<dyn GeomBoundedCurve>(),
        ParameterisationType::TgtThetaOver2,
    )?;
    if !comp.add(
        &second.cast::<dyn GeomBoundedCurve>(),
        tolerance,
        after,
        true,
        0,
    )? {
        return Err(failure(
            "Standard_ConstructionError",
            "GeomConvert Concatenation Error",
        ));
    }
    Ok(comp.bspline_curve())
}

// A rational curve whose end weights are not 1 is multiplied by its Hermit law
// (Hermit::Solution with its default tolerances 1e-6).
fn hermit_treated(curve: &Handle<GeomBSplineCurve>) -> Result<Handle<GeomBSplineCurve>, Failure> {
    if need_to_be_treated(live_bspline(curve)?)? {
        return mult_num_and_denom(&hermit::solution_3d(curve, 1e-6, 1e-6)?, curve);
    }
    Ok(curve.clone())
}

// How the open groups append: the junction tolerances and the Add side.
struct GroupJoins<'t> {
    toler: &'t Array1<'t, f64>,
    after: bool,
}

// One step of the open-array loop: curve `current` of group `number`, which starts
// at curve `index`.
struct GroupStep {
    number: i32,
    index: i32,
    current: i32,
}

impl GroupStep {
    // The first curve of a group fills its slot; the next ones are appended.
    fn append(
        &self,
        table: &HArrayOfBSplineCurves,
        curve: Handle<GeomBSplineCurve>,
        joins: &GroupJoins<'_>,
    ) -> Result<(), Failure> {
        if self.current == self.index {
            return set_curve(table, self.number, curve);
        }
        let first: Handle<GeomBSplineCurve> = table_of(table)?
            .array1()
            .value(self.number)
            .map_err(table_error)?
            .clone();
        let joined = merged(
            &first,
            &curve,
            cloned_at(joins.toler, self.current - 1)?,
            joins.after,
        )?;
        set_curve(table, self.number, joined)
    }
}

// The main loop on each group and the secondary loop inside it.
fn for_each_group(
    plan: &GroupPlan,
    mut visit: impl FnMut(&GroupStep, i32) -> Result<(), Failure>,
) -> Result<(), Failure> {
    let mut index: i32 = 0;
    for number in 0..=(plan.nb_group - 1) {
        let nb_vertex_g1: i32 = plan.vertices_g1(index)?;
        for current in index..=(index + nb_vertex_g1) {
            visit(
                &GroupStep {
                    number,
                    index,
                    current,
                },
                current,
            )?;
        }
        index = index + 1 + nb_vertex_g1;
    }
    Ok(())
}

// Filling the array of preserved indices: the first curve of each group, then the
// closing index, shifted by the rotation of a closed array.
fn record_group_indices(
    indices: &Handle<HArray1<i32>>,
    plan: &GroupPlan,
    step: &GroupStep,
    closed_flag: bool,
) -> Result<(), Failure> {
    let k: i32 = step.number + 1;
    let (first, closing): (i32, i32) = if !closed_flag || plan.nb_group == 1 {
        (step.index, 0)
    } else {
        (
            step.index + plan.nb_vertex_group0 + 1,
            plan.nb_vertex_group0 + 1,
        )
    };
    set_index(indices, k - 1, first)?;
    if k == plan.nb_group {
        set_index(indices, k, closing)?;
    }
    Ok(())
}

fn set_index(indices: &Handle<HArray1<i32>>, index: i32, value: i32) -> Result<(), Failure> {
    table_of(indices)?
        .change_array1()
        .set_value(index, value)
        .map_err(table_error)
}

fn set_curve(
    table: &HArrayOfBSplineCurves,
    index: i32,
    curve: Handle<GeomBSplineCurve>,
) -> Result<(), Failure> {
    table_of(table)?
        .change_array1()
        .set_value(index, curve)
        .map_err(table_error)
}

// The object of an output table handle (always allocated before it is written).
fn table_of<T: Send + Sync + 'static>(table: &Handle<HArray1<T>>) -> Result<&HArray1<T>, Failure> {
    table.get().ok_or_else(|| {
        failure(
            "Standard_NullObject",
            "GeomConvert: null output table; expected an array",
        )
    })
}

fn table_error(error: ArrayError) -> Failure {
    array_failure(error, "GeomConvert")
}
