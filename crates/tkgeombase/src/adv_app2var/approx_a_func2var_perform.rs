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

//! `AdvApp2Var_ApproxAFunc2Var::Perform`, `ComputePatches` and
//! `ComputeConstraints`: the loops that approximate the isos and the patches and
//! cut the domain until every piece is approximated.
use super::approx_a_func2var::AdvApp2VarApproxAFunc2Var;
use super::criterion::{
    AdvApp2VarCriterion, AdvApp2VarCriterionRepartition, AdvApp2VarCriterionType,
};
use super::evaluator::AdvApp2VarEvaluatorFunc2Var;
use super::node::AdvApp2VarNode;
use super::patch::construction;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adv_approx::AdvApproxCutting;
use rustycad_tkmath::geom_abs::IsoType;

/// The cuttings a patch may ask for and where they would be
/// (`NumDec`, `Umore`/`Udec`, `Vmore`/`Vdec`).
struct AllowedCuts {
    num_dec: i32,
    u_cut: f64,
    v_cut: f64,
}

/// The `NumDec` of `ComputePatches`: which cuttings stay within the patch budget.
/// `lone` is the value when only U or only V may be cut in a grid with room for a
/// full row and column: `[3, 4]` without a criterion, `[1, 2]` with one.
fn num_dec(nb: [i32; 2], max_patches: i32, more: [bool; 2], lone: [i32; 2]) -> i32 {
    let ([nb_u, nb_v], [u_more, v_more]) = (nb, more);
    let nb_patch: i32 = nb_u * nb_v;
    let (fits_row, fits_col) = (
        nb_patch + nb_v <= max_patches,
        nb_patch + nb_u <= max_patches,
    );
    let mut num_dec: i32 = 0;
    if fits_row && !fits_col && u_more {
        num_dec = 1;
    }
    if !fits_row && fits_col && v_more {
        num_dec = 2;
    }
    if fits_row && fits_col {
        if u_more {
            num_dec = 3;
        }
        if nb_v > nb_u && v_more {
            num_dec = 4;
        }
    }
    if (nb_u + 1) * (nb_v + 1) <= max_patches {
        num_dec = match (u_more, v_more) {
            (false, false) => 0,
            (true, false) => lone[0],
            (false, true) => lone[1],
            (true, true) => 5,
        };
    }
    num_dec
}

fn approximation_error() -> Failure {
    construction("AdvApp2Var_ApproxAFunc2Var : Surface Approximation Error")
}

impl AdvApp2VarApproxAFunc2Var {
    /// Approximates every patch, then computes the 3D errors, and the criterion
    /// error when there is a criterion (`Perform`, both overloads).
    pub(super) fn perform(
        &mut self,
        u_choice: &dyn AdvApproxCutting,
        v_choice: &dyn AdvApproxCutting,
        func: &dyn AdvApp2VarEvaluatorFunc2Var,
        crit: Option<&dyn AdvApp2VarCriterion>,
    ) -> Result<(), Failure> {
        match crit {
            None => self.compute_patches(u_choice, v_choice, func)?,
            Some(crit) => self.compute_patches_with(u_choice, v_choice, func, crit)?,
        }
        self.has_result = true;
        self.done = true;
        self.compute_3d_errors()?;
        if crit.is_some() {
            self.compute_crit_error();
        }
        Ok(())
    }

    /// Discretises patch `first` after completing the constraints; a failed
    /// discretisation is a ConstructionError.
    fn discretise_patch(
        &mut self,
        first: i32,
        u_choice: &dyn AdvApproxCutting,
        v_choice: &dyn AdvApproxCutting,
        func: &dyn AdvApp2VarEvaluatorFunc2Var,
        crit: Option<&dyn AdvApp2VarCriterion>,
    ) -> Result<(), Failure> {
        self.compute_constraints(u_choice, v_choice, func, crit)?;
        let patch = self.result.change_patch(first)?;
        patch.discretise(&self.conditions, &self.constraints, func)?;
        if !patch.is_discretised() {
            return Err(construction(
                "AdvApp2Var_ApproxAFunc2Var : Surface Discretisation Error",
            ));
        }
        Ok(())
    }

    /// The cuttings allowed on patch `first` and their parameters.
    fn allowed_cuts(
        &self,
        first: i32,
        u_choice: &dyn AdvApproxCutting,
        v_choice: &dyn AdvApproxCutting,
        lone: [i32; 2],
    ) -> Result<AllowedCuts, Failure> {
        let nb: [i32; 2] = [self.result.nb_patch_in_u(), self.result.nb_patch_in_v()];
        let patch = self.result.change_patch(first)?;
        let (u_more, u_cut) = u_choice.value(patch.u0(), patch.u1());
        let (v_more, v_cut) = v_choice.value(patch.v0(), patch.v1());
        Ok(AllowedCuts {
            num_dec: num_dec(nb, self.max_patches, [u_more, v_more], lone),
            u_cut,
            v_cut,
        })
    }

    /// Applies a cutting decision to patch `first`: 0 keeps its result (an error
    /// when there is none), 1 cuts in U, 2 in V, 3 in both.
    fn apply_cut(&mut self, decision: i32, first: i32, cuts: &AllowedCuts) -> Result<(), Failure> {
        match decision {
            0 => {
                let patch = self.result.change_patch(first)?;
                if !patch.has_result() {
                    return Err(approximation_error());
                }
                patch.overwrite_approx();
            }
            1 => self.cut_in_u(cuts.u_cut)?,
            2 => self.cut_in_v(cuts.v_cut)?,
            3 => {
                self.cut_in_u(cuts.u_cut)?;
                self.cut_in_v(cuts.v_cut)?;
            }
            _ => return Err(approximation_error()),
        }
        Ok(())
    }

    fn cut_in_u(&mut self, value: f64) -> Result<(), Failure> {
        self.result.update_in_u(value)?;
        self.constraints.update_in_u(value)
    }

    fn cut_in_v(&mut self, value: f64) -> Result<(), Failure> {
        self.result.update_in_v(value)?;
        self.constraints.update_in_v(value)
    }

    /// Approximates the patches, cutting where the approximation asks for it
    /// (`ComputePatches` without criterion).
    fn compute_patches(
        &mut self,
        u_choice: &dyn AdvApproxCutting,
        v_choice: &dyn AdvApproxCutting,
        func: &dyn AdvApp2VarEvaluatorFunc2Var,
    ) -> Result<(), Failure> {
        while let Some(first) = self.result.first_not_approx() {
            self.discretise_patch(first, u_choice, v_choice, func, None)?;
            let cuts: AllowedCuts = self.allowed_cuts(first, u_choice, v_choice, [3, 4])?;
            let patch = self.result.change_patch(first)?;
            patch.make_approx(&self.conditions, &self.constraints, cuts.num_dec)?;
            if !patch.is_approximated() {
                let decision: i32 = patch.cut_sense();
                self.apply_cut(decision, first, &cuts)?;
            }
        }
        Ok(())
    }

    /// Approximates the patches, cutting where the approximation or the criterion
    /// asks for it, with a new regular grid for a regular criterion
    /// (`ComputePatches` with criterion). The native running maximum `m1` of the
    /// criterion values is never read and is left out.
    fn compute_patches_with(
        &mut self,
        u_choice: &dyn AdvApproxCutting,
        v_choice: &dyn AdvApproxCutting,
        func: &dyn AdvApp2VarEvaluatorFunc2Var,
        crit: &dyn AdvApp2VarCriterion,
    ) -> Result<(), Failure> {
        let crit_abs: bool = crit.criterion_type() == AdvApp2VarCriterionType::Absolute;
        while let Some(first) = self.result.first_not_approx() {
            self.discretise_patch(first, u_choice, v_choice, func, Some(crit))?;
            let nb_int: i32 = self.result.nb_patch_in_u();
            let mut cuts: AllowedCuts = self.allowed_cuts(first, u_choice, v_choice, [1, 2])?;
            let patch = self.result.change_patch(first)?;
            patch.make_approx(
                &self.conditions,
                &self.constraints,
                if crit_abs { 0 } else { cuts.num_dec },
            )?;
            if cuts.num_dec >= 3 {
                cuts.num_dec -= 2;
            }
            if patch.has_result() {
                crit.value(patch, &self.conditions)?;
            }
            let decision: i32 = patch.cut_sense_with(crit, cuts.num_dec);
            if crit.repartition() == AdvApp2VarCriterionRepartition::Regular && decision > 0 {
                self.init_grid(nb_int + 1)?;
            } else {
                self.apply_cut(decision, first, &cuts)?;
            }
        }
        Ok(())
    }

    /// Approximates every iso not yet approximated with its end nodes, cutting the
    /// domain when an iso misses its tolerance and the patch budget allows it
    /// (`ComputeConstraints`, both overloads: with an absolute criterion the isos
    /// are never cut).
    fn compute_constraints(
        &mut self,
        u_choice: &dyn AdvApproxCutting,
        v_choice: &dyn AdvApproxCutting,
        func: &dyn AdvApp2VarEvaluatorFunc2Var,
        crit: Option<&dyn AdvApp2VarCriterion>,
    ) -> Result<(), Failure> {
        let crit_rel: bool =
            crit.is_none_or(|crit| crit.criterion_type() == AdvApp2VarCriterionType::Relative);
        let (iu, iv) = (self.conditions.u_order(), self.conditions.v_order());
        let (n1, n2) = (
            AdvApp2VarNode::with_orders(iu, iv),
            AdvApp2VarNode::with_orders(iu, iv),
        );
        while let Some((iso, ind1, ind2)) = self.constraints.first_not_approx() {
            let ind_n1: i32 = self.constraints.first_node(iso.iso_type(), ind1, ind2);
            n1.assign(self.constraints.node(ind_n1)?);
            let ind_n2: i32 = self.constraints.last_node(iso.iso_type(), ind1, ind2);
            n2.assign(self.constraints.node(ind_n2)?);
            let (u0, u1, v0, v1) = (
                self.first_par_in_u,
                self.last_par_in_u,
                self.first_par_in_v,
                self.last_par_in_v,
            );
            iso.make_approx(&self.conditions, u0, u1, v0, v1, func, &n1, &n2)?;
            if !iso.is_approximated() {
                let (nb_u, nb_v) = (self.result.nb_patch_in_u(), self.result.nb_patch_in_v());
                let is_v: bool = iso.iso_type() == IsoType::IsoV;
                let nb_patch: i32 = if is_v {
                    (nb_u + 1) * nb_v
                } else {
                    (nb_v + 1) * nb_u
                };
                let choice: &dyn AdvApproxCutting = if is_v { u_choice } else { v_choice };
                let (more, dec) = choice.value(iso.t0(), iso.t1());
                // To force Overwrite if the criterion is Absolute
                if nb_patch <= self.max_patches && more && crit_rel {
                    if is_v {
                        self.cut_in_u(dec)?
                    } else {
                        self.cut_in_v(dec)?
                    }
                    continue;
                }
                if !iso.has_result() {
                    return Err(construction(
                        "AdvApp2Var_ApproxAFunc2Var : Curve Approximation Error",
                    ));
                }
                iso.overwrite_approx();
            }
            self.constraints.change_iso(ind1, ind2, &iso)?;
            self.constraints.node(ind_n1)?.assign(&n1);
            self.constraints.node(ind_n2)?.assign(&n2);
        }
        Ok(())
    }
}
