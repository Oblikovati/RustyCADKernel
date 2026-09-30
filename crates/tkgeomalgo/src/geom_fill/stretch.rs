// Copyright (c) 1993-1999 Matra Datavision
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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_Stretch.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_Stretch`: the bilinear (stretched) filling of four boundary pole rows.
use super::GeomFillFilling;
use super::coons::check_weights;
use super::filling::{
    Rows, blend, blend_terms, coords, corner_terms, corners, domain_error, fill_four_sided,
    lengths, point_from,
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Point3;
use std::ops::{Deref, DerefMut};

/// The stretched filling (`GeomFill_Stretch`): each inner pole is the bilinear blend
/// of the four boundary rows minus the bilinear blend of the corners; the weights are
/// half the bilinear blend of the weight rows. The `GeomFill_Filling` result is
/// reached through `Deref`.
///
/// `let stretch = GeomFillStretch::with_poles([&p1, &p2, &p3, &p4])?; stretch.nb_u_poles()?`
#[derive(Clone, Default)]
pub struct GeomFillStretch {
    filling: GeomFillFilling,
}

impl Deref for GeomFillStretch {
    type Target = GeomFillFilling;
    fn deref(&self) -> &GeomFillFilling {
        &self.filling
    }
}

impl DerefMut for GeomFillStretch {
    fn deref_mut(&mut self) -> &mut GeomFillFilling {
        &mut self.filling
    }
}

impl GeomFillStretch {
    /// An empty filling (`GeomFill_Stretch()`); `GeomFillStretch::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `GeomFill_Stretch(P1, P2, P3, P4)`; `GeomFillStretch::with_poles([&p1, &p2, &p3, &p4])?`.
    pub fn with_poles(poles: Rows<'_, '_, Point3>) -> Result<Self, Failure> {
        let mut stretch = Self::new();
        stretch.init(poles)?;
        Ok(stretch)
    }
    /// `GeomFill_Stretch(P1, P2, P3, P4, W1, W2, W3, W4)`;
    /// `GeomFillStretch::with_weights([&p1, &p2, &p3, &p4], [&w1, &w2, &w3, &w4])?`.
    pub fn with_weights(
        poles: Rows<'_, '_, Point3>,
        weights: Rows<'_, '_, f64>,
    ) -> Result<Self, Failure> {
        let mut stretch = Self::new();
        stretch.init_weighted(poles, weights)?;
        Ok(stretch)
    }
    /// `Init(P1, P2, P3, P4)`: `P1` and `P3` are the first and last columns, `P4`
    /// and `P2` the first and last rows (the transpose of the Coons layout); opposite
    /// rows of different lengths raise Standard_DomainError;
    /// `stretch.init([&p1, &p2, &p3, &p4])?`.
    pub fn init(&mut self, poles: Rows<'_, '_, Point3>) -> Result<(), Failure> {
        if poles[0].len() != poles[2].len() || poles[1].len() != poles[3].len() {
            return Err(domain_error("GeomFill_Stretch::Init", &lengths(poles)));
        }
        let (npol_u, npol_v) = (poles[0].len() as i32, poles[1].len() as i32);
        self.filling.is_rational = false;
        let grid = self.filling.new_poles(npol_u, npol_v)?;
        fill_four_sided(grid, poles, false, |r, i, j| {
            let terms = blend_terms(poles, i, j)?.map(coords);
            let corner = corner_terms(poles)?.map(coords);
            Ok(point_from(|k| {
                let edge = [terms[0][k], terms[1][k], terms[2][k], terms[3][k]];
                let corner = [corner[0][k], corner[1][k], corner[2][k], corner[3][k]];
                blend(r, edge) - corners(r, corner)
            }))
        })
    }
    /// `Init(P1, P2, P3, P4, W1, W2, W3, W4)`: the poles of `init` and the weights
    /// `0.5 (PV1 W1(i) + PV W3(i) + PU W2(j) + PU1 W4(j))`; the weight rows must match
    /// the pole rows (Standard_DomainError);
    /// `stretch.init_weighted([&p1, &p2, &p3, &p4], [&w1, &w2, &w3, &w4])?`.
    pub fn init_weighted(
        &mut self,
        poles: Rows<'_, '_, Point3>,
        weights: Rows<'_, '_, f64>,
    ) -> Result<(), Failure> {
        check_weights("GeomFill_Stretch::Init", poles, weights)?;
        self.init(poles)?;
        self.filling.is_rational = true;
        let (npol_u, npol_v) = (weights[0].len() as i32, weights[1].len() as i32);
        let grid = self.filling.new_weights(npol_u, npol_v)?;
        fill_four_sided(grid, weights, false, |r, i, j| {
            Ok(0.5 * blend(r, blend_terms(weights, i, j)?))
        })
    }
}
