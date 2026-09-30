// Rust translation of ModelingData/TKG3d/Geom/Geom_BezierSurface.cxx (AddPoleCol, AddPoleRow, DeletePoleCol, DeletePoleRow and their rational twins) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The file-static grid copies of Geom_BezierSurface.cxx, written once for poles and
//! weights: the rational variants run the same loops over both arrays, which only
//! copy values, so two passes give the same arrays.
use crate::geom::errors;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;

/// `grid(row, col)` with the native Standard_OutOfRange check; `at2(&poles, 1, 2)?`.
pub(super) fn at2<T: Copy>(grid: &Array2<'_, T>, row: i32, col: i32) -> Result<T, Failure> {
    grid.value(row, col).copied().map_err(errors::array_bounds)
}

/// `grid(row, col) = value` with the native Standard_OutOfRange check.
pub(super) fn put2<T>(
    grid: &mut Array2<'_, T>,
    row: i32,
    col: i32,
    value: T,
) -> Result<(), Failure> {
    grid.set_value(row, col, value)
        .map_err(errors::array_bounds)
}

/// `array(index)` with the native Standard_OutOfRange check.
pub(super) fn at1<T: Copy>(array: &Array1<'_, T>, index: i32) -> Result<T, Failure> {
    array.value(index).copied().map_err(errors::array_bounds)
}

/// A new grid with bounds `(1..rows, 1..cols)` (`NCollection_Array2(1, rows, 1, cols)`).
pub(super) fn one_based_grid<T: Default>(
    rows: i32,
    cols: i32,
) -> Result<Array2<'static, T>, Failure> {
    Array2::with_bounds(1, rows, 1, cols).map_err(|error| {
        errors::out_of_range(&format!(
            "NCollection_Array2(1, {rows}, 1, {cols}): {error}; expected positive sizes"
        ))
    })
}

/// `AddPoleCol`: copy `source` into `target` (one more column) with `column` inserted
/// after column `after`; `add_col(&poles, &col, 2, &mut new_poles)?`.
pub(super) fn add_col<T: Copy>(
    source: &Array2<'_, T>,
    column: &Array1<'_, T>,
    after: i32,
    target: &mut Array2<'_, T>,
) -> Result<(), Failure> {
    let insert: i32 = after + target.lower_col();
    let offset: i32 = target.lower_row() - column.lower();
    let (rows, last) = (target.lower_row()..=target.upper_row(), target.upper_col());
    for col in target.lower_col()..=last {
        for row in rows.clone() {
            let value: T = match col.cmp(&insert) {
                std::cmp::Ordering::Less => at2(source, row, col)?,
                std::cmp::Ordering::Equal => at1(column, row - offset)?,
                std::cmp::Ordering::Greater => at2(source, row, col - 1)?,
            };
            put2(target, row, col, value)?;
        }
    }
    Ok(())
}

/// `AddPoleRow`: copy `source` into `target` (one more row) with `line` inserted after
/// row `after`; `add_row(&poles, &row, 2, &mut new_poles)?`.
pub(super) fn add_row<T: Copy>(
    source: &Array2<'_, T>,
    line: &Array1<'_, T>,
    after: i32,
    target: &mut Array2<'_, T>,
) -> Result<(), Failure> {
    let insert: i32 = after + target.lower_row();
    let offset: i32 = target.lower_col() - line.lower();
    let (cols, last) = (target.lower_col()..=target.upper_col(), target.upper_row());
    for row in target.lower_row()..=last {
        for col in cols.clone() {
            let value: T = match row.cmp(&insert) {
                std::cmp::Ordering::Less => at2(source, row, col)?,
                std::cmp::Ordering::Equal => at1(line, col - offset)?,
                std::cmp::Ordering::Greater => at2(source, row - 1, col)?,
            };
            put2(target, row, col, value)?;
        }
    }
    Ok(())
}

/// `DeletePoleCol`: copy `source` into `target` (one less column) without column `index`;
/// `delete_col(&poles, 2, &mut new_poles)?`.
pub(super) fn delete_col<T: Copy>(
    source: &Array2<'_, T>,
    index: i32,
    target: &mut Array2<'_, T>,
) -> Result<(), Failure> {
    for col in target.lower_col()..=target.upper_col() {
        let shift: i32 = i32::from(col >= index);
        for row in target.lower_row()..=target.upper_row() {
            put2(target, row, col, at2(source, row, col + shift)?)?;
        }
    }
    Ok(())
}

/// `DeletePoleRow`: copy `source` into `target` (one less row) without row `index`;
/// `delete_row(&poles, 2, &mut new_poles)?`.
pub(super) fn delete_row<T: Copy>(
    source: &Array2<'_, T>,
    index: i32,
    target: &mut Array2<'_, T>,
) -> Result<(), Failure> {
    for row in target.lower_row()..=target.upper_row() {
        let shift: i32 = i32::from(row >= index);
        for col in target.lower_col()..=target.upper_col() {
            put2(target, row, col, at2(source, row + shift, col)?)?;
        }
    }
    Ok(())
}
