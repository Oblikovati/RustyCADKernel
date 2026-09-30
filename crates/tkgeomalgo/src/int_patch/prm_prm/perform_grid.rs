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
// Rust translation of IntPatch_PrmPrmIntersection::Perform(Surf1, D1, Surf2, D2,
// TolTangency, Epsilon, Deflection, Increment, ClearFlag)
// (ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_PrmPrmIntersection.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The main `Perform`: the domains are sampled; a sampling of at most 2500 points on
//! each surface runs the polyhedral intersection of IntPolyh (`grid_sections.rs`),
//! a larger one the start points of `PointDepart`.
use super::IntPatchPrmPrmIntersection;
use super::helpers::surface_periods;
use super::start_params;
use super::walk_steps::{
    AcceptOptions, EndsCheck, WalkInputs, duplicated_line, is_new_start, keep_rejected,
};
use crate::int_start::TheTopolTool;
use crate::int_surf::{IntSurfLineOn2S, IntSurfPntOn2S};
use crate::int_walk::IntWalkPWalking;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;

/// The sampling of both domains: the counts and the parameters.
pub(super) struct DomainSamples {
    pub(super) counts: [i32; 4],
    pub(super) u_pars1: Array1<'static, f64>,
    pub(super) v_pars1: Array1<'static, f64>,
    pub(super) u_pars2: Array1<'static, f64>,
    pub(super) v_pars2: Array1<'static, f64>,
    pub(super) periods: [f64; 4],
}

impl DomainSamples {
    /// `SamplePnts(Deflection, 10, 10)` on both domains, then their counts, parameters,
    /// and the periods of the surfaces.
    fn new(inputs: &WalkInputs<'_>, d2: &(dyn TheTopolTool + 'static)) -> Result<Self, Failure> {
        let d1: &(dyn TheTopolTool + 'static) = inputs.d1;
        d1.sample_pnts(inputs.deflection, 10, 10)?;
        d2.sample_pnts(inputs.deflection, 10, 10)?;
        let counts: [i32; 4] = [
            d1.nb_samples_u()?,
            d1.nb_samples_v()?,
            d2.nb_samples_u()?,
            d2.nb_samples_v()?,
        ];
        Ok(Self {
            counts,
            u_pars1: d1.u_parameters()?,
            v_pars1: d1.v_parameters()?,
            u_pars2: d2.u_parameters()?,
            v_pars2: d2.v_parameters()?,
            periods: surface_periods(inputs.surf1, inputs.surf2)?,
        })
    }
}

impl IntPatchPrmPrmIntersection {
    /// `Perform(Caro1, Domain1, Caro2, Domain2, TolTangency, Epsilon, Deflection,
    /// Increment, ClearFlag)`: the intersection lines of the two surfaces, marched from
    /// the start points of the polyhedral intersection of the sampled domains (or, for
    /// more than 2500 samples on a surface, of a grid of the common box). `clear_flag`
    /// (true natively by default) clears the lines of a previous computation.
    ///
    /// ```ignore
    /// inter.perform(&s1, &*d1, &s2, &*d2, 1e-8, 1e-8, 0.1, 0.01, true)?;
    /// ```
    #[allow(clippy::too_many_arguments)]
    pub fn perform(
        &mut self,
        surf1: &Adaptor3dSurfaceHandle,
        d1: &(dyn TheTopolTool + 'static),
        surf2: &Adaptor3dSurfaceHandle,
        d2: &(dyn TheTopolTool + 'static),
        tol_tangency: f64,
        epsilon: f64,
        deflection: f64,
        increment: f64,
        clear_flag: bool,
    ) -> Result<(), Failure> {
        const LIMIT: i32 = 2500;
        let inputs = WalkInputs {
            surf1,
            d1,
            surf2,
            d2: Some(d2),
            tol_tangency,
            epsilon,
            deflection,
            increment,
        };
        let samples = DomainSamples::new(&inputs, d2)?;
        let [nbu1, nbv1, nbu2, nbv2] = samples.counts;
        if nbu1.wrapping_mul(nbv1) <= LIMIT && nbv2.wrapping_mul(nbu2) <= LIMIT {
            return self.perform_polyh(&inputs, &samples, clear_flag);
        }
        let points: Handle<IntSurfLineOn2S> = Handle::new(IntSurfLineOn2S::new(None));
        self.point_depart(&points, surf1, (nbu1, nbv1), surf2, (nbu2, nbv2))?;
        self.empt = true;
        self.done = true;
        let mut pw =
            IntWalkPWalking::new(surf1, surf2, tol_tangency, epsilon, deflection, increment)?;
        for z in 1..=points.nb_points() {
            self.walk_grid_point(&inputs, &mut pw, &points.value(z)?)?;
        }
        Ok(())
    }

    /// The walking from a start point of `PointDepart`.
    fn walk_grid_point(
        &mut self,
        inputs: &WalkInputs<'_>,
        pw: &mut IntWalkPWalking,
        point: &IntSurfPntOn2S,
    ) -> Result<(), Failure> {
        let (u1, v1, u2, v2) = point.parameters();
        let start = start_params([u1, v1, u2, v2])?;
        let Some(start_pon2s) = pw.perform_first_point(&start)? else {
            return Ok(());
        };
        let nb_lig_calculee: i32 = self.slin.len() as i32;
        if !is_new_start(inputs, &start_pon2s, &self.slin)? {
            return Ok(());
        }
        pw.perform(&start)?;
        if !pw.is_done() || pw.nb_points()? <= 2 {
            return Ok(());
        }
        let ends = (pw.value(1)?.value(), pw.value(pw.nb_points()?)?.value());
        let duplicated = duplicated_line(
            inputs,
            pw,
            &self.slin,
            (nb_lig_calculee, true, EndsCheck::Either),
        )?;
        if keep_rejected(pw, duplicated, &mut self.slin)? {
            return Ok(());
        }
        let options = AcceptOptions {
            loose: false,
            purging: None,
            several: None,
        };
        self.accept_walking(inputs, pw, &options, ends)?;
        Ok(())
    }
}
