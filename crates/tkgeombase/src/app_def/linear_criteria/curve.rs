// Created on: 1998-11-30
// Created by: Igor FEOKTISTOV
// Copyright (c) 1998-1999 Matra Datavision
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
// Rust translation of ModelingData/TKGeomBase/AppDef/AppDef_LinearCriteria.cxx (SetCurve)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AppDef_LinearCriteria::SetCurve`: the element criteria follow the curve's basis.
use super::{CriteriaState, non_null};
use crate::fem_tool::{Curve, ElementaryCriterion, LinearFlexion, LinearJerk, LinearTension};
use crate::native_failure::array_failure;
use rustycad_tkernel::ncollection::HArray2;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::geom_abs::Shape;

/// What the criteria depend on: WorkDegree, Dimension and the basis NivConstr.
#[derive(Clone, Copy, PartialEq)]
struct CurveShape {
    work_degree: i32,
    dimension: i32,
    order: i32,
}

fn shape_of(curve: &Curve) -> CurveShape {
    CurveShape {
        work_degree: curve.base().work_degree(),
        dimension: curve.dimension(),
        order: curve.base().niv_constr(),
    }
}

// The switch on Order; an order above 2 keeps GeomAbs_C0, as natively.
fn constraint_order(order: i32) -> Shape {
    match order {
        1 => Shape::C1,
        2 => Shape::C2,
        _ => Shape::C0,
    }
}

// myCriteria[0..3] = Tension, Flexion, Jerk, each assigned as soon as it is built.
fn build_criteria(state: &mut CriteriaState, shape: CurveShape) -> Result<(), Failure> {
    let order: Shape = constraint_order(shape.order);
    state.criteria[0] = Handle::new(LinearTension::new(shape.work_degree, order)?).cast();
    state.criteria[1] = Handle::new(LinearFlexion::new(shape.work_degree, order)?).cast();
    state.criteria[2] = Handle::new(LinearJerk::new(shape.work_degree, order)?).cast();
    Ok(())
}

// A (0, 0, 1, NbDim) coefficient table given to the three criteria.
fn share_coefficients(state: &CriteriaState, dimension: i32) -> Result<(), Failure> {
    let coeff: HArray2<f64> = HArray2::with_bounds(0, 0, 1, dimension)
        .map_err(|error| array_failure(error, "AppDef_LinearCriteria::SetCurve"))?;
    let coeff: Handle<HArray2<f64>> = Handle::new(coeff);
    for criterion in &state.criteria {
        let criterion: &dyn ElementaryCriterion = non_null(criterion, "an element criterion")?;
        criterion.set_coeff(&coeff);
    }
    Ok(())
}

/// `SetCurve(C)`: the first curve builds the criteria; a different curve rebuilds
/// them when its WorkDegree or NivConstr changed, or only gives them new
/// coefficients when just its dimension changed; the same curve changes nothing.
pub(super) fn set_curve(state: &mut CriteriaState, curve: &Handle<Curve>) -> Result<(), Failure> {
    if state.curve.is_null() {
        state.curve = curve.clone();
        let shape: CurveShape = shape_of(non_null(curve, "the curve")?);
        build_criteria(state, shape)?;
        return share_coefficients(state, shape.dimension);
    }
    if state.curve == *curve {
        return Ok(());
    }
    let old: CurveShape = shape_of(non_null(&state.curve, "the curve")?);
    state.curve = curve.clone();
    let shape: CurveShape = shape_of(non_null(curve, "the curve")?);
    if shape.work_degree != old.work_degree || shape.order != old.order {
        build_criteria(state, shape)?;
        return share_coefficients(state, shape.dimension);
    }
    if shape.dimension != old.dimension {
        return share_coefficients(state, shape.dimension);
    }
    Ok(())
}
