//! Checked math_Vector / math_Matrix accesses: the native `operator()` raises
//! Standard_OutOfRange outside the bounds (RelWithDebInfo keeps the checks).
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::{Matrix, Vector};

fn outside(what: &str, index: String, bounds: String) -> Failure {
    Failure::out_of_range(&format!("{what} index {index}; expected one in {bounds}"))
}

/// `V(i)`; `vget(&v, 1)?`.
pub(crate) fn vget(vector: &Vector<'_>, index: i32) -> Result<f64, Failure> {
    vector.value(index).copied()
}

/// `V(i) = value`; `vset(&mut v, 1, 0.0)?`.
pub(crate) fn vset(vector: &mut Vector<'_>, index: i32, value: f64) -> Result<(), Failure> {
    if index < vector.lower() || index > vector.upper() {
        let bounds = format!("[{}, {}]", vector.lower(), vector.upper());
        return Err(outside("math_Vector", index.to_string(), bounds));
    }
    vector[index] = value;
    Ok(())
}

fn check(matrix: &Matrix<'_>, row: i32, col: i32) -> Result<(), Failure> {
    let rows = matrix.lower_row()..=matrix.upper_row();
    let cols = matrix.lower_col()..=matrix.upper_col();
    if rows.contains(&row) && cols.contains(&col) {
        return Ok(());
    }
    let bounds = format!("[{:?}] x [{:?}]", rows, cols);
    Err(outside("math_Matrix", format!("({row}, {col})"), bounds))
}

/// `M(i, j) = value`; `mset(&mut m, 1, 1, 0.0)?`.
pub(crate) fn mset(matrix: &mut Matrix<'_>, row: i32, col: i32, value: f64) -> Result<(), Failure> {
    check(matrix, row, col)?;
    matrix[(row, col)] = value;
    Ok(())
}

/// A new vector of bounds `[1, n]` filled with zeros (native uninitialized).
pub(crate) fn vector(n: i32) -> Result<Vector<'static>, Failure> {
    Vector::filled(1, n, 0.0)
}

/// A new matrix of bounds `[1, rows] x [1, cols]` filled with zeros (native uninitialized).
pub(crate) fn matrix(rows: i32, cols: i32) -> Result<Matrix<'static>, Failure> {
    Matrix::filled(1, rows, 1, cols, 0.0)
        .map_err(|error| Failure::with_message("Standard_RangeError", &format!("{error:?}")))
}

/// `Dst = Src` for vectors of equal length (Standard_DimensionError otherwise).
pub(crate) fn copy_vector(target: &mut Vector<'_>, source: &Vector<'_>) -> Result<(), Failure> {
    target.initialized(source)
}

/// `Dst = Src` for matrices of equal shape.
pub(crate) fn copy_matrix(target: &mut Matrix<'_>, source: &Matrix<'_>) -> Result<(), Failure> {
    if target.row_number() != source.row_number() || target.col_number() != source.col_number() {
        return Err(Failure::with_message(
            "Standard_DimensionError",
            &format!(
                "math_Matrix assignment {}x{} from {}x{}; expected equal shapes",
                target.row_number(),
                target.col_number(),
                source.row_number(),
                source.col_number()
            ),
        ));
    }
    for (row_t, row_s) in (target.lower_row()..=target.upper_row()).zip(source.lower_row()..) {
        for (col_t, col_s) in (target.lower_col()..=target.upper_col()).zip(source.lower_col()..) {
            target[(row_t, col_t)] = source[(row_s, col_s)];
        }
    }
    Ok(())
}

/// `gp_Pnt(gp_XYZ)`; `point_from(xyz)`.
pub(crate) fn point_from(xyz: rustycad_tkmath::gp::Xyz) -> rustycad_tkmath::gp::Point3 {
    let mut point = rustycad_tkmath::gp::Point3::default();
    point.set_xyz(xyz);
    point
}

/// `InfBound(i) -= range; SupBound(i) += range` for finite bounds (the widening of the
/// `GetBounds` members).
pub(crate) fn widen(inf: &mut Vector<'_>, sup: &mut Vector<'_>, i: i32) -> Result<(), Failure> {
    let (low, up) = (vget(inf, i)?, vget(sup, i)?);
    if !rustycad_tkernel::precision::is_infinite(low)
        && !rustycad_tkernel::precision::is_infinite(up)
    {
        let range = up - low;
        vset(inf, i, low - range)?;
        vset(sup, i, up + range)?;
    }
    Ok(())
}
