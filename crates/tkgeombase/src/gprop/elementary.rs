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
// Rust translation of the inertia assembly shared by GProp_CelGProps.cxx,
// GProp_SelGProps.cxx and GProp_VelGProps.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The inertia assembly that the elementary curve, surface and solid frameworks
//! repeat after computing their local inertia matrix `Dm`.
use super::{GProps, h_operator};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Direction3, Matrix3, Xyz};
use rustycad_tkmath::math::{Jacobi, Matrix, Vector};

/// The local frame of an elementary entity: `(X, Y, Z)` directions whose
/// coordinates fill the columns of the native `Passage` matrix.
pub(super) type Frame = [Direction3; 3];

/// The symmetric local inertia matrix `Dm` from its diagonal and its
/// off-diagonal terms `(12, 13, 23)`.
pub(super) fn local_inertia(diagonal: [f64; 3], off: [f64; 3]) -> [[f64; 3]; 3] {
    [
        [diagonal[0], off[0], off[1]],
        [off[0], diagonal[1], off[2]],
        [off[1], off[2], diagonal[2]],
    ]
}

/// The native tail of every elementary `Perform`: diagonalize `Dm` with
/// math_Jacobi, map each eigenvector through `Passage` and scale it by
/// `factor * eigenvalue` (the vectors become the columns' rows of the inertia),
/// then move the result to the location with the Huygens operator.
///
/// `V.Multiply(Passage, V)` aliases its operand: each component is written before
/// the next row reads it, so rows 2 and 3 see the updated first components; the
/// port reproduces that sequential product. StdFail_NotDone when Jacobi fails.
pub(super) fn assemble_inertia(
    props: &mut GProps,
    dm: [[f64; 3]; 3],
    frame: Frame,
    factor: f64,
) -> Result<(), Failure> {
    let mut matrix: Matrix<'static> = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    for (i, row) in dm.iter().enumerate() {
        for (j, value) in row.iter().enumerate() {
            *matrix.change_value(i as i32 + 1, j as i32 + 1)? = *value;
        }
    }
    let jacobi = Jacobi::new(&matrix)?;
    let passage: [[f64; 3]; 3] = passage_rows(frame);
    let mut columns: [[f64; 3]; 3] = [[0.0; 3]; 3];
    for (k, column) in columns.iter_mut().enumerate() {
        let index: i32 = k as i32 + 1;
        let mut vector: Vector<'static> = Vector::with_bounds(1, 3)?;
        jacobi.vector(index, &mut vector)?;
        let mut v: [f64; 3] = [vector[1], vector[2], vector[3]];
        multiply_aliased(&passage, &mut v);
        let scale: f64 = factor * jacobi.value(index)?;
        *column = v.map(|x| x * scale);
    }
    let [v1, v2, v3] = columns;
    props.inertia = Matrix3::from_columns(
        Xyz::new(v1[0], v2[0], v3[0]),
        Xyz::new(v1[1], v2[1], v3[1]),
        Xyz::new(v1[2], v2[2], v3[2]),
    );
    props.inertia += h_operator(props.g, props.loc, props.dim);
    Ok(())
}

// Passage(r, c) is coordinate r of frame direction c.
fn passage_rows(frame: Frame) -> [[f64; 3]; 3] {
    let [x, y, z] = frame.map(|d| d.xyz());
    [
        [x.x(), y.x(), z.x()],
        [x.y(), y.y(), z.y()],
        [x.z(), y.z(), z.z()],
    ]
}

// math_VectorBase::Multiply(Left, Right) with Right aliasing the result: each
// row sum starts at 0.0 and is stored before the next row reads the operand.
fn multiply_aliased(passage: &[[f64; 3]; 3], v: &mut [f64; 3]) {
    for (r, row) in passage.iter().enumerate() {
        let mut sum: f64 = 0.0;
        for (c, coefficient) in row.iter().enumerate() {
            sum += coefficient * v[c];
        }
        v[r] = sum;
    }
}
