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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/Plate/Plate_Plate.cxx (fillXYZmatrix and the
// matrix blocks of SolveTI3) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The blocks of the plate systems: the kernel rows of the pinpoint and linear
//! constraints and the polynomial rows.
use super::{PlatePinpointConstraint, PlatePlate};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Xy;
use rustycad_tkmath::math::{Matrix, Vector};

/// A linear constraint read once at solve time: its pinpoint constraints and its
/// coefficients (`ColLength` rows of `RowLength` columns, one-based). `C` is `f64` for
/// the XYZ constraints and `Xyz` for the scalar ones.
pub(super) struct Block<C: 'static> {
    pub(super) ppc: Array1<'static, PlatePinpointConstraint>,
    pub(super) coef: Array2<'static, C>,
}

impl<C: Copy + 'static> Block<C> {
    /// `ColLength`, the number of equations.
    pub(super) fn col_length(&self) -> i32 {
        self.coef.nb_rows() as i32
    }

    /// `RowLength`, the number of pinpoint constraints.
    pub(super) fn row_length(&self) -> i32 {
        self.coef.nb_columns() as i32
    }

    pub(super) fn coeff(&self, row: i32, col: i32) -> C {
        self.coef[(row, col)]
    }

    pub(super) fn ppc(&self, index: i32) -> PlatePinpointConstraint {
        self.ppc[index]
    }
}

/// The sign `(-1)^(Deru + Derv)` of the adjoint kernel.
fn signe(deru: i32, derv: i32) -> f64 {
    if (deru + derv) % 2 == 1 { -1.0 } else { 1.0 }
}

/// The constraint points and derivation orders of a solve (`points`, `deru`, `derv`).
pub(super) struct Nodes<'a> {
    pub(super) points: &'a [Xy],
    pub(super) deru: &'a [i32],
    pub(super) derv: &'a [i32],
}

impl Nodes<'_> {
    /// `signe_j * SolEm(P_i - P_j, Deru_i + Deru_j, Derv_i + Derv_j)`, the kernel
    /// between nodes `i` and `j`.
    pub(super) fn kernel(&self, plate: &PlatePlate, i: usize, j: usize) -> Result<f64, Failure> {
        let iu: i32 = self.deru[i] + self.deru[j];
        let iv: i32 = self.derv[i] + self.derv[j];
        Ok(signe(self.deru[j], self.derv[j])
            * plate.sol_em(self.points[i] - self.points[j], iu, iv)?)
    }

    /// `Polm(P_j, iu, iv, Deru_j, Derv_j)`, the monomial row entry of node `j`.
    pub(super) fn monomial(
        &self,
        plate: &PlatePlate,
        j: usize,
        iu: i32,
        iv: i32,
    ) -> Result<f64, Failure> {
        plate.polm(self.points[j], iu, iv, self.deru[j], self.derv[j])
    }
}

/// `(iu, iv)` of the monomials `u^iu v^iv` with `iu + iv < order`, in the native order.
pub(super) fn monomials(order: i32) -> impl Iterator<Item = (i32, i32)> {
    (0..order).flat_map(move |iu| (0..order - iu).map(move |iv| (iu, iv)))
}

/// Copies the lower triangle of the `size` block at `(i0, j0)` onto the upper one.
pub(super) fn symmetrize(mat: &mut Matrix<'_>, (i0, j0): (i32, i32), size: i32) {
    for i in 0..size {
        for j in i + 1..size {
            mat[(i0 + i, j0 + j)] = mat[(i0 + j, j0 + i)];
        }
    }
}

/// `fillXYZmatrix`: the block of the pinpoint and linear XYZ constraints at `(i0, j0)`
/// of the system, the kernel between the nodes, the XYZ combinations of kernel rows
/// and the monomial rows, symmetrized.
pub(super) fn fill_xyz_matrix(
    plate: &PlatePlate,
    nodes: &Nodes<'_>,
    xyz: &[Block<f64>],
    mat: &mut Matrix<'_>,
    (i0, j0, ncc1, ncc2): (i32, i32, i32, i32),
) -> Result<(), Failure> {
    for i in 0..ncc1 as usize {
        for j in 0..i {
            mat[(i0 + i as i32, j0 + j as i32)] = nodes.kernel(plate, i, j)?;
        }
    }
    let (mut k, mut kppc): (i32, i32) = (ncc1, ncc1);
    for (index, block) in xyz.iter().enumerate() {
        xyz_rows_against_nodes(plate, nodes, block, mat, (i0 + k, j0, kppc, ncc1))?;
        let (mut k2, mut kppc2): (i32, i32) = (ncc1, ncc1);
        for other in &xyz[..=index] {
            let tmp = kernel_block(
                plate,
                nodes,
                (kppc, block.row_length()),
                (kppc2, other.row_length()),
            )?;
            for irow in 1..=block.col_length() {
                for irow2 in 1..=other.col_length() {
                    for icol in 1..=block.row_length() {
                        for icol2 in 1..=other.row_length() {
                            mat[(i0 + k + irow - 1, j0 + k2 + irow2 - 1)] += block
                                .coeff(irow, icol)
                                * other.coeff(irow2, icol2)
                                * tmp[(icol, icol2)];
                        }
                    }
                }
            }
            k2 += other.col_length();
            kppc2 += other.row_length();
        }
        k += block.col_length();
        kppc += block.row_length();
    }
    xyz_monomial_rows(plate, nodes, xyz, mat, (i0 + ncc1 + ncc2, j0, ncc1))?;
    symmetrize(
        mat,
        (i0, j0),
        ncc1 + ncc2 + plate.order * (plate.order + 1) / 2,
    );
    Ok(())
}

/// The kernel values of the nodes of a block against the pinpoint nodes `0..ncc1`,
/// combined by the block rows.
fn xyz_rows_against_nodes(
    plate: &PlatePlate,
    nodes: &Nodes<'_>,
    block: &Block<f64>,
    mat: &mut Matrix<'_>,
    (row0, j0, kppc, ncc1): (i32, i32, i32, i32),
) -> Result<(), Failure> {
    for a_j in 0..ncc1 {
        let vmat: Vector<'static> =
            node_kernels(plate, nodes, kppc, block.row_length(), a_j as usize)?;
        for irow in 1..=block.col_length() {
            for icol in 1..=block.row_length() {
                mat[(row0 + irow - 1, j0 + a_j)] += block.coeff(irow, icol) * vmat[icol];
            }
        }
    }
    Ok(())
}

/// `vmat(ippc) = kernel(kppc + ippc - 1, j)` for the `count` nodes of a block.
pub(super) fn node_kernels(
    plate: &PlatePlate,
    nodes: &Nodes<'_>,
    kppc: i32,
    count: i32,
    j: usize,
) -> Result<Vector<'static>, Failure> {
    let mut vmat: Vector<'static> = Vector::with_bounds(1, count)?;
    for ippc in 1..=count {
        vmat[ippc] = nodes.kernel(plate, (kppc + ippc - 1) as usize, j)?;
    }
    Ok(vmat)
}

/// `tmpmat(ippc, ippc2)`: the kernel between the nodes of two blocks.
pub(super) fn kernel_block(
    plate: &PlatePlate,
    nodes: &Nodes<'_>,
    (kppc, count): (i32, i32),
    (kppc2, count2): (i32, i32),
) -> Result<Matrix<'static>, Failure> {
    let mut tmp: Matrix<'static> = Matrix::with_bounds(1, count, 1, count2)
        .map_err(rustycad_tkg2d::geom2d::errors::array_bounds)?;
    for ippc in 1..=count {
        for ippc2 in 1..=count2 {
            let (i, j) = ((kppc + ippc - 1) as usize, (kppc2 + ippc2 - 1) as usize);
            tmp[(ippc, ippc2)] = nodes.kernel(plate, i, j)?;
        }
    }
    Ok(tmp)
}

/// `vmat(ippc) = Polm(node, iu, iv)` for the `count` nodes of a block.
pub(super) fn node_monomials(
    plate: &PlatePlate,
    nodes: &Nodes<'_>,
    (kppc, count): (i32, i32),
    (iu, iv): (i32, i32),
) -> Result<Vector<'static>, Failure> {
    let mut vmat: Vector<'static> = Vector::with_bounds(1, count)?;
    for ippc in 1..=count {
        vmat[ippc] = nodes.monomial(plate, (kppc + ippc - 1) as usize, iu, iv)?;
    }
    Ok(vmat)
}

/// The monomial rows of the pinpoint nodes and of the XYZ blocks.
fn xyz_monomial_rows(
    plate: &PlatePlate,
    nodes: &Nodes<'_>,
    xyz: &[Block<f64>],
    mat: &mut Matrix<'_>,
    (first_row, j0, ncc1): (i32, i32, i32),
) -> Result<(), Failure> {
    for (i, (iu, iv)) in (first_row..).zip(monomials(plate.order)) {
        for a_j in 0..ncc1 {
            mat[(i, j0 + a_j)] = nodes.monomial(plate, a_j as usize, iu, iv)?;
        }
        let (mut k2, mut kppc2): (i32, i32) = (ncc1, ncc1);
        for block in xyz {
            let vmat = node_monomials(plate, nodes, (kppc2, block.row_length()), (iu, iv))?;
            for irow2 in 1..=block.col_length() {
                for icol2 in 1..=block.row_length() {
                    mat[(i, j0 + k2 + irow2 - 1)] += block.coeff(irow2, icol2) * vmat[icol2];
                }
            }
            k2 += block.col_length();
            kppc2 += block.row_length();
        }
    }
    Ok(())
}
