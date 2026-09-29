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
// Rust translation of ModelingData/TKGeomBase/AdvApp2Var/AdvApp2Var_Criterion.hxx/.cxx,
// AdvApp2Var_CriterionType.hxx and AdvApp2Var_CriterionRepartition.hxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AdvApp2Var_Criterion`: an extra condition an approximation must satisfy.
use super::context::AdvApp2VarContext;
use super::patch::AdvApp2VarPatch;
use rustycad_tkernel::standard::Failure;

/// How the criterion drives the cutting (`AdvApp2Var_CriterionType`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum AdvApp2VarCriterionType {
    /// Cut whenever the criterion is not satisfied, without the error bound
    /// (`AdvApp2Var_Absolute`).
    #[default]
    Absolute,
    /// Cut when the error bound fails, or when it holds and the criterion does not
    /// (`AdvApp2Var_Relative`).
    Relative,
}

/// How the cuttings are placed (`AdvApp2Var_CriterionRepartition`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum AdvApp2VarCriterionRepartition {
    /// A new regular grid of N+1 intervals at each step (`AdvApp2Var_Regular`).
    #[default]
    Regular,
    /// One new cutting point at each step (`AdvApp2Var_Incremental`).
    Incremental,
}

/// A criterion evaluated on each approximated patch (`AdvApp2Var_Criterion`).
///
/// The native base class stores the maximum value, the type and the repartition in
/// protected fields read by non-virtual getters; here an implementer returns them.
/// `struct Flat; impl AdvApp2VarCriterion for Flat { ... }`.
pub trait AdvApp2VarCriterion {
    /// Computes the criterion on the patch and stores it with `SetCritValue`
    /// (`Value`); `crit.value(&patch, &context)?`.
    fn value(&self, patch: &AdvApp2VarPatch, context: &AdvApp2VarContext) -> Result<(), Failure>;

    /// Whether the patch satisfies the criterion (`IsSatisfied`); `crit.is_satisfied(&patch)`.
    fn is_satisfied(&self, patch: &AdvApp2VarPatch) -> bool;

    /// The bound of the criterion (`MaxValue`); `crit.max_value()`.
    fn max_value(&self) -> f64;

    /// Absolute or relative (`Type`); `crit.criterion_type()`.
    fn criterion_type(&self) -> AdvApp2VarCriterionType;

    /// Regular or incremental cutting (`Repartition`); `crit.repartition()`.
    fn repartition(&self) -> AdvApp2VarCriterionRepartition;
}
