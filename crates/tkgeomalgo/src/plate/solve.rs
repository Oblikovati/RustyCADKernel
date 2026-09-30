// Rust translation of ModelingAlgorithms/TKGeomAlgo/Plate/Plate_Plate.cxx (SolveTI, SolveTI1,
// SolveTI2 and UVBox) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `SolveTI`: the scaling of the parameter box and the systems of the pinpoint-only
//! and pinpoint plus linear XYZ cases. The general case is in `solve_scalar`.
use super::system::{Block, Nodes, fill_xyz_matrix, monomials, symmetrize};
use super::{PlatePinpointConstraint, PlatePlate};
use rustycad_tkernel::message::{ProgressRange, ProgressScope};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkmath::gp::{Xy, Xyz};
use rustycad_tkmath::math::{Gauss, Matrix, Vector};

/// `order * (order + 1) / 2`, the number of monomials of total degree below `order`.
pub(super) fn nb_monomials(order: i32) -> i32 {
    order * (order + 1) / 2
}

/// A `size x size` zero matrix with bounds from 0.
pub(super) fn zero_matrix(size: i32) -> Result<Matrix<'static>, Failure> {
    Matrix::filled(0, size - 1, 0, size - 1, 0.0).map_err(errors::array_bounds)
}

/// The two-step factorization of the plate systems: pivot 1e-12, then, if singular, the
/// diagonal of the polynomial unknowns set by `regularize` and pivot 1e-18. `None` when
/// the user breaks; the returned solver tells whether it is done.
pub(super) fn factor(
    mat: &mut Matrix<'static>,
    scope: &ProgressScope<'_>,
    regularize: impl FnOnce(&mut Matrix<'static>),
) -> Result<Option<Gauss>, Failure> {
    let gauss = Gauss::with_progress(mat, 1.0e-12, &scope.next(7.0))?;
    if scope.user_break() {
        return Ok(None);
    }
    if gauss.is_done() {
        return Ok(Some(gauss));
    }
    regularize(mat);
    let retry = Gauss::with_progress(mat, 1.0e-18, &scope.next(3.0))?;
    if scope.user_break() {
        return Ok(None);
    }
    Ok(Some(retry))
}

/// `algo_gauss.Solve(sec_member, sol)`, then `iterations` refinement steps (the native
/// callers pass 0).
pub(super) fn solve_refined(
    gauss: &Gauss,
    mat: &Matrix<'_>,
    sec_member: &Vector<'_>,
    iterations: i32,
) -> Result<Vector<'static>, Failure> {
    let size: i32 = sec_member.length();
    let mut sol: Vector<'static> = Vector::with_bounds(0, size - 1)?;
    gauss.solve_into(sec_member, &mut sol)?;
    let mut sol1: Vector<'static> = Vector::with_bounds(0, size - 1)?;
    for _ in 1..=iterations {
        let residual = sec_member.subtracted(&mat.multiplied_vector(&sol)?)?;
        gauss.solve_into(&residual, &mut sol1)?;
        sol += &sol1;
    }
    Ok(sol)
}

impl PlatePlate {
    /// Solves the loaded constraints with a plate of order `ord` (`SolveTI`; the native
    /// defaults are 4 and 1): `ord` must be in `2..=9`, there must be constraints and
    /// `anisotropie` in `[1e-6, 1e6]`, else the plate stays not done. The parameter box
    /// is scaled by `anisotropie` in U (above 1) or V (below 1). Standard_OutOfRange
    /// when no pinpoint constraint was loaded (`UVBox` reads the first one).
    /// `plate.solve_ti(4, 1.0, &ProgressRange::default())?`.
    pub fn solve_ti(
        &mut self,
        ord: i32,
        anisotropie: f64,
        the_progress: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        let iteration_number: i32 = 0;
        self.ok = false;
        self.order = ord;
        if ord <= 1 || ord > 9 || self.n_el < 1 || anisotropie < 1.0e-6 || anisotropie > 1.0e6 {
            return Ok(());
        }
        let (xmin, xmax, ymin, ymax) = self.uv_box()?;
        let mut du: f64 = 0.5 * (xmax - xmin);
        if anisotropie > 1.0 {
            du *= anisotropie;
        }
        if du < 1.0e-10 {
            return Ok(());
        }
        scale_factors(&mut self.ddu, du);
        let mut dv: f64 = 0.5 * (ymax - ymin);
        if anisotropie < 1.0 {
            dv /= anisotropie;
        }
        if dv < 1.0e-10 {
            return Ok(());
        }
        scale_factors(&mut self.ddv, dv);
        if !self.my_lscalar_constraints.is_empty() {
            return self.solve_ti3(iteration_number, the_progress);
        }
        if self.my_lxyz_constraints.is_empty() {
            return self.solve_ti1(iteration_number, the_progress);
        }
        self.solve_ti2(iteration_number, the_progress)
    }

    /// The bounding box `(UMin, UMax, VMin, VMax)` of the constraint points, widened to
    /// at least 1e-3 in each direction (`UVBox`); Standard_OutOfRange without a pinpoint
    /// constraint. `plate.uv_box()?`.
    pub fn uv_box(&self) -> Result<(f64, f64, f64, f64), Failure> {
        let first: Xy = self
            .my_constraints
            .value(1)
            .map_err(|_| {
                errors::out_of_range(
                    "Plate_Plate::UVBox: no pinpoint constraint; expected at least one",
                )
            })?
            .pnt2d();
        let mut bounds = [first.x(), first.x(), first.y(), first.y()];
        let mut widen = |p: Xy| {
            let (x, y) = (p.x(), p.y());
            if x < bounds[0] {
                bounds[0] = x;
            }
            if x > bounds[1] {
                bounds[1] = x;
            }
            if y < bounds[2] {
                bounds[2] = y;
            }
            if y > bounds[3] {
                bounds[3] = y;
            }
        };
        self.my_constraints
            .iter()
            .skip(1)
            .for_each(|c| widen(c.pnt2d()));
        for block in self.xyz_blocks()? {
            block.ppc.iter().for_each(|c| widen(c.pnt2d()));
        }
        for block in self.scalar_blocks()? {
            block.ppc.iter().for_each(|c| widen(c.pnt2d()));
        }
        let [umin, umax] = min_width(bounds[0], bounds[1]);
        let [vmin, vmax] = min_width(bounds[2], bounds[3]);
        Ok((umin, umax, vmin, vmax))
    }

    /// The linear XYZ constraints, read once.
    pub(super) fn xyz_blocks(&self) -> Result<Vec<Block<f64>>, Failure> {
        self.my_lxyz_constraints
            .iter()
            .map(|c| c.snapshot().map(|(ppc, coef)| Block { ppc, coef }))
            .collect()
    }

    /// The linear scalar constraints, read once.
    pub(super) fn scalar_blocks(&self) -> Result<Vec<Block<Xyz>>, Failure> {
        self.my_lscalar_constraints
            .iter()
            .map(|c| c.snapshot().map(|(ppc, coef)| Block { ppc, coef }))
            .collect()
    }

    /// Stores the nodes of the pinpoint constraints then of the blocks' constraints.
    pub(super) fn set_nodes<'a>(
        &mut self,
        extra: impl Iterator<Item = &'a PlatePinpointConstraint>,
    ) {
        let all: Vec<PlatePinpointConstraint> = self
            .my_constraints
            .iter()
            .copied()
            .chain(extra.copied())
            .collect();
        self.points = Some(all.iter().map(|c| c.pnt2d()).collect());
        self.deru = Some(all.iter().map(|c| c.idu()).collect());
        self.derv = Some(all.iter().map(|c| c.idv()).collect());
    }

    /// The stored nodes, for the matrix builders.
    pub(super) fn nodes(&self) -> Nodes<'_> {
        Nodes {
            points: self.points.as_deref().unwrap_or_default(),
            deru: self.deru.as_deref().unwrap_or_default(),
            derv: self.derv.as_deref().unwrap_or_default(),
        }
    }

    /// `SolveTI1`: pinpoint constraints only.
    fn solve_ti1(&mut self, iterations: i32, progress: &ProgressRange<'_>) -> Result<(), Failure> {
        let nbm: i32 = nb_monomials(self.order);
        self.n_dim = self.n_el + nbm;
        let mut mat: Matrix<'static> = zero_matrix(self.n_dim)?;
        self.set_nodes(std::iter::empty());
        self.fill_pinpoint_system(&mut mat)?;
        self.ok = true;
        let scope = ProgressScope::with_literal(progress, "Plate_Plate::SolveTI1()", 10.0, false);
        let n_el: i32 = self.n_el;
        let Some(gauss) = factor(&mut mat, &scope, |mat| regularize(mat, &[n_el], nbm))? else {
            self.ok = false;
            return Ok(());
        };
        self.ok = gauss.is_done();
        if !self.ok {
            return Ok(());
        }
        let mut sec_member: Vector<'static> = Vector::filled(0, self.n_dim - 1, 0.0)?;
        let mut solution: Vec<Xyz> = vec![Xyz::default(); self.n_dim as usize];
        for icoor in 1..=3 {
            for (i, c) in (0..).zip(self.my_constraints.iter()) {
                sec_member[i] = c.value().coord(icoor)?;
            }
            let sol = solve_refined(&gauss, &mat, &sec_member, iterations)?;
            for (i, value) in (0..).zip(solution.iter_mut()) {
                value.set_coord(icoor, sol[i])?;
            }
        }
        self.solution = Some(solution);
        Ok(())
    }

    /// The kernel and monomial entries of the pinpoint-only system, symmetrized.
    fn fill_pinpoint_system(&self, mat: &mut Matrix<'static>) -> Result<(), Failure> {
        let nodes = self.nodes();
        for i in 0..self.n_el as usize {
            for j in 0..i {
                mat[(i as i32, j as i32)] = nodes.kernel(self, i, j)?;
            }
        }
        for (i, (iu, iv)) in (self.n_el..).zip(monomials(self.order)) {
            for j in 0..self.n_el {
                mat[(i, j)] = nodes.monomial(self, j as usize, iu, iv)?;
            }
        }
        symmetrize(mat, (0, 0), self.n_dim);
        Ok(())
    }

    /// `SolveTI2`: pinpoint and linear XYZ constraints.
    fn solve_ti2(&mut self, iterations: i32, progress: &ProgressRange<'_>) -> Result<(), Failure> {
        let blocks: Vec<Block<f64>> = self.xyz_blocks()?;
        let ncc1: i32 = self.my_constraints.length();
        let ncc2: i32 = blocks.iter().map(Block::col_length).sum();
        let nbm: i32 = nb_monomials(self.order);
        let n_dimat: i32 = ncc1 + ncc2 + nbm;
        self.set_nodes(blocks.iter().flat_map(|b| b.ppc.iter()));
        let mut mat: Matrix<'static> = zero_matrix(n_dimat)?;
        fill_xyz_matrix(self, &self.nodes(), &blocks, &mut mat, (0, 0, ncc1, ncc2))?;
        self.ok = true;
        let scope = ProgressScope::with_literal(progress, "Plate_Plate::SolveTI2()", 10.0, false);
        let Some(gauss) = factor(&mut mat, &scope, |mat| regularize(mat, &[ncc1 + ncc2], nbm))?
        else {
            self.ok = false;
            return Ok(());
        };
        self.ok = gauss.is_done();
        if !self.ok {
            return Ok(());
        }
        self.n_dim = self.n_el + nbm;
        let mut solution: Vec<Xyz> = vec![Xyz::default(); self.n_dim as usize];
        // The right-hand side is allocated once: the linear XYZ rows accumulate the
        // X, then Y, then Z terms with `+=`, as natively.
        let mut sec_member: Vector<'static> = Vector::filled(0, n_dimat - 1, 0.0)?;
        for icoor in 1..=3 {
            self.xyz_right_hand_side(&blocks, &mut sec_member, (0, icoor))?;
            let sol = solve_refined(&gauss, &mat, &sec_member, iterations)?;
            self.store_xyz_solution(&blocks, &mut solution, |i| sol[i], icoor)?;
        }
        self.solution = Some(solution);
        Ok(())
    }

    /// The pinpoint values and the XYZ combinations of coordinate `icoor`, from `offset`.
    pub(super) fn xyz_right_hand_side(
        &self,
        blocks: &[Block<f64>],
        sec_member: &mut Vector<'_>,
        (offset, icoor): (i32, i32),
    ) -> Result<(), Failure> {
        for (i, c) in (0..).zip(self.my_constraints.iter()) {
            sec_member[offset + i] = c.value().coord(icoor)?;
        }
        let mut k: i32 = self.my_constraints.length();
        for block in blocks {
            for irow in 1..=block.col_length() {
                for icol in 1..=block.row_length() {
                    sec_member[offset + k] +=
                        block.coeff(irow, icol) * block.ppc(icol).value().coord(icoor)?;
                }
                k += 1;
            }
        }
        Ok(())
    }

    /// Coordinate `icoor` of the solution from the multipliers `sol(i)`: the pinpoint
    /// multipliers, the XYZ combinations transposed back onto their constraints, then
    /// the polynomial coefficients.
    pub(super) fn store_xyz_solution(
        &self,
        blocks: &[Block<f64>],
        solution: &mut [Xyz],
        sol: impl Fn(i32) -> f64,
        icoor: i32,
    ) -> Result<(), Failure> {
        let ncc1: i32 = self.my_constraints.length();
        for i in 0..ncc1 {
            solution[i as usize].set_coord(icoor, sol(i))?;
        }
        let (mut k_solution, mut ksol): (usize, i32) = (ncc1 as usize, ncc1);
        for block in blocks {
            for icol in 1..=block.row_length() {
                let mut vsol: f64 = 0.0;
                for irow in 1..=block.col_length() {
                    vsol += block.coeff(irow, icol) * sol(ksol + irow - 1);
                }
                solution[k_solution].set_coord(icoor, vsol)?;
                k_solution += 1;
            }
            ksol += block.col_length();
        }
        for i in 0..nb_monomials(self.order) {
            solution[(self.n_el + i) as usize].set_coord(icoor, sol(ksol + i))?;
        }
        Ok(())
    }
}

/// `dd[0] = 1`, `dd[i] = dd[i - 1] / d`: the scaled powers of the half box size.
fn scale_factors(dd: &mut [f64; 10], d: f64) {
    dd[0] = 1.0;
    for i in 1..=9 {
        dd[i] = dd[i - 1] / d;
    }
}

/// A range widened symmetrically to at least `Bmin = 1e-3`.
fn min_width(min: f64, max: f64) -> [f64; 2] {
    const BMIN: f64 = 1.0e-3;
    // Written as the native `if (UMax - UMin < Bmin)`, so that a NaN width is kept.
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    if !(max - min < BMIN) {
        return [min, max];
    }
    let mid: f64 = 0.5 * (min + max);
    [mid - 0.5 * BMIN, mid + 0.5 * BMIN]
}

/// Sets `mat(i, i) = 1e-8` for the `nbm` polynomial unknowns after each start.
pub(super) fn regularize(mat: &mut Matrix<'_>, starts: &[i32], nbm: i32) {
    for &start in starts {
        for i in start..start + nbm {
            mat[(i, i)] = 1.0e-8;
        }
    }
}
