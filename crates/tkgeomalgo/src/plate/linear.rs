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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/Plate/Plate_PinpointConstraint.hxx/.lxx/.cxx,
// Plate_LinearXYZConstraint.hxx/.lxx/.cxx and Plate_LinearScalarConstraint.hxx/.lxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The pinpoint constraint and the two linear combinations of pinpoint constraints.
use rustycad_tkernel::ncollection::{Array1, Array2, HArray1, HArray2};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkmath::gp::{Xy, Xyz};

/// A value (or derivative `D^(idu,idv)`) imposed at a parameter point
/// (`Plate_PinpointConstraint`).
///
/// `PlatePinpointConstraint::new(Xy::new(0.0, 0.0), Xyz::new(0.0, 0.0, 1.0), 0, 0)`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PlatePinpointConstraint {
    value: Xyz,
    pnt2d: Xy,
    idu: i32,
    idv: i32,
}

impl PlatePinpointConstraint {
    /// The derivative `D^(iu,iv)` equal to `imposed_value` at `point2d`
    /// (`Plate_PinpointConstraint(point2d, ImposedValue, iu, iv)`; the native defaults of
    /// `iu`, `iv` are 0). `PlatePinpointConstraint::new(uv, value, 0, 0)`.
    pub const fn new(point2d: Xy, imposed_value: Xyz, iu: i32, iv: i32) -> Self {
        Self {
            value: imposed_value,
            pnt2d: point2d,
            idu: iu,
            idv: iv,
        }
    }

    /// The value 0 at the origin (`Plate_PinpointConstraint()`); `PlatePinpointConstraint::origin()`.
    pub fn origin() -> Self {
        Self::default()
    }

    /// The parameter point (`Pnt2d`); `c.pnt2d()`.
    pub const fn pnt2d(&self) -> Xy {
        self.pnt2d
    }

    /// The derivation order in U (`Idu`); `c.idu()`.
    pub const fn idu(&self) -> i32 {
        self.idu
    }

    /// The derivation order in V (`Idv`); `c.idv()`.
    pub const fn idv(&self) -> i32 {
        self.idv
    }

    /// The imposed value (`Value`); `c.value()`.
    pub const fn value(&self) -> Xyz {
        self.value
    }
}

/// A matrix of coefficients over pinpoint constraints, shared through handles as the
/// native `myPPC` and `myCoef`: copies of a constraint share them, so a `SetPPC` or
/// `SetCoeff` after a copy is seen by the copy. `C` is `f64` for the XYZ constraints and
/// `Xyz` for the scalar ones.
#[derive(Clone)]
pub(crate) struct LinearCombination<C: Copy + Default + Send + Sync + 'static> {
    ppc: Handle<HArray1<PlatePinpointConstraint>>,
    coef: Handle<HArray2<C>>,
}

impl<C: Copy + Default + Send + Sync + 'static> Default for LinearCombination<C> {
    fn default() -> Self {
        Self {
            ppc: Handle::default(),
            coef: Handle::default(),
        }
    }
}

fn dimension_mismatch(class: &str, coefficients: usize, constraints: usize) -> Failure {
    Failure::with_message(
        "Standard_DimensionMismatch",
        &format!(
            "{class}: {coefficients} coefficients per row for {constraints} constraints; expected one per constraint"
        ),
    )
}

fn null_arrays(class: &str) -> Failure {
    errors::null_object(&format!(
        "{class}: default-constructed constraint without arrays"
    ))
}

impl<C: Copy + Default + Send + Sync + 'static> LinearCombination<C> {
    /// One row of coefficients, re-indexed from 1.
    pub(crate) fn from_row(
        class: &str,
        ppc: &Array1<'_, PlatePinpointConstraint>,
        coeff: &Array1<'_, C>,
    ) -> Result<Self, Failure> {
        if coeff.len() != ppc.len() {
            return Err(dimension_mismatch(class, coeff.len(), ppc.len()));
        }
        let mut row: Array2<'static, C> =
            Array2::with_bounds(1, 1, 1, coeff.len() as i32).map_err(errors::array_bounds)?;
        row.as_mut_slice().copy_from_slice(coeff.as_slice());
        Ok(Self::from_owned(ppc, row))
    }

    /// A matrix of coefficients, re-indexed from 1.
    pub(crate) fn from_matrix(
        class: &str,
        ppc: &Array1<'_, PlatePinpointConstraint>,
        coeff: &Array2<'_, C>,
    ) -> Result<Self, Failure> {
        if coeff.nb_columns() != ppc.len() {
            return Err(dimension_mismatch(class, coeff.nb_columns(), ppc.len()));
        }
        let (rows, cols) = (coeff.nb_rows() as i32, coeff.nb_columns() as i32);
        let mut matrix: Array2<'static, C> =
            Array2::with_bounds(1, rows, 1, cols).map_err(errors::array_bounds)?;
        matrix.as_mut_slice().copy_from_slice(coeff.as_slice());
        Ok(Self::from_owned(ppc, matrix))
    }

    fn from_owned(ppc: &Array1<'_, PlatePinpointConstraint>, coef: Array2<'static, C>) -> Self {
        let mut constraints: Array1<'static, PlatePinpointConstraint> =
            Array1::with_size(ppc.len());
        constraints.as_mut_slice().copy_from_slice(ppc.as_slice());
        Self {
            ppc: Handle::new(HArray1::from_owned(constraints)),
            coef: Handle::new(HArray2::from_owned(coef)),
        }
    }

    /// `ColLen x RowLen` zero coefficients and `RowLen` default constraints.
    pub(crate) fn with_size(col_len: i32, row_len: i32) -> Result<Self, Failure> {
        let ppc = HArray1::with_bounds(1, row_len).map_err(errors::array_bounds)?;
        let coef = HArray2::with_value(1, col_len, 1, row_len, C::default())
            .map_err(errors::array_bounds)?;
        Ok(Self {
            ppc: Handle::new(ppc),
            coef: Handle::new(coef),
        })
    }

    fn arrays(
        &self,
        class: &str,
    ) -> Result<(&HArray1<PlatePinpointConstraint>, &HArray2<C>), Failure> {
        match (self.ppc.get(), self.coef.get()) {
            (Some(ppc), Some(coef)) => Ok((ppc, coef)),
            _ => Err(null_arrays(class)),
        }
    }

    /// Copies of the constraints and coefficients.
    pub(crate) fn snapshot(
        &self,
        class: &str,
    ) -> Result<(Array1<'static, PlatePinpointConstraint>, Array2<'static, C>), Failure> {
        let (ppc, coef) = self.arrays(class)?;
        let constraints = ppc.array1().clone();
        let coefficients = coef.array2().clone();
        Ok((constraints, coefficients))
    }

    pub(crate) fn set_ppc(
        &self,
        class: &str,
        index: i32,
        value: PlatePinpointConstraint,
    ) -> Result<(), Failure> {
        let (ppc, _) = self.arrays(class)?;
        ppc.change_array1()
            .set_value(index, value)
            .map_err(errors::array_bounds)
    }

    pub(crate) fn set_coeff(
        &self,
        class: &str,
        row: i32,
        col: i32,
        value: C,
    ) -> Result<(), Failure> {
        let (_, coef) = self.arrays(class)?;
        coef.change_array2()
            .set_value(row, col, value)
            .map_err(errors::array_bounds)
    }
}

const LXYZ: &str = "Plate_LinearXYZConstraint";
const LSC: &str = "Plate_LinearScalarConstraint";

/// Linear combinations (rows of real coefficients) of pinpoint constraints imposed on
/// the three coordinates (`Plate_LinearXYZConstraint`).
///
/// `let c = PlateLinearXYZConstraint::with_size(1, 2)?; c.set_coeff(1, 1, 1.0)?;`
#[derive(Clone, Default)]
pub struct PlateLinearXYZConstraint {
    combination: LinearCombination<f64>,
}

impl PlateLinearXYZConstraint {
    /// A constraint without arrays (`Plate_LinearXYZConstraint()`); a plate raises
    /// Standard_NullObject when it is loaded. `PlateLinearXYZConstraint::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// One row of coefficients (`Plate_LinearXYZConstraint(PPC, Coeff)` with an array);
    /// Standard_DimensionMismatch unless there is one coefficient per constraint.
    /// `PlateLinearXYZConstraint::from_row(&ppc, &coeff)?`.
    pub fn from_row(
        ppc: &Array1<'_, PlatePinpointConstraint>,
        coeff: &Array1<'_, f64>,
    ) -> Result<Self, Failure> {
        Ok(Self {
            combination: LinearCombination::from_row(LXYZ, ppc, coeff)?,
        })
    }

    /// A matrix of coefficients, one column per constraint
    /// (`Plate_LinearXYZConstraint(PPC, Coeff)` with a matrix); Standard_DimensionMismatch
    /// otherwise. `PlateLinearXYZConstraint::from_matrix(&ppc, &coeff)?`.
    pub fn from_matrix(
        ppc: &Array1<'_, PlatePinpointConstraint>,
        coeff: &Array2<'_, f64>,
    ) -> Result<Self, Failure> {
        Ok(Self {
            combination: LinearCombination::from_matrix(LXYZ, ppc, coeff)?,
        })
    }

    /// `col_len` rows of `row_len` zero coefficients over default constraints
    /// (`Plate_LinearXYZConstraint(ColLen, RowLen)`). `PlateLinearXYZConstraint::with_size(1, 2)?`.
    pub fn with_size(col_len: i32, row_len: i32) -> Result<Self, Failure> {
        Ok(Self {
            combination: LinearCombination::with_size(col_len, row_len)?,
        })
    }

    /// A copy of the constraints (`GetPPC`); Standard_NullObject without arrays.
    /// `c.get_ppc()?`.
    pub fn get_ppc(&self) -> Result<Array1<'static, PlatePinpointConstraint>, Failure> {
        Ok(self.combination.snapshot(LXYZ)?.0)
    }

    /// A copy of the coefficients (`Coeff`); Standard_NullObject without arrays.
    /// `c.coeff()?`.
    pub fn coeff(&self) -> Result<Array2<'static, f64>, Failure> {
        Ok(self.combination.snapshot(LXYZ)?.1)
    }

    /// Replaces constraint `index` (`SetPPC`) in the shared array; Standard_OutOfRange
    /// for a bad index. `c.set_ppc(1, ppc)?`.
    pub fn set_ppc(&self, index: i32, value: PlatePinpointConstraint) -> Result<(), Failure> {
        self.combination.set_ppc(LXYZ, index, value)
    }

    /// Replaces coefficient `(row, col)` (`SetCoeff`) in the shared matrix;
    /// Standard_OutOfRange for a bad index. `c.set_coeff(1, 2, 0.5)?`.
    pub fn set_coeff(&self, row: i32, col: i32, value: f64) -> Result<(), Failure> {
        self.combination.set_coeff(LXYZ, row, col, value)
    }

    pub(crate) fn snapshot(
        &self,
    ) -> Result<
        (
            Array1<'static, PlatePinpointConstraint>,
            Array2<'static, f64>,
        ),
        Failure,
    > {
        self.combination.snapshot(LXYZ)
    }
}

/// Linear combinations (rows of `gp_XYZ` coefficients dotted with the values) of
/// pinpoint constraints, each row a scalar equation (`Plate_LinearScalarConstraint`).
///
/// `PlateLinearScalarConstraint::single(ppc, Xyz::new(0.0, 0.0, 1.0))`.
#[derive(Clone, Default)]
pub struct PlateLinearScalarConstraint {
    combination: LinearCombination<Xyz>,
}

impl PlateLinearScalarConstraint {
    /// A constraint without arrays (`Plate_LinearScalarConstraint()`).
    /// `PlateLinearScalarConstraint::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// One constraint with one coefficient (`Plate_LinearScalarConstraint(PPC1, Coeff)`).
    /// `PlateLinearScalarConstraint::single(ppc, normal)`.
    pub fn single(ppc1: PlatePinpointConstraint, coeff: Xyz) -> Self {
        let combination = LinearCombination::with_size(1, 1)
            .unwrap_or_else(|_| unreachable!("a 1x1 constraint has valid bounds"));
        let set = combination
            .set_ppc(LSC, 1, ppc1)
            .and_then(|()| combination.set_coeff(LSC, 1, 1, coeff));
        set.unwrap_or_else(|_| unreachable!("index 1 is inside a 1x1 constraint"));
        Self { combination }
    }

    /// One row of coefficients (`Plate_LinearScalarConstraint(PPC, Coeff)` with an
    /// array); Standard_DimensionMismatch unless there is one coefficient per constraint.
    /// `PlateLinearScalarConstraint::from_row(&ppc, &coeff)?`.
    pub fn from_row(
        ppc: &Array1<'_, PlatePinpointConstraint>,
        coeff: &Array1<'_, Xyz>,
    ) -> Result<Self, Failure> {
        Ok(Self {
            combination: LinearCombination::from_row(LSC, ppc, coeff)?,
        })
    }

    /// A matrix of coefficients (`Plate_LinearScalarConstraint(PPC, Coeff)` with a
    /// matrix). `PlateLinearScalarConstraint::from_matrix(&ppc, &coeff)?`.
    pub fn from_matrix(
        ppc: &Array1<'_, PlatePinpointConstraint>,
        coeff: &Array2<'_, Xyz>,
    ) -> Result<Self, Failure> {
        Ok(Self {
            combination: LinearCombination::from_matrix(LSC, ppc, coeff)?,
        })
    }

    /// `col_len` rows of `row_len` null coefficients
    /// (`Plate_LinearScalarConstraint(ColLen, RowLen)`). `PlateLinearScalarConstraint::with_size(2, 1)?`.
    pub fn with_size(col_len: i32, row_len: i32) -> Result<Self, Failure> {
        Ok(Self {
            combination: LinearCombination::with_size(col_len, row_len)?,
        })
    }

    /// A copy of the constraints (`GetPPC`); `c.get_ppc()?`.
    pub fn get_ppc(&self) -> Result<Array1<'static, PlatePinpointConstraint>, Failure> {
        Ok(self.combination.snapshot(LSC)?.0)
    }

    /// A copy of the coefficients (`Coeff`); `c.coeff()?`.
    pub fn coeff(&self) -> Result<Array2<'static, Xyz>, Failure> {
        Ok(self.combination.snapshot(LSC)?.1)
    }

    /// Replaces constraint `index` (`SetPPC`); `c.set_ppc(1, ppc)?`.
    pub fn set_ppc(&self, index: i32, value: PlatePinpointConstraint) -> Result<(), Failure> {
        self.combination.set_ppc(LSC, index, value)
    }

    /// Replaces coefficient `(row, col)` (`SetCoeff`); `c.set_coeff(1, 1, normal)?`.
    pub fn set_coeff(&self, row: i32, col: i32, value: Xyz) -> Result<(), Failure> {
        self.combination.set_coeff(LSC, row, col, value)
    }

    pub(crate) fn snapshot(
        &self,
    ) -> Result<
        (
            Array1<'static, PlatePinpointConstraint>,
            Array2<'static, Xyz>,
        ),
        Failure,
    > {
        self.combination.snapshot(LSC)
    }
}
