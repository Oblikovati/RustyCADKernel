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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/Plate/Plate_Plate.cxx (SolveTI3) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `SolveTI3`: the general plate system, where the scalar constraints couple the three
//! coordinate systems.
use super::PlatePlate;
use super::solve::{factor, nb_monomials, regularize, solve_refined, zero_matrix};
use super::system::{
    Block, Nodes, fill_xyz_matrix, kernel_block, monomials, node_kernels, node_monomials,
};
use rustycad_tkernel::message::{ProgressRange, ProgressScope};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Xyz;
use rustycad_tkmath::math::{Matrix, Vector};

/// The sizes of the general system: `nCC1` pinpoint rows, `nCC2` XYZ rows, one
/// coordinate block of `n_dimsousmat` unknowns, the node index `nPPC2`
/// of the first scalar constraint node.
#[derive(Clone, Copy)]
struct Layout {
    ncc1: i32,
    ncc2: i32,
    sub: i32,
    n_ppc2: i32,
}

/// Adds `coeff.X * value`, `coeff.Y * value`, `coeff.Z * value` to the three
/// coordinate columns `col`, `sub + col`, `2 * sub + col` of `row`.
fn add_xyz(mat: &mut Matrix<'_>, (row, col, sub): (i32, i32, i32), coeff: Xyz, value: f64) {
    mat[(row, col)] += coeff.x() * value;
    mat[(row, sub + col)] += coeff.y() * value;
    mat[(row, 2 * sub + col)] += coeff.z() * value;
}

impl PlatePlate {
    /// `SolveTI3`: pinpoint, linear XYZ and linear scalar constraints.
    pub(super) fn solve_ti3(
        &mut self,
        iterations: i32,
        progress: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        let xyz: Vec<Block<f64>> = self.xyz_blocks()?;
        let scalar: Vec<Block<Xyz>> = self.scalar_blocks()?;
        let ncc1: i32 = self.my_constraints.length();
        let ncc2: i32 = xyz.iter().map(Block::col_length).sum();
        let ncc3: i32 = scalar.iter().map(Block::col_length).sum();
        let nbm: i32 = nb_monomials(self.order);
        let sub: i32 = ncc1 + ncc2 + nbm;
        let n_dimat: i32 = 3 * sub + ncc3;
        let n_ppc2: i32 = ncc1 + xyz.iter().map(Block::row_length).sum::<i32>();
        let layout = Layout {
            ncc1,
            ncc2,
            sub,
            n_ppc2,
        };
        self.set_nodes(
            xyz.iter()
                .flat_map(|b| b.ppc.iter())
                .chain(scalar.iter().flat_map(|b| b.ppc.iter())),
        );
        let mut mat: Matrix<'static> = zero_matrix(n_dimat)?;
        let nodes = self.nodes();
        for offset in [0, sub, 2 * sub] {
            fill_xyz_matrix(self, &nodes, &xyz, &mut mat, (offset, offset, ncc1, ncc2))?;
        }
        self.fill_scalar_rows(&nodes, (&xyz, &scalar), &mut mat, layout)?;
        for j in 3 * sub..n_dimat {
            for i in 0..j {
                mat[(i, j)] = mat[(j, i)];
            }
        }
        self.ok = true;
        let scope = ProgressScope::with_literal(progress, "Plate_Plate::SolveTI3()", 10.0, false);
        let starts = [ncc1 + ncc2, sub + ncc1 + ncc2, 2 * sub + ncc1 + ncc2];
        let Some(gauss) = factor(&mut mat, &scope, |mat| regularize(mat, &starts, nbm))? else {
            self.ok = false;
            return Ok(());
        };
        self.ok = gauss.is_done();
        if !self.ok {
            return Ok(());
        }
        let mut sec_member: Vector<'static> = Vector::filled(0, n_dimat - 1, 0.0)?;
        self.n_dim = self.n_el + nbm;
        let mut solution: Vec<Xyz> = vec![Xyz::default(); self.n_dim as usize];
        for icoor in 1..=3 {
            self.xyz_right_hand_side(&xyz, &mut sec_member, ((icoor - 1) * sub, icoor))?;
        }
        let mut k: i32 = 3 * sub;
        for block in &scalar {
            for irow in 1..=block.col_length() {
                for icol in 1..=block.row_length() {
                    sec_member[k] += block.coeff(irow, icol) * block.ppc(icol).value();
                }
                k += 1;
            }
        }
        let sol = solve_refined(&gauss, &mat, &sec_member, iterations)?;
        for icoor in 1..=3 {
            let offset: i32 = (icoor - 1) * sub;
            self.store_xyz_solution(&xyz, &mut solution, |i| sol[offset + i], icoor)?;
        }
        store_scalar_solution(&scalar, &mut solution, |i| sol[i], layout);
        self.solution = Some(solution);
        Ok(())
    }

    /// The rows of the scalar constraints: against the pinpoint nodes, the XYZ blocks,
    /// the monomials and the scalar blocks up to themselves.
    fn fill_scalar_rows(
        &self,
        nodes: &Nodes<'_>,
        (xyz, scalar): (&[Block<f64>], &[Block<Xyz>]),
        mat: &mut Matrix<'static>,
        layout: Layout,
    ) -> Result<(), Failure> {
        let (mut k, mut kppc): (i32, i32) = (3 * layout.sub, layout.n_ppc2);
        for (index, block) in scalar.iter().enumerate() {
            self.scalar_against_nodes(nodes, block, mat, (k, kppc, layout))?;
            self.scalar_against_xyz(nodes, block, xyz, mat, (k, kppc, layout))?;
            self.scalar_against_monomials(nodes, block, mat, (k, kppc, layout))?;
            let (mut k2, mut kppc2): (i32, i32) = (3 * layout.sub, layout.n_ppc2);
            for other in &scalar[..=index] {
                let tmp = kernel_block(
                    self,
                    nodes,
                    (kppc, block.row_length()),
                    (kppc2, other.row_length()),
                )?;
                for irow in 1..=block.col_length() {
                    for irow2 in 1..=other.col_length() {
                        for icol in 1..=block.row_length() {
                            for icol2 in 1..=other.row_length() {
                                mat[(k + irow - 1, k2 + irow2 - 1)] += (block.coeff(irow, icol)
                                    * other.coeff(irow2, icol2))
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
        Ok(())
    }

    fn scalar_against_nodes(
        &self,
        nodes: &Nodes<'_>,
        block: &Block<Xyz>,
        mat: &mut Matrix<'static>,
        (k, kppc, layout): (i32, i32, Layout),
    ) -> Result<(), Failure> {
        for j in 0..layout.ncc1 {
            let vmat = node_kernels(self, nodes, kppc, block.row_length(), j as usize)?;
            for irow in 1..=block.col_length() {
                for icol in 1..=block.row_length() {
                    add_xyz(
                        mat,
                        (k + irow - 1, j, layout.sub),
                        block.coeff(irow, icol),
                        vmat[icol],
                    );
                }
            }
        }
        Ok(())
    }

    fn scalar_against_xyz(
        &self,
        nodes: &Nodes<'_>,
        block: &Block<Xyz>,
        xyz: &[Block<f64>],
        mat: &mut Matrix<'static>,
        (k, kppc, layout): (i32, i32, Layout),
    ) -> Result<(), Failure> {
        let (mut k2, mut kppc2): (i32, i32) = (layout.ncc1, layout.ncc1);
        for other in xyz {
            let tmp = kernel_block(
                self,
                nodes,
                (kppc, block.row_length()),
                (kppc2, other.row_length()),
            )?;
            for irow in 1..=block.col_length() {
                for irow2 in 1..=other.col_length() {
                    for icol in 1..=block.row_length() {
                        for icol2 in 1..=other.row_length() {
                            let c: Xyz = block.coeff(irow, icol);
                            let factor: f64 = other.coeff(irow2, icol2);
                            let t: f64 = tmp[(icol, icol2)];
                            let (row, col, sub) = (k + irow - 1, k2 + irow2 - 1, layout.sub);
                            mat[(row, col)] += c.x() * factor * t;
                            mat[(row, sub + col)] += c.y() * factor * t;
                            mat[(row, 2 * sub + col)] += c.z() * factor * t;
                        }
                    }
                }
            }
            k2 += other.col_length();
            kppc2 += other.row_length();
        }
        Ok(())
    }

    fn scalar_against_monomials(
        &self,
        nodes: &Nodes<'_>,
        block: &Block<Xyz>,
        mat: &mut Matrix<'static>,
        (k, kppc, layout): (i32, i32, Layout),
    ) -> Result<(), Failure> {
        for (j, (iu, iv)) in (layout.ncc1 + layout.ncc2..).zip(monomials(self.order)) {
            let vmat = node_monomials(self, nodes, (kppc, block.row_length()), (iu, iv))?;
            for irow in 1..=block.col_length() {
                for icol in 1..=block.row_length() {
                    add_xyz(
                        mat,
                        (k + irow - 1, j, layout.sub),
                        block.coeff(irow, icol),
                        vmat[icol],
                    );
                }
            }
        }
        Ok(())
    }
}

/// The solution of the scalar constraint nodes: `sum_irow Coeff(irow, icol) *
/// sol(ksol + irow - 1)` for each node, from the scalar multipliers after the three
/// coordinate blocks.
fn store_scalar_solution(
    scalar: &[Block<Xyz>],
    solution: &mut [Xyz],
    sol: impl Fn(i32) -> f64,
    layout: Layout,
) {
    let (mut ksol, mut k_solution): (i32, usize) = (3 * layout.sub, layout.n_ppc2 as usize);
    for block in scalar {
        for icol in 1..=block.row_length() {
            let mut vsol = Xyz::new(0.0, 0.0, 0.0);
            for irow in 1..=block.col_length() {
                vsol += block.coeff(irow, icol) * sol(ksol + irow - 1);
            }
            solution[k_solution] = vsol;
            k_solution += 1;
        }
        ksol += block.col_length();
    }
}
