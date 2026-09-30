// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_Curved.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_Curved`: the curved filling of four boundary pole rows, and the
//! translation surface of two.
use super::GeomFillFilling;
use super::coons::check_weights;
use super::filling::{
    Rows, at, blend, blend_terms, coords, domain_error, fill_four_sided, lengths, point_from, put2,
};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point3, Vector3};
use std::ops::{Deref, DerefMut};

/// The curved filling (`GeomFill_Curved`): with four rows, each inner pole is half
/// the bilinear blend of the rows (no corner correction); with two, the first row
/// translated along the second. The `GeomFill_Filling` result is reached through
/// `Deref`.
///
/// `let curved = GeomFillCurved::with_two_rows(&p1, &p2)?; curved.nb_v_poles()?`
#[derive(Clone, Default)]
pub struct GeomFillCurved {
    filling: GeomFillFilling,
}

impl Deref for GeomFillCurved {
    type Target = GeomFillFilling;
    fn deref(&self) -> &GeomFillFilling {
        &self.filling
    }
}

impl DerefMut for GeomFillCurved {
    fn deref_mut(&mut self) -> &mut GeomFillFilling {
        &mut self.filling
    }
}

impl GeomFillCurved {
    /// An empty filling (`GeomFill_Curved()`); `GeomFillCurved::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `GeomFill_Curved(P1, P2, P3, P4)`; `GeomFillCurved::with_poles([&p1, &p2, &p3, &p4])?`.
    pub fn with_poles(poles: Rows<'_, '_, Point3>) -> Result<Self, Failure> {
        let mut curved = Self::new();
        curved.init(poles)?;
        Ok(curved)
    }
    /// `GeomFill_Curved(P1, P2, P3, P4, W1, W2, W3, W4)`;
    /// `GeomFillCurved::with_weights([&p1, &p2, &p3, &p4], [&w1, &w2, &w3, &w4])?`.
    pub fn with_weights(
        poles: Rows<'_, '_, Point3>,
        weights: Rows<'_, '_, f64>,
    ) -> Result<Self, Failure> {
        let mut curved = Self::new();
        curved.init_weighted(poles, weights)?;
        Ok(curved)
    }
    /// `GeomFill_Curved(P1, P2)`; `GeomFillCurved::with_two_rows(&p1, &p2)?`.
    pub fn with_two_rows(
        p1: &Array1<'_, Point3>,
        p2: &Array1<'_, Point3>,
    ) -> Result<Self, Failure> {
        let mut curved = Self::new();
        curved.init_two_rows(p1, p2)?;
        Ok(curved)
    }
    /// `GeomFill_Curved(P1, P2, W1, W2)`;
    /// `GeomFillCurved::with_two_weighted_rows(&p1, &p2, &w1, &w2)?`.
    pub fn with_two_weighted_rows(
        p1: &Array1<'_, Point3>,
        p2: &Array1<'_, Point3>,
        w1: &Array1<'_, f64>,
        w2: &Array1<'_, f64>,
    ) -> Result<Self, Failure> {
        let mut curved = Self::new();
        curved.init_two_weighted_rows(p1, p2, w1, w2)?;
        Ok(curved)
    }
    /// `Init(P1, P2, P3, P4)`: the layout of `GeomFill_Stretch`, the inner poles
    /// `PV1 P1(i) + PV P3(i) + PU P2(j) + PU1 P4(j)` with every ratio halved;
    /// `curved.init([&p1, &p2, &p3, &p4])?`.
    pub fn init(&mut self, poles: Rows<'_, '_, Point3>) -> Result<(), Failure> {
        if poles[0].len() != poles[2].len() || poles[1].len() != poles[3].len() {
            return Err(domain_error("GeomFill_Curved::Init", &lengths(poles)));
        }
        let (npol_u, npol_v) = (poles[0].len() as i32, poles[1].len() as i32);
        self.filling.is_rational = false;
        let grid = self.filling.new_poles(npol_u, npol_v)?;
        fill_four_sided(grid, poles, true, |r, i, j| {
            let terms = blend_terms(poles, i, j)?.map(coords);
            Ok(point_from(|k| {
                blend(r, [terms[0][k], terms[1][k], terms[2][k], terms[3][k]])
            }))
        })
    }
    /// `Init(P1, P2, P3, P4, W1, W2, W3, W4)`: the poles of `init` and the weights
    /// blended the same way; `curved.init_weighted([&p1, &p2, &p3, &p4], [&w1, &w2, &w3, &w4])?`.
    pub fn init_weighted(
        &mut self,
        poles: Rows<'_, '_, Point3>,
        weights: Rows<'_, '_, f64>,
    ) -> Result<(), Failure> {
        check_weights("GeomFill_Curved::Init", poles, weights)?;
        self.init(poles)?;
        self.filling.is_rational = true;
        let (npol_u, npol_v) = (weights[0].len() as i32, weights[1].len() as i32);
        let grid = self.filling.new_weights(npol_u, npol_v)?;
        fill_four_sided(grid, weights, true, |r, i, j| {
            Ok(blend(r, blend_terms(weights, i, j)?))
        })
    }
    /// `Init(P1, P2)`: pole `(i, j)` is `P1(i)` translated by `P2(j) - P2(1)`;
    /// `curved.init_two_rows(&p1, &p2)?`.
    pub fn init_two_rows(
        &mut self,
        p1: &Array1<'_, Point3>,
        p2: &Array1<'_, Point3>,
    ) -> Result<(), Failure> {
        let (npol_u, npol_v) = (p1.len() as i32, p2.len() as i32);
        self.filling.is_rational = false;
        let grid = self.filling.new_poles(npol_u, npol_v)?;
        for j in 1..=npol_v {
            let tra = Vector3::between(at(p2, 1)?, at(p2, j)?);
            for i in 1..=npol_u {
                put2(grid, i, j, at(p1, i)?.translated(tra))?;
            }
        }
        Ok(())
    }
    /// `Init(P1, P2, W1, W2)`: the poles of `init_two_rows` and the weights
    /// `W1(i) W2(j) / W1(1)`, sized by the weight rows;
    /// `curved.init_two_weighted_rows(&p1, &p2, &w1, &w2)?`.
    pub fn init_two_weighted_rows(
        &mut self,
        p1: &Array1<'_, Point3>,
        p2: &Array1<'_, Point3>,
        w1: &Array1<'_, f64>,
        w2: &Array1<'_, f64>,
    ) -> Result<(), Failure> {
        self.init_two_rows(p1, p2)?;
        self.filling.is_rational = true;
        let (npol_u, npol_v) = (w1.len() as i32, w2.len() as i32);
        let grid = self.filling.new_weights(npol_u, npol_v)?;
        for j in 1..=npol_v {
            let factor: f64 = at(w2, j)? / at(w1, 1)?;
            for i in 1..=npol_u {
                put2(grid, i, j, at(w1, i)? * factor)?;
            }
        }
        Ok(())
    }
}
