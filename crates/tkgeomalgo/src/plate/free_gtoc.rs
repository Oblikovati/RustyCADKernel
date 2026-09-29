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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/Plate/Plate_FreeGtoCConstraint.hxx/.lxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Plate_FreeGtoCConstraint`: G1/G2/G3 matching of a free plate onto a target
//! surface, the highest order imposed along the target normal only.
use super::gtoc_math::{COSMIN, G2Frame, g1_offsets, incremental_rotation, projected, unit_normal};
use super::{PlateD1, PlateD2, PlateD3, PlateLinearScalarConstraint, PlatePinpointConstraint};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkmath::gp::{Transform3, Xy, Xyz};

/// The free G1/G2/G3 matching constraints at a point (`Plate_FreeGtoCConstraint`): up
/// to five pinpoint constraints and four scalar constraints along the target normal.
/// `IncrementalLoad` rotates the target frame from the source normal by that fraction
/// of their angle; `orientation` picks the target normal side (0: the nearer one).
///
/// `let c = PlateFreeGtoCConstraint::g1(uv, &d1s, &d1t, 1.0, 0)?; c.nb_lsc()`.
#[derive(Clone)]
pub struct PlateFreeGtoCConstraint {
    pnt2d: Xy,
    nb_pp_constraints: i32,
    nb_ls_constraints: i32,
    my_ppc: [PlatePinpointConstraint; 5],
    my_lsc: [PlateLinearScalarConstraint; 4],
}

/// The target derivatives, rotated in place by the incremental load.
struct Target {
    normale: Xyz,
    d1: PlateD1,
    d2: Option<PlateD2>,
    d3: Option<PlateD3>,
}

impl Target {
    fn rotate(&mut self, rota: Transform3) {
        let turn = |v: &mut Xyz| *v = rota.apply_coordinates(*v);
        turn(&mut self.normale);
        turn(&mut self.d1.du);
        turn(&mut self.d1.dv);
        if let Some(d2) = self.d2.as_mut() {
            turn(&mut d2.duu);
            turn(&mut d2.duv);
            turn(&mut d2.dvv);
        }
        if let Some(d3) = self.d3.as_mut() {
            turn(&mut d3.duuu);
            turn(&mut d3.duuv);
            turn(&mut d3.duvv);
            turn(&mut d3.dvvv);
        }
    }
}

impl PlateFreeGtoCConstraint {
    fn empty(point2d: Xy) -> Self {
        Self {
            pnt2d: point2d,
            nb_pp_constraints: 0,
            nb_ls_constraints: 0,
            my_ppc: [PlatePinpointConstraint::default(); 5],
            my_lsc: Default::default(),
        }
    }

    fn push_ppc(&mut self, value: Xyz, iu: i32, iv: i32) {
        self.my_ppc[self.nb_pp_constraints as usize] =
            PlatePinpointConstraint::new(self.pnt2d, value, iu, iv);
        self.nb_pp_constraints += 1;
    }

    /// Stores scalar constraints `value . normale` from slot 0.
    fn set_lsc(&mut self, values: &[(Xyz, i32, i32)], normale: Xyz) {
        for (slot, &(value, iu, iv)) in values.iter().enumerate() {
            let ppc = PlatePinpointConstraint::new(self.pnt2d, value, iu, iv);
            self.my_lsc[slot] = PlateLinearScalarConstraint::single(ppc, normale);
        }
        self.nb_ls_constraints = values.len() as i32;
    }

    /// The G1 scalar constraints `-D1S . normale`, used when the normals cannot be
    /// matched otherwise.
    fn set_g1_lsc(&mut self, d1s: &PlateD1, normale: Xyz) {
        let (du, dv) = (d1s.du * -1.0, d1s.dv * -1.0);
        self.set_lsc(&[(du, 1, 0), (dv, 0, 1)], normale);
    }

    /// G1 matching (`Plate_FreeGtoCConstraint(point2d, D1S, D1T, IncrementalLoad,
    /// orientation)`; the native defaults are 1 and 0). Standard_ConstructionError
    /// when the incremental load meets parallel normals (null rotation axis).
    /// `PlateFreeGtoCConstraint::g1(uv, &d1s, &d1t, 1.0, 0)?`.
    pub fn g1(
        point2d: Xy,
        d1s: &PlateD1,
        d1t: &PlateD1,
        incremental_load: f64,
        orientation: i32,
    ) -> Result<Self, Failure> {
        let mut constraint = Self::empty(point2d);
        let Some(mut normale) = unit_normal(d1t) else {
            return Ok(constraint);
        };
        if incremental_load != 1.0 {
            let Some(n0) = unit_normal(d1s) else {
                return Ok(constraint);
            };
            let Some(rota) = incremental_rotation(n0, normale, incremental_load, orientation)?
            else {
                return Ok(constraint);
            };
            normale = rota.apply_coordinates(normale);
        }
        constraint.set_g1_lsc(d1s, normale);
        Ok(constraint)
    }

    /// The common start of the G2 and G3 constructors: the rotated target, or the
    /// G1 fallback. Returns `None` when the constraint is complete.
    fn start_higher(
        &mut self,
        d1s: &PlateD1,
        mut target: Target,
        incremental_load: f64,
        orientation: i32,
    ) -> Result<Option<(Target, Xyz, f64)>, Failure> {
        let Some(normale_s) = unit_normal(d1s) else {
            if incremental_load == 1.0 {
                self.set_g1_lsc(d1s, target.normale);
            }
            return Ok(None);
        };
        if incremental_load != 1.0 {
            let Some(rota) =
                incremental_rotation(normale_s, target.normale, incremental_load, orientation)?
            else {
                return Ok(None);
            };
            target.rotate(rota);
        }
        let cos_normales: f64 = target.normale * normale_s;
        if cos_normales.abs() < COSMIN {
            self.set_g1_lsc(d1s, target.normale);
            return Ok(None);
        }
        let invcos: f64 = 1.0 / cos_normales;
        let (du, dv) = g1_offsets(normale_s, target.normale, d1s, invcos);
        self.push_ppc(du, 1, 0);
        self.push_ppc(dv, 0, 1);
        Ok(Some((target, normale_s, invcos)))
    }

    /// G1 and G2 matching, the second derivatives along the target normal
    /// (`Plate_FreeGtoCConstraint(point2d, D1S, D1T, D2S, D2T, IncrementalLoad,
    /// orientation)`). `PlateFreeGtoCConstraint::g2(uv, (&d1s, &d1t), (&d2s, &d2t), 1.0, 0)?`.
    pub fn g2(
        point2d: Xy,
        (d1s, d1t0): (&PlateD1, &PlateD1),
        (d2s, d2t0): (&PlateD2, &PlateD2),
        incremental_load: f64,
        orientation: i32,
    ) -> Result<Self, Failure> {
        let mut constraint = Self::empty(point2d);
        let Some(normale) = unit_normal(d1t0) else {
            return Ok(constraint);
        };
        let target = Target {
            normale,
            d1: *d1t0,
            d2: Some(*d2t0),
            d3: None,
        };
        let Some((target, normale_s, invcos)) =
            constraint.start_higher(d1s, target, incremental_load, orientation)?
        else {
            return Ok(constraint);
        };
        let g1 = g1_offsets(normale_s, target.normale, d1s, invcos);
        let Some(frame) = G2Frame::new(d1s, g1, &target.d1)? else {
            return Ok(constraint);
        };
        let d2t: PlateD2 = target.d2.unwrap_or_else(|| unreachable!("set above"));
        let (suu, suv, svv) = frame.second(&d2t);
        let mut duu: Xyz = suu - d2s.duu;
        let mut duv: Xyz = suv - d2s.duv;
        let mut dvv: Xyz = svv - d2s.dvv;
        duu *= incremental_load;
        duv *= incremental_load;
        dvv *= incremental_load;
        constraint.set_lsc(&[(duu, 2, 0), (duv, 1, 1), (dvv, 0, 2)], target.normale);
        Ok(constraint)
    }

    /// G1, G2 and G3 matching, the third derivatives along the target normal
    /// (`Plate_FreeGtoCConstraint(point2d, D1S, D1T, D2S, D2T, D3S, D3T,
    /// IncrementalLoad, orientation)`).
    /// `PlateFreeGtoCConstraint::g3(uv, (&d1s, &d1t), (&d2s, &d2t), (&d3s, &d3t), 1.0, 0)?`.
    pub fn g3(
        point2d: Xy,
        (d1s, d1t0): (&PlateD1, &PlateD1),
        (d2s, d2t0): (&PlateD2, &PlateD2),
        (d3s, d3t0): (&PlateD3, &PlateD3),
        incremental_load: f64,
        orientation: i32,
    ) -> Result<Self, Failure> {
        let mut constraint = Self::empty(point2d);
        let Some(normale) = unit_normal(d1t0) else {
            return Ok(constraint);
        };
        let target = Target {
            normale,
            d1: *d1t0,
            d2: Some(*d2t0),
            d3: Some(*d3t0),
        };
        let Some((target, normale_s, invcos)) =
            constraint.start_higher(d1s, target, incremental_load, orientation)?
        else {
            return Ok(constraint);
        };
        let g1 = g1_offsets(normale_s, target.normale, d1s, invcos);
        let Some(frame) = G2Frame::new(d1s, g1, &target.d1)? else {
            return Ok(constraint);
        };
        let (d2t, d3t) = match (target.d2, target.d3) {
            (Some(d2t), Some(d3t)) => (d2t, d3t),
            _ => unreachable!("set above"),
        };
        let second = frame.second(&d2t);
        let corrected = (
            projected(normale_s, target.normale, second.0 - d2s.duu, invcos),
            projected(normale_s, target.normale, second.1 - d2s.duv, invcos),
            projected(normale_s, target.normale, second.2 - d2s.dvv, invcos),
        );
        constraint.push_ppc(corrected.0, 2, 0);
        constraint.push_ppc(corrected.1, 1, 1);
        constraint.push_ppc(corrected.2, 0, 2);
        let s = frame.third(d2s, corrected, second, (&d2t, &d3t))?;
        let mut d = [
            s[0] - d3s.duuu,
            s[1] - d3s.duuv,
            s[2] - d3s.duvv,
            s[3] - d3s.dvvv,
        ];
        for value in &mut d {
            *value *= incremental_load;
        }
        constraint.set_lsc(
            &[(d[0], 3, 0), (d[1], 2, 1), (d[2], 1, 2), (d[3], 0, 3)],
            target.normale,
        );
        Ok(constraint)
    }

    /// The number of pinpoint constraints: 0, 2 or 5 (`nb_PPC`); `c.nb_ppc()`.
    pub const fn nb_ppc(&self) -> i32 {
        self.nb_pp_constraints
    }

    /// Pinpoint constraint `index` of the fixed array of five (`GetPPC`);
    /// Standard_OutOfRange outside `0..5`, where the native read is undefined.
    /// `c.get_ppc(0)?`.
    pub fn get_ppc(&self, index: i32) -> Result<PlatePinpointConstraint, Failure> {
        let slot = usize::try_from(index)
            .ok()
            .filter(|&i| i < self.my_ppc.len());
        slot.map(|i| self.my_ppc[i]).ok_or_else(|| {
            errors::out_of_range(&format!(
                "Plate_FreeGtoCConstraint::GetPPC: Index = {index}; expected in [0, 4]"
            ))
        })
    }

    /// The number of scalar constraints: 0, 2, 3 or 4 (`nb_LSC`); `c.nb_lsc()`.
    pub const fn nb_lsc(&self) -> i32 {
        self.nb_ls_constraints
    }

    /// Scalar constraint `index` of the fixed array of four (`LSC`; unused entries have
    /// no arrays); Standard_OutOfRange outside `0..4`. `c.lsc(0)?`.
    pub fn lsc(&self, index: i32) -> Result<&PlateLinearScalarConstraint, Failure> {
        let slot = usize::try_from(index)
            .ok()
            .filter(|&i| i < self.my_lsc.len());
        slot.map(|i| &self.my_lsc[i]).ok_or_else(|| {
            errors::out_of_range(&format!(
                "Plate_FreeGtoCConstraint::LSC: Index = {index}; expected in [0, 3]"
            ))
        })
    }
}
