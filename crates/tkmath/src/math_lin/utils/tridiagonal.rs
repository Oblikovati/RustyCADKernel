//! `MathLin::Utils` QL helpers of MathLin_EigenSearch.hxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871. Indices are zero-based like the native
//! `math_Vector::operator[]`/`math_Matrix::At`; an index the native code would read out of
//! bounds (undefined behaviour there) panics here with the index and the length.
use crate::math::{Matrix, Vector};

/// Relative threshold that deflates a negligible tridiagonal coupling
/// (`MathLin::Utils::THE_TRIDIAGONAL_DEFLATION_TOLERANCE`);
/// `assert_eq!(TRIDIAGONAL_DEFLATION_TOLERANCE, f64::EPSILON)`.
pub const TRIDIAGONAL_DEFLATION_TOLERANCE: f64 = f64::EPSILON;

/// Multiplier of the scale-normalized eigenpair residual threshold
/// (`MathLin::Utils::THE_TRIDIAGONAL_RESIDUAL_TOLERANCE_FACTOR`);
/// `assert_eq!(TRIDIAGONAL_RESIDUAL_TOLERANCE_FACTOR, 256.0)`.
pub const TRIDIAGONAL_RESIDUAL_TOLERANCE_FACTOR: f64 = 256.0;

/// `std::max(first, second)`: keeps `first` unless `first < second`, so a NaN second
/// argument is dropped and a NaN first argument kept, unlike `f64::max`.
fn native_max(first: f64, second: f64) -> f64 {
    if first < second { second } else { first }
}

fn entry(vector: &Vector<'_>, index: usize) -> f64 {
    match vector.at(index) {
        Ok(value) => *value,
        Err(_) => panic!(
            "index {index} outside a vector of length {}",
            vector.length()
        ),
    }
}

fn entry_mut<'v>(vector: &'v mut Vector<'_>, index: usize) -> &'v mut f64 {
    let length: i32 = vector.length();
    match vector.change_at(index) {
        Ok(value) => value,
        Err(_) => panic!("index {index} outside a vector of length {length}"),
    }
}

fn cell(matrix: &Matrix<'_>, row: usize, col: usize) -> f64 {
    match matrix.at(row, col) {
        Ok(value) => *value,
        Err(_) => panic!(
            "cell ({row}, {col}) outside a {}x{} matrix",
            matrix.row_number(),
            matrix.col_number()
        ),
    }
}

fn cell_mut<'m>(matrix: &'m mut Matrix<'_>, row: usize, col: usize) -> &'m mut f64 {
    let shape: (i32, i32) = (matrix.row_number(), matrix.col_number());
    match matrix.change_at(row, col) {
        Ok(value) => value,
        Err(_) => panic!(
            "cell ({row}, {col}) outside a {}x{} matrix",
            shape.0, shape.1
        ),
    }
}

/// Last row of the unreduced block starting at `start`: the first coupling below the
/// deflation threshold, or `n - 1`; `find_submatrix_end(&diag, &sub, 0, 3)`.
pub fn find_submatrix_end(
    diag: &Vector<'_>,
    subdiag: &Vector<'_>,
    start: usize,
    n: usize,
) -> usize {
    let mut end: usize = start;
    while end + 1 < n {
        let scale: f64 = native_max(entry(diag, end).abs(), entry(diag, end + 1).abs());
        if entry(subdiag, end).abs() <= TRIDIAGONAL_DEFLATION_TOLERANCE * scale {
            break;
        }
        end += 1;
    }
    end
}

/// Wilkinson shift of the block `[start, end]`, with the native sign branch on the
/// half-gap ratio; `compute_wilkinson_shift(&diag, &sub, 0, 2)`.
pub fn compute_wilkinson_shift(
    diag: &Vector<'_>,
    subdiag: &Vector<'_>,
    start: usize,
    end: usize,
) -> f64 {
    let coupling: f64 = entry(subdiag, start);
    let ratio: f64 = (entry(diag, start + 1) - entry(diag, start)) / (2.0 * coupling);
    let radius: f64 = 1.0_f64.hypot(ratio);
    // Native keeps both branches spelled out; `ratio - radius` avoids cancellation when negative.
    let denominator: f64 = if ratio < 0.0 {
        ratio - radius
    } else {
        ratio + radius
    };
    entry(diag, end) - entry(diag, start) + coupling / denominator
}

/// Applies the Givens rotation of row `row` to eigenvector columns `row` and `row + 1`.
fn rotate_columns(vectors: &mut Matrix<'_>, row: usize, sine: f64, cosine: f64, n: usize) {
    for i in 0..n {
        let old: f64 = cell(vectors, i, row + 1);
        let current: f64 = cell(vectors, i, row);
        *cell_mut(vectors, i, row + 1) = sine * current + cosine * old;
        *cell_mut(vectors, i, row) = cosine * current - sine * old;
    }
}

/// Rotation state carried up the block by one implicit QL sweep.
struct QlSweep {
    sine: f64,
    cosine: f64,
    previous: f64,
    shift: f64,
}

impl QlSweep {
    /// One rotation at `row`; returns false when the native early exit (zero radius) fires.
    fn rotate(
        &mut self,
        diag: &mut Vector<'_>,
        subdiag: &mut Vector<'_>,
        row: usize,
        end: usize,
    ) -> bool {
        let temp: f64 = self.sine * entry(subdiag, row);
        let subtemp: f64 = self.cosine * entry(subdiag, row);
        let radius: f64 = temp.hypot(self.shift);
        *entry_mut(subdiag, row + 1) = radius;
        if radius == 0.0 {
            *entry_mut(diag, row + 1) -= self.previous;
            *entry_mut(subdiag, end) = 0.0;
            return false;
        }
        self.sine = temp / radius;
        self.cosine = self.shift / radius;
        self.shift = entry(diag, row + 1) - self.previous;
        let radius_temp: f64 =
            (entry(diag, row) - self.shift) * self.sine + 2.0 * self.cosine * subtemp;
        self.previous = self.sine * radius_temp;
        *entry_mut(diag, row + 1) = self.shift + self.previous;
        self.shift = self.cosine * radius_temp - subtemp;
        true
    }
}

/// One implicit-shift QL sweep over rows `end-1 ..= start`, rotating the `n` eigenvector
/// rows as it goes. Always returns true, as the native helper does;
/// `perform_ql_step(&mut diag, &mut sub, &mut vectors, 0, 2, shift, 3)`.
pub fn perform_ql_step(
    diag: &mut Vector<'_>,
    subdiag: &mut Vector<'_>,
    vectors: &mut Matrix<'_>,
    start: usize,
    end: usize,
    shift: f64,
    n: usize,
) -> bool {
    let mut sweep: QlSweep = QlSweep {
        sine: 1.0,
        cosine: 1.0,
        previous: 0.0,
        shift,
    };
    for row in (start..end).rev() {
        if !sweep.rotate(diag, subdiag, row, end) {
            return true;
        }
        rotate_columns(vectors, row, sweep.sine, sweep.cosine, n);
    }
    *entry_mut(diag, start) -= sweep.previous;
    *entry_mut(subdiag, start) = sweep.shift;
    *entry_mut(subdiag, end) = 0.0;
    true
}

/// Largest magnitude among the diagonal and the `n - 1` couplings of the original matrix.
fn tridiagonal_scale(diag: &Vector<'_>, subdiag: &Vector<'_>, n: usize) -> f64 {
    let mut scale: f64 = 0.0;
    for i in 0..n {
        scale = native_max(scale, entry(diag, i).abs());
        if i + 1 < n {
            scale = native_max(scale, entry(subdiag, i).abs());
        }
    }
    scale
}

/// Scale-normalized residual `(T v_j - lambda_j v_j)_i / scale` in native summation order.
fn scaled_residual(
    original: [&Vector<'_>; 2],
    vectors: &Matrix<'_>,
    i: usize,
    j: usize,
    scale: f64,
    lambda: f64,
) -> f64 {
    let [diag, subdiag] = original;
    let n: usize = diag.length() as usize;
    let mut residual: f64 =
        entry(diag, i) / scale * cell(vectors, i, j) - lambda * cell(vectors, i, j);
    if i > 0 {
        residual += entry(subdiag, i - 1) / scale * cell(vectors, i - 1, j);
    }
    if i + 1 < n {
        residual += entry(subdiag, i) / scale * cell(vectors, i + 1, j);
    }
    residual
}

/// Checks `T v = lambda v` column by column on the scale-normalized matrix, so no product can
/// overflow; a zero matrix is accepted outright;
/// `validate_eigenpairs(&diag, &sub, &values, &vectors)`.
pub fn validate_eigenpairs(
    original_diag: &Vector<'_>,
    original_subdiag: &Vector<'_>,
    eigen_values: &Vector<'_>,
    eigen_vectors: &Matrix<'_>,
) -> bool {
    let n: usize = original_diag.length() as usize;
    let scale: f64 = tridiagonal_scale(original_diag, original_subdiag, n);
    if scale == 0.0 {
        return true;
    }
    let dimension: f64 = n.max(1) as f64;
    for j in 0..n {
        let lambda: f64 = entry(eigen_values, j) / scale;
        let tolerance: f64 = TRIDIAGONAL_RESIDUAL_TOLERANCE_FACTOR
            * f64::EPSILON
            * dimension
            * native_max(1.0, lambda.abs());
        for i in 0..n {
            let residual: f64 = scaled_residual(
                [original_diag, original_subdiag],
                eigen_vectors,
                i,
                j,
                scale,
                lambda,
            );
            if !residual.is_finite() || residual.abs() > tolerance {
                return false;
            }
        }
    }
    true
}
