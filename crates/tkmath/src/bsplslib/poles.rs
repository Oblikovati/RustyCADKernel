// Rust translation of FoundationClasses/TKMath/BSplSLib/BSplSLib.cxx (SetPoles, GetPoles) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Packing of a pole grid into reals, row by row (U direction) or column by column.
use crate::gp::Point3;
use crate::plib::access::{array_cell, array_value, grid_cell, grid_value};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;

// Grid indices in packing order: rows then columns (U direction) or the reverse.
fn packing_order(rows: (i32, i32), cols: (i32, i32), u_direction: bool) -> Vec<(i32, i32)> {
    let mut order: Vec<(i32, i32)> = Vec::new();
    if u_direction {
        for i in rows.0..=rows.1 {
            for j in cols.0..=cols.1 {
                order.push((i, j));
            }
        }
    } else {
        for j in cols.0..=cols.1 {
            for i in rows.0..=rows.1 {
                order.push((i, j));
            }
        }
    }
    order
}

fn grid_order<T>(grid: &Array2<'_, T>, u_direction: bool) -> Vec<(i32, i32)> {
    packing_order(
        (grid.lower_row(), grid.upper_row()),
        (grid.lower_col(), grid.upper_col()),
        u_direction,
    )
}

/// Writes the pole coordinates from `FP.Lower()` (`BSplSLib::SetPoles`);
/// `bsplslib::set_poles(&poles, &mut flat, true)?`.
pub fn set_poles(
    poles: &Array2<'_, Point3>,
    fp: &mut Array1<'_, f64>,
    u_direction: bool,
) -> Result<(), Failure> {
    let mut l: i32 = fp.lower();
    for (i, j) in grid_order(poles, u_direction) {
        let point: Point3 = grid_value(poles, i, j)?;
        for value in [point.x(), point.y(), point.z()] {
            *array_cell(fp, l)? = value;
            l += 1;
        }
    }
    Ok(())
}

/// Writes weighted coordinates followed by each weight (`BSplSLib::SetPoles` with Weights);
/// `bsplslib::set_rational_poles(&poles, &weights, &mut flat, true)?`.
pub fn set_rational_poles(
    poles: &Array2<'_, Point3>,
    weights: &Array2<'_, f64>,
    fp: &mut Array1<'_, f64>,
    u_direction: bool,
) -> Result<(), Failure> {
    let mut l: i32 = fp.lower();
    for (i, j) in grid_order(poles, u_direction) {
        let point: Point3 = grid_value(poles, i, j)?;
        let w: f64 = grid_value(weights, i, j)?;
        for value in [point.x() * w, point.y() * w, point.z() * w, w] {
            *array_cell(fp, l)? = value;
            l += 1;
        }
    }
    Ok(())
}

/// Reads poles back from packed coordinates (`BSplSLib::GetPoles`);
/// `bsplslib::get_poles(&flat, &mut poles, true)?`.
pub fn get_poles(
    fp: &Array1<'_, f64>,
    poles: &mut Array2<'_, Point3>,
    u_direction: bool,
) -> Result<(), Failure> {
    let mut l: i32 = fp.lower();
    for (i, j) in grid_order(poles, u_direction) {
        let point: &mut Point3 = grid_cell(poles, i, j)?;
        point.set_x(array_value(fp, l)?);
        point.set_y(array_value(fp, l + 1)?);
        point.set_z(array_value(fp, l + 2)?);
        l += 3;
    }
    Ok(())
}

/// Divides packed homogeneous coordinates by their weight (`BSplSLib::GetPoles` with
/// Weights); the weight is read and stored first, as natively;
/// `bsplslib::get_rational_poles(&flat, &mut poles, &mut weights, true)?`.
pub fn get_rational_poles(
    fp: &Array1<'_, f64>,
    poles: &mut Array2<'_, Point3>,
    weights: &mut Array2<'_, f64>,
    u_direction: bool,
) -> Result<(), Failure> {
    let mut l: i32 = fp.lower();
    for (i, j) in grid_order(poles, u_direction) {
        let w: f64 = array_value(fp, l + 3)?;
        *grid_cell(weights, i, j)? = w;
        let point: &mut Point3 = grid_cell(poles, i, j)?;
        point.set_x(array_value(fp, l)? / w);
        point.set_y(array_value(fp, l + 1)? / w);
        point.set_z(array_value(fp, l + 2)? / w);
        l += 4;
    }
    Ok(())
}
