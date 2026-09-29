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
// Rust translation of ModelingData/TKGeomBase/AdvApp2Var/AdvApp2Var_Network.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AdvApp2Var_Network`: the grid of patches of the approximation.
use super::patch::AdvApp2VarPatch;
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::{Failure, Handle};

/// The patches, row by row in V, and the cutting parameters in U and V
/// (`AdvApp2Var_Network`).
///
/// `let mut net = AdvApp2VarNetwork::new(patches, us, vs); net.update_in_v(0.5)?; net.nb_patch()`.
#[derive(Clone, Default)]
pub struct AdvApp2VarNetwork {
    net: Sequence<Handle<AdvApp2VarPatch>>,
    u_parameters: Sequence<f64>,
    v_parameters: Sequence<f64>,
}

impl AdvApp2VarNetwork {
    /// The network of `net` on the parameters `the_u` and `the_v`
    /// (`AdvApp2Var_Network(Net, TheU, TheV)`); the patches are shared.
    /// `AdvApp2VarNetwork::new(patches, us, vs)`.
    pub fn new(
        net: Sequence<Handle<AdvApp2VarPatch>>,
        the_u: Sequence<f64>,
        the_v: Sequence<f64>,
    ) -> Self {
        Self {
            net,
            u_parameters: the_u,
            v_parameters: the_v,
        }
    }

    /// The index of the first patch not approximated (`FirstNotApprox`);
    /// `net.first_not_approx()`.
    pub fn first_not_approx(&self) -> Option<i32> {
        (1..)
            .zip(self.net.iter())
            .find(|(_, patch)| !patch.is_approximated())
            .map(|(index, _)| index)
    }

    /// The patch of index `index` (`ChangePatch`, `operator()(Index)`); the patch is
    /// shared, its members take `&self`. `net.change_patch(1)?`.
    pub fn change_patch(&self, index: i32) -> Result<&AdvApp2VarPatch, Failure> {
        Ok(self.net.value(index.max(0) as usize)?)
    }

    /// Inserts the cut `U = cutting_value` (`UpdateInU`): the patches of the cut
    /// column are shortened and the right halves inserted after them.
    /// `net.update_in_u(0.5)?`.
    pub fn update_in_u(&mut self, cutting_value: f64) -> Result<(), Failure> {
        let mut i: usize = 1;
        while *self.u_parameters.value(i)? < cutting_value {
            i += 1;
        }
        self.u_parameters.insert_before(i, cutting_value)?;
        let nb_patch_in_u: usize = self.u_parameters.len() - 1;
        for j in 1..self.v_parameters.len() {
            let index: usize = nb_patch_in_u * (j - 1) + i - 1;
            let patch = self.net.value(index)?.clone();
            patch.change_domain(patch.u0(), cutting_value, patch.v0(), patch.v1());
            patch.reset_approx();
            let (v0, v1) = (
                *self.v_parameters.value(j)?,
                *self.v_parameters.value(j + 1)?,
            );
            let domain = [cutting_value, *self.u_parameters.value(i + 1)?, v0, v1];
            let new_patch = AdvApp2VarPatch::with_domain(domain, patch.u_order(), patch.v_order());
            new_patch.reset_approx();
            self.net.insert_after(index, Handle::new(new_patch))?;
        }
        Ok(())
    }

    /// Inserts the cut `V = cutting_value` (`UpdateInV`): the patches of the cut row
    /// are shortened and a row of upper halves inserted. `net.update_in_v(0.5)?`.
    pub fn update_in_v(&mut self, cutting_value: f64) -> Result<(), Failure> {
        let mut j: usize = 1;
        while *self.v_parameters.value(j)? < cutting_value {
            j += 1;
        }
        self.v_parameters.insert_before(j, cutting_value)?;
        let nb_patch_in_u: usize = self.u_parameters.len() - 1;
        for i in 1..=nb_patch_in_u {
            let patch = self.net.value(nb_patch_in_u * (j - 2) + i)?;
            patch.change_domain(patch.u0(), patch.u1(), patch.v0(), cutting_value);
            patch.reset_approx();
        }
        for i in 1..=nb_patch_in_u {
            let index: usize = nb_patch_in_u * (j - 1) + i - 1;
            let source = self.net.value(nb_patch_in_u * (j - 2) + i)?.clone();
            let (u0, u1) = (
                *self.u_parameters.value(i)?,
                *self.u_parameters.value(i + 1)?,
            );
            let domain = [u0, u1, cutting_value, *self.v_parameters.value(j + 1)?];
            let new_patch =
                AdvApp2VarPatch::with_domain(domain, source.u_order(), source.v_order());
            new_patch.reset_approx();
            self.net.insert_after(index, Handle::new(new_patch))?;
        }
        Ok(())
    }

    /// Raises every patch to the largest coefficient counts, at least `2*i + 2`
    /// (`SameDegree`); returns `(ncfu, ncfv)`. `net.same_degree(1, 1)`.
    pub fn same_degree(&mut self, iu: i32, iv: i32) -> (i32, i32) {
        let mut ncfu: i32 = 2 * iu + 2;
        let mut ncfv: i32 = 2 * iv + 2;
        for patch in self.net.iter() {
            ncfu = ncfu.max(patch.nb_coeff_in_u());
            ncfv = ncfv.max(patch.nb_coeff_in_v());
        }
        for patch in self.net.iter() {
            patch.change_nb_coeff(ncfu, ncfv);
        }
        (ncfu, ncfv)
    }

    /// The number of patches (`NbPatch`); `net.nb_patch()`.
    pub fn nb_patch(&self) -> i32 {
        self.net.length()
    }

    /// The number of patches in U (`NbPatchInU`); `net.nb_patch_in_u()`.
    pub fn nb_patch_in_u(&self) -> i32 {
        self.u_parameters.length() - 1
    }

    /// The number of patches in V (`NbPatchInV`); `net.nb_patch_in_v()`.
    pub fn nb_patch_in_v(&self) -> i32 {
        self.v_parameters.length() - 1
    }

    /// The U parameter of index `index` (`UParameter`); `net.u_parameter(2)?`.
    pub fn u_parameter(&self, index: i32) -> Result<f64, Failure> {
        Ok(*self.u_parameters.value(index.max(0) as usize)?)
    }

    /// The V parameter of index `index` (`VParameter`); `net.v_parameter(2)?`.
    pub fn v_parameter(&self, index: i32) -> Result<f64, Failure> {
        Ok(*self.v_parameters.value(index.max(0) as usize)?)
    }

    /// The patch `(u_index, v_index)` (`Patch`, `operator()(UIndex, VIndex)`);
    /// `net.patch(2, 1)?`.
    pub fn patch(&self, u_index: i32, v_index: i32) -> Result<&AdvApp2VarPatch, Failure> {
        let index: i32 = (v_index - 1) * (self.u_parameters.length() - 1) + u_index;
        Ok(self.net.value(index.max(0) as usize)?)
    }
}
