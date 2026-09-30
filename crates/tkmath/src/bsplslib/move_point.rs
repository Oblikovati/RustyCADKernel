// Rust translation of FoundationClasses/TKMath/BSplSLib/BSplSLib.cxx (MovePoint) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Local modification of surface poles so that a point reaches a new position.
use crate::bsplclib::eval_bspline_basis;
use crate::gp::{Point3, Vector3};
use crate::math::Matrix;
use crate::plib::access::{grid_cell, grid_value};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;

/// Index ranges allowed to move in U and V.
#[derive(Clone, Copy, Debug)]
pub struct MoveRange {
    /// First U pole index allowed to move.
    pub u_index1: i32,
    /// Last U pole index allowed to move.
    pub u_index2: i32,
    /// First V pole index allowed to move.
    pub v_index1: i32,
    /// Last V pole index allowed to move.
    pub v_index2: i32,
}

// The basis in one direction and the predominant pole(s) kk1..kk2.
struct Direction {
    basis: Matrix<'static>,
    first_non_zero: i32,
    first: i32,
    last: i32,
    kk1: i32,
    kk2: i32,
}

impl Direction {
    fn value(&self, column: i32) -> Result<f64, Failure> {
        self.basis.value(1, column).copied()
    }

    fn predominant(&mut self, degree: i32, index1: i32, index2: i32) -> Result<(), Failure> {
        self.first = self.first_non_zero.max(index1);
        self.last = (self.first_non_zero + degree).min(index2);
        let mut max_value: f64 = 0.0;
        self.kk1 = 0;
        for i in self.first - self.first_non_zero + 1..=self.last - self.first_non_zero + 1 {
            let value: f64 = self.value(i)?;
            if value > max_value {
                self.kk1 = i + self.first_non_zero - 1;
                max_value = value;
            }
        }
        // A symmetric maximum spreads the displacement over two poles.
        self.kk2 = self.kk1;
        if self.kk1 < self.last
            && (self.value(self.kk1 - self.first_non_zero + 2)? - max_value).abs() < 1.0e-10
        {
            self.kk2 = self.kk1 + 1;
        }
        Ok(())
    }

    fn dval(&self, index: i32) -> f64 {
        if index < self.kk1 {
            return f64::from(self.kk1 - index);
        }
        if index > self.kk2 {
            return f64::from(index - self.kk2);
        }
        0.0
    }

    fn inside(&self, index: i32) -> bool {
        index >= self.first && index <= self.last
    }
}

fn basis(
    degree: i32,
    flat_knots: &Array1<'_, f64>,
    parameter: f64,
) -> Result<(Direction, i32), Failure> {
    let mut matrix: Matrix<'static> = Matrix::with_bounds(1, 1, 1, degree + 1)
        .map_err(|error| Failure::out_of_range(&format!("BSplSLib::MovePoint: {error}")))?;
    let mut first: i32 = 0;
    let code: i32 = eval_bspline_basis(
        0,
        degree + 1,
        flat_knots,
        parameter,
        &mut first,
        &mut matrix,
        false,
    )?;
    let direction = Direction {
        basis: matrix,
        first_non_zero: first,
        first: 0,
        last: 0,
        kk1: 0,
        kk2: 0,
    };
    Ok((direction, code))
}

/// Moves the surface point at (U, V) by `displ` changing only poles in `range`
/// (`BSplSLib::MovePoint`); returns `(UFirst, ULast, VFirst, VLast)`, all 0 with
/// `new_poles` untouched when a basis cannot be evaluated;
/// `bsplslib::move_point(0.5, 0.5, d, range, 3, 3, false, &p, &w, &uf, &vf, &mut np)?`.
pub fn move_point(
    u: f64,
    v: f64,
    displ: Vector3,
    range: MoveRange,
    u_degree: i32,
    v_degree: i32,
    rational: bool,
    poles: &Array2<'_, Point3>,
    weights: &Array2<'_, f64>,
    u_flat_knots: &Array1<'_, f64>,
    v_flat_knots: &Array1<'_, f64>,
    new_poles: &mut Array2<'_, Point3>,
) -> Result<(i32, i32, i32, i32), Failure> {
    let (mut du, code_u) = basis(u_degree, u_flat_knots, u)?;
    let (mut dv, code_v) = basis(v_degree, v_flat_knots, v)?;
    if code_u != 0 || code_v != 0 {
        return Ok((0, 0, 0, 0));
    }
    du.predominant(u_degree, range.u_index1, range.u_index2)?;
    dv.predominant(v_degree, range.v_index1, range.v_index2)?;
    let coef: f64 = coefficient(&du, &dv, [u_degree, v_degree], rational, weights)?;
    for i in poles.lower_row()..=poles.upper_row() {
        let dval_u: f64 = du.dval(i);
        for j in poles.lower_col()..=poles.upper_col() {
            let pole: Point3 = grid_value(poles, i, j)?;
            let moved: Point3 = if du.inside(i) && dv.inside(j) {
                let scale: f64 = coef / (dval_u + dv.dval(j) + 1.);
                pole.translated(Vector3::new(
                    displ.x() * scale,
                    displ.y() * scale,
                    displ.z() * scale,
                ))
            } else {
                pole
            };
            *grid_cell(new_poles, i, j)? = moved;
        }
    }
    Ok((du.first, du.last, dv.first, dv.last))
}

fn coefficient(
    du: &Direction,
    dv: &Direction,
    [u_degree, v_degree]: [i32; 2],
    rational: bool,
    weights: &Array2<'_, f64>,
) -> Result<f64, Failure> {
    let (mut d1, mut d2): (f64, f64) = (0.0, 0.0);
    for i in 1..=u_degree + 1 {
        let ii: i32 = i + du.first_non_zero - 1;
        let dval_u: f64 = du.dval(ii);
        for j in 1..=v_degree + 1 {
            let jj: i32 = j + dv.first_non_zero - 1;
            let h_n: f64 = if rational {
                let term: f64 = grid_value(weights, ii, jj)? * du.value(i)? * dv.value(j)?;
                d2 += term;
                term
            } else {
                du.value(i)? * dv.value(j)?
            };
            if du.inside(ii) && dv.inside(jj) {
                d1 += 1. / (dval_u + dv.dval(jj) + 1.) * h_n;
            }
        }
    }
    if rational {
        return Ok(d2 / d1);
    }
    Ok(1. / d1)
}
