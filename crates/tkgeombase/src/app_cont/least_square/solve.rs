// Rust translation of ModelingData/TKGeomBase/AppCont/AppCont_LeastSquare.cxx
// (the resolution) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The resolution part of the `AppCont_LeastSquare` constructor: the right-hand
//! side, the direct solution without constraint, and the elimination of the pass
//! and tangency poles before solving for the free ones.
use super::{Evaluation, LeastSquare, matrix};
use crate::app_cont::cont_matrices::{ibp_matrix, ibt_matrix, inv_m_matrix, m_matrix};
use crate::app_cont::function::Function;
use crate::app_par_curves::Constraint;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::{Matrix, Vector};

// Whether the tangent pole follows the first pole or precedes the last one.
#[derive(Clone, Copy)]
enum TangentSide {
    Add,
    Subtract,
}

// The poles solved for: rows `bdeb..=bfin` of the pole matrix.
struct FreeRows {
    bdeb: i32,
    bfin: i32,
}

impl LeastSquare {
    // `B(c, .) += sum_i TheWeights(i) VB(c, i) myPoints(i, .)`, accumulated per pole
    // in a temporary row as natively.
    pub(super) fn right_hand_side(
        &self,
        weights: &Vector<'_>,
        mut b: Matrix<'static>,
    ) -> Matrix<'static> {
        let nbcol: i32 = self.nbcol();
        let mut tmppoints: Vec<f64> = vec![0.0; nbcol as usize];
        for c in 1..=self.my_degre + 1 {
            tmppoints.fill(0.0);
            for i in 1..=self.my_nbdiscret {
                let coeff: f64 = weights[i] * self.my_vb[(c, i)];
                for (j, sum) in (1..).zip(tmppoints.iter_mut()) {
                    *sum += self.my_points[(i, j)] * coeff;
                }
            }
            for (k, sum) in (1..).zip(&tmppoints) {
                b[(c, k)] += *sum;
            }
        }
        b
    }

    // No constraint: `myPoles = InvM B`, with the stored inverse mass matrix.
    pub(super) fn solve_free(&mut self, b: &Matrix<'_>) -> Result<(), Failure> {
        let classe: i32 = self.my_degre + 1;
        let mut inv_m: Matrix<'static> = matrix([1, classe], [1, classe], 0.0)?;
        inv_m_matrix(classe, &mut inv_m)?;
        self.add_product(
            &inv_m,
            b,
            FreeRows {
                bdeb: 1,
                bfin: classe,
            },
        );
        Ok(())
    }

    // `myPoles(i, .) += sum_j ib(i, j) rhs(j, .)` for `i, j` in the free rows, the
    // loop order of both native solutions.
    fn add_product(&mut self, ib: &Matrix<'_>, rhs: &Matrix<'_>, rows: FreeRows) {
        for i in rows.bdeb..=rows.bfin {
            for j in rows.bdeb..=rows.bfin {
                let ibij: f64 = ib[(i, j)];
                for k in 1..=self.nbcol() {
                    self.my_poles[(i, k)] += ibij * rhs[(j, k)];
                }
            }
        }
    }

    // Fixes the constrained end poles, removes them from the right-hand side, then
    // solves for the free poles.
    pub(super) fn solve_constrained(
        &mut self,
        ssp: &dyn Function,
        u: [f64; 2],
        [first_c, last_c]: [Constraint; 2],
        mut b: Matrix<'static>,
        arrays: &mut Evaluation,
    ) -> Result<(), Failure> {
        let classe: i32 = self.my_degre + 1;
        let mut m: Matrix<'static> = matrix([1, classe], [1, classe], 0.0)?;
        m_matrix(classe, &mut m)?;
        let is_fixed =
            |c: Constraint| matches!(c, Constraint::PassPoint | Constraint::TangencyPoint);
        if is_fixed(first_c) {
            self.fix_end_pole(ssp, 1, u[0], u, arrays)?;
            self.fix_end_pole_period(1, 1, 3 * self.my_nb_p + 2 * self.my_nb_p2d)?;
        }
        if is_fixed(last_c) {
            self.fix_end_pole(ssp, classe, u[1], u, arrays)?;
            // Natively only the first two coordinates of the last pole are brought
            // back into the period of the last sample (a preserved quirk).
            self.fix_end_pole_period(classe, self.my_nbdiscret, 2)?;
        }
        let rows: FreeRows =
            self.eliminate_constraints(ssp, u, [first_c, last_c], &m, &mut b, arrays);
        if rows.bdeb <= rows.bfin {
            self.solve_free_rows(&m, &b, rows)?;
        }
        Ok(())
    }

    // Subtracts the fixed poles from the right-hand side, in the native order: the
    // first then the last pass point, then the first then the last tangency.
    fn eliminate_constraints(
        &mut self,
        ssp: &dyn Function,
        u: [f64; 2],
        [first_c, last_c]: [Constraint; 2],
        m: &Matrix<'_>,
        b: &mut Matrix<'static>,
        arrays: &mut Evaluation,
    ) -> FreeRows {
        let classe: i32 = self.my_degre + 1;
        let mut rows = FreeRows {
            bdeb: 1,
            bfin: classe,
        };
        if first_c == Constraint::PassPoint {
            rows.bdeb = 2;
            self.subtract_poles(m, b, &[1]);
        }
        if last_c == Constraint::PassPoint {
            rows.bfin = self.my_degre;
            self.subtract_poles(m, b, &[classe]);
        }
        if first_c == Constraint::TangencyPoint {
            rows.bdeb = 3;
            ssp.d1(u[0], &mut arrays.tab_v2d, &mut arrays.tab_v);
            let coeff: f64 = (u[1] - u[0]) / f64::from(self.my_degre);
            self.set_tangent_pole([2, 1], coeff, TangentSide::Add, arrays);
            self.subtract_poles(m, b, &[1, 2]);
        }
        if last_c == Constraint::TangencyPoint {
            rows.bfin = classe - 2;
            ssp.d1(u[1], &mut arrays.tab_v2d, &mut arrays.tab_v);
            let (cl1, coeff): (i32, f64) =
                (self.my_degre, (u[1] - u[0]) / f64::from(self.my_degre));
            self.set_tangent_pole([cl1, classe], coeff, TangentSide::Subtract, arrays);
            self.subtract_poles(m, b, &[classe, cl1]);
        }
        rows
    }

    // `B(i, .) -= sum myPoles(p, .) M(i, p)` over the fixed pole rows `poles`, summed
    // left to right before the subtraction as natively.
    fn subtract_poles(&self, m: &Matrix<'_>, b: &mut Matrix<'static>, poles: &[i32]) {
        for i in 1..=self.my_degre + 1 {
            for k in 1..=self.nbcol() {
                let fixed: f64 = match poles {
                    [p] => self.my_poles[(*p, k)] * m[(i, *p)],
                    [p, q] => {
                        self.my_poles[(*p, k)] * m[(i, *p)] + self.my_poles[(*q, k)] * m[(i, *q)]
                    }
                    _ => 0.0,
                };
                b[(i, k)] -= fixed;
            }
        }
    }

    // Pole row `row` = pole row `from` + or - the derivative times `coeff` (the native
    // `Coeff = (U1 - U0) / Deg`): added after the first pole, subtracted before the last.
    fn set_tangent_pole(
        &mut self,
        [row, from]: [i32; 2],
        coeff: f64,
        side: TangentSide,
        arrays: &Evaluation,
    ) {
        let mut i2: i32 = 1;
        for k in 1..=self.my_nb_p {
            let v = arrays.tab_v[k];
            for (col, derivative) in (i2..).zip([v.x(), v.y(), v.z()]) {
                self.shift_pole([row, from, col], derivative * coeff, side);
            }
            i2 += 3;
        }
        for k in 1..=self.my_nb_p2d {
            let v = arrays.tab_v2d[k];
            for (col, derivative) in (i2..).zip([v.x(), v.y()]) {
                self.shift_pole([row, from, col], derivative * coeff, side);
            }
            i2 += 2;
        }
    }

    // `myPoles(row, col) = myPoles(from, col) +/- step`.
    fn shift_pole(&mut self, [row, from, col]: [i32; 3], step: f64, side: TangentSide) {
        let base: f64 = self.my_poles[(from, col)];
        self.my_poles[(row, col)] = match side {
            TangentSide::Add => base + step,
            TangentSide::Subtract => base - step,
        };
    }

    // `B2 = M B` on the free rows, the inverse normal matrix of the free poles, then
    // `myPoles += IBP B2`; the fit is done from here.
    fn solve_free_rows(
        &mut self,
        m: &Matrix<'_>,
        b: &Matrix<'_>,
        rows: FreeRows,
    ) -> Result<(), Failure> {
        let classe: i32 = self.my_degre + 1;
        let mut b2: Matrix<'static> = matrix([rows.bdeb, rows.bfin], [1, b.upper_col()], 0.0)?;
        for i in rows.bdeb..=rows.bfin {
            for j in 1..=classe {
                let coeff: f64 = m[(i, j)];
                for k in 1..=self.nbcol() {
                    b2[(i, k)] += b[(j, k)] * coeff;
                }
            }
        }
        let ibp: Matrix<'static> = free_normal_inverse(m, &rows, classe)?;
        self.my_done = true;
        self.add_product(&ibp, &b2, rows);
        Ok(())
    }
}

// The inverse of `MP^T MP` for the free columns of M: stored for two pass or two
// tangency points (classes <= 26), computed otherwise.
fn free_normal_inverse(
    m: &Matrix<'_>,
    rows: &FreeRows,
    classe: i32,
) -> Result<Matrix<'static>, Failure> {
    let (bdeb, bfin) = (rows.bdeb, rows.bfin);
    let mut ibp: Matrix<'static> = matrix([bdeb, bfin], [bdeb, bfin], 0.0)?;
    if bdeb == 2 && bfin == classe - 1 && classe <= 26 {
        ibp_matrix(classe, &mut ibp)?;
    } else if bdeb == 3 && bfin == classe - 2 && classe <= 26 {
        ibt_matrix(classe, &mut ibp)?;
    } else {
        let mut mp: Matrix<'static> = matrix([1, classe], [bdeb, bfin], 0.0)?;
        for i in 1..=classe {
            for j in bdeb..=bfin {
                mp[(i, j)] = m[(i, j)];
            }
        }
        let ibp1: Matrix<'static> = mp.transposed().multiplied_matrix(&mp)?;
        ibp.initialized(&ibp1.inverse()?)?;
    }
    Ok(ibp)
}
