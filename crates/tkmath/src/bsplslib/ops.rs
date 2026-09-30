// Rust translation of FoundationClasses/TKMath/BSplSLib/BSplSLib.cxx (InsertKnots, RemoveKnot, IncreaseDegree, Unperiodize) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Knot and degree operations along one direction of a surface.
//!
//! The surface is processed as a curve of curves: along U each "pole" is a whole
//! row of the grid (a point of dimension 3*RowLength, 4*RowLength when weighted).
use super::poles::{get_poles, get_rational_poles, set_poles, set_rational_poles};
use crate::bsplclib::{
    increase_degree_flat, insert_knots_flat, remove_knot_flat, unperiodize_flat,
};
use crate::gp::Point3;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;

fn flat(count: usize) -> Result<Array1<'static, f64>, Failure> {
    Array1::with_bounds(1, count as i32)
        .map_err(|error| Failure::out_of_range(&format!("BSplSLib: {error}")))
}

// The poles packed along the direction, with the curve dimension.
fn pack(
    poles: &Array2<'_, Point3>,
    weights: Option<&Array2<'_, f64>>,
    u_direction: bool,
) -> Result<(Array1<'static, f64>, i32), Failure> {
    let dim: usize = if weights.is_some() { 4 } else { 3 };
    let mut packed: Array1<'static, f64> = flat(dim * poles.len())?;
    match weights {
        Some(weights) => set_rational_poles(poles, weights, &mut packed, u_direction)?,
        None => set_poles(poles, &mut packed, u_direction)?,
    }
    let across: usize = if u_direction {
        poles.nb_columns()
    } else {
        poles.nb_rows()
    };
    Ok((packed, (dim * across) as i32))
}

fn unpack(
    packed: &Array1<'_, f64>,
    rational: bool,
    new_poles: &mut Array2<'_, Point3>,
    new_weights: Option<&mut Array2<'_, f64>>,
    u_direction: bool,
) -> Result<(), Failure> {
    if !rational {
        return get_poles(packed, new_poles, u_direction);
    }
    let new_weights = new_weights.ok_or_else(|| {
        Failure::out_of_range("BSplSLib: rational poles need NewWeights; received None")
    })?;
    get_rational_poles(packed, new_poles, new_weights, u_direction)
}

fn output(
    weights: Option<&Array2<'_, f64>>,
    new_poles: &Array2<'_, Point3>,
) -> Result<Array1<'static, f64>, Failure> {
    let dim: usize = if weights.is_some() { 4 } else { 3 };
    flat(dim * new_poles.len())
}

/// Inserts knots along U (`u_direction`) or V (`BSplSLib::InsertKnots`);
/// `bsplslib::insert_knots(true, 3, false, &p, None, &k, &m, &add, None, &mut np, None, &mut nk, &mut nm, 0.0, true)?`.
pub fn insert_knots(
    u_direction: bool,
    degree: i32,
    periodic: bool,
    poles: &Array2<'_, Point3>,
    weights: Option<&Array2<'_, f64>>,
    knots: &Array1<'_, f64>,
    mults: &Array1<'_, i32>,
    add_knots: &Array1<'_, f64>,
    add_mults: Option<&Array1<'_, i32>>,
    new_poles: &mut Array2<'_, Point3>,
    new_weights: Option<&mut Array2<'_, f64>>,
    new_knots: &mut Array1<'_, f64>,
    new_mults: &mut Array1<'_, i32>,
    epsilon: f64,
    add: bool,
) -> Result<(), Failure> {
    let (packed, dim) = pack(poles, weights, u_direction)?;
    let mut new_packed: Array1<'static, f64> = output(weights, new_poles)?;
    insert_knots_flat(
        degree,
        periodic,
        dim,
        &packed,
        knots,
        mults,
        add_knots,
        add_mults,
        &mut new_packed,
        new_knots,
        new_mults,
        epsilon,
        add,
    )?;
    unpack(
        &new_packed,
        weights.is_some(),
        new_poles,
        new_weights,
        u_direction,
    )
}

/// Removes or lowers a knot along U or V (`BSplSLib::RemoveKnot`); on false the new
/// arrays keep their poles; `bsplslib::remove_knot(true, 2, 0, 3, false, &p, None, &k, &m, &mut np, None, &mut nk, &mut nm, 1e-7)?`.
pub fn remove_knot(
    u_direction: bool,
    index: i32,
    mult: i32,
    degree: i32,
    periodic: bool,
    poles: &Array2<'_, Point3>,
    weights: Option<&Array2<'_, f64>>,
    knots: &Array1<'_, f64>,
    mults: &Array1<'_, i32>,
    new_poles: &mut Array2<'_, Point3>,
    new_weights: Option<&mut Array2<'_, f64>>,
    new_knots: &mut Array1<'_, f64>,
    new_mults: &mut Array1<'_, i32>,
    tolerance: f64,
) -> Result<bool, Failure> {
    let (packed, dim) = pack(poles, weights, u_direction)?;
    let mut new_packed: Array1<'static, f64> = output(weights, new_poles)?;
    let removed: bool = remove_knot_flat(
        index,
        mult,
        degree,
        periodic,
        dim,
        &packed,
        knots,
        mults,
        &mut new_packed,
        new_knots,
        new_mults,
        tolerance,
    )?;
    if !removed {
        return Ok(false);
    }
    unpack(
        &new_packed,
        weights.is_some(),
        new_poles,
        new_weights,
        u_direction,
    )?;
    Ok(true)
}

/// Elevates the degree along U or V (`BSplSLib::IncreaseDegree`);
/// `bsplslib::increase_degree(true, 2, 3, false, &p, None, &k, &m, &mut np, None, &mut nk, &mut nm)?`.
pub fn increase_degree(
    u_direction: bool,
    degree: i32,
    new_degree: i32,
    periodic: bool,
    poles: &Array2<'_, Point3>,
    weights: Option<&Array2<'_, f64>>,
    knots: &Array1<'_, f64>,
    mults: &Array1<'_, i32>,
    new_poles: &mut Array2<'_, Point3>,
    new_weights: Option<&mut Array2<'_, f64>>,
    new_knots: &mut Array1<'_, f64>,
    new_mults: &mut Array1<'_, i32>,
) -> Result<(), Failure> {
    let (packed, dim) = pack(poles, weights, u_direction)?;
    let mut new_packed: Array1<'static, f64> = output(weights, new_poles)?;
    increase_degree_flat(
        degree,
        new_degree,
        periodic,
        dim,
        &packed,
        knots,
        mults,
        &mut new_packed,
        new_knots,
        new_mults,
    )?;
    unpack(
        &new_packed,
        weights.is_some(),
        new_poles,
        new_weights,
        u_direction,
    )
}

/// Rewrites a periodic direction as non-periodic (`BSplSLib::Unperiodize`);
/// `bsplslib::unperiodize(true, 3, &m, &k, &p, None, &mut nm, &mut nk, &mut np, None)?`.
pub fn unperiodize(
    u_direction: bool,
    degree: i32,
    mults: &Array1<'_, i32>,
    knots: &Array1<'_, f64>,
    poles: &Array2<'_, Point3>,
    weights: Option<&Array2<'_, f64>>,
    new_mults: &mut Array1<'_, i32>,
    new_knots: &mut Array1<'_, f64>,
    new_poles: &mut Array2<'_, Point3>,
    new_weights: Option<&mut Array2<'_, f64>>,
) -> Result<(), Failure> {
    let (packed, dim) = pack(poles, weights, u_direction)?;
    let mut new_packed: Array1<'static, f64> = output(weights, new_poles)?;
    unperiodize_flat(
        degree,
        dim,
        mults,
        knots,
        &packed,
        new_mults,
        new_knots,
        &mut new_packed,
    )?;
    unpack(
        &new_packed,
        weights.is_some(),
        new_poles,
        new_weights,
        u_direction,
    )
}
