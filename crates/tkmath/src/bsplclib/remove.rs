// Rust translation of FoundationClasses/TKMath/BSplCLib/BSplCLib.cxx (RemoveKnot) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Knot removal on poles of any dimension by the reversed Boor scheme.
use super::build::{boor_index, build_knots, get_pole};
use super::insert::copy_poles;
use super::knots::{first_u_knot_index, last_u_knot_index, pole_index};
use super::scheme::anti_boor_scheme;
use crate::plib::access::{array_cell, array_value, non_negative};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;

const REMOVE: &str = "BSplCLib::RemoveKnot";

/// Lowers the multiplicity of `Knots(Index)` to `mult`, removing the knot at 0
/// (`BSplCLib::RemoveKnot` on flat poles).
///
/// Returns false, leaving the new arrays untouched, when the index is outside the
/// removable range or two candidate poles differ by more than `tolerance`;
/// `bsplclib::remove_knot_flat(2, 0, 3, false, 3, &p, &k, &m, &mut np, &mut nk, &mut nm, 1e-7)?`.
#[allow(clippy::too_many_arguments)] // Mirrors the native twelve-argument signature.
pub fn remove_knot_flat(
    index: i32,
    mult: i32,
    degree: i32,
    periodic: bool,
    dimension: i32,
    poles: &Array1<'_, f64>,
    knots: &Array1<'_, f64>,
    mults: &Array1<'_, i32>,
    new_poles: &mut Array1<'_, f64>,
    new_knots: &mut Array1<'_, f64>,
    new_mults: &mut Array1<'_, i32>,
    tolerance: f64,
) -> Result<bool, Failure> {
    let (first, last): (i32, i32) = if periodic {
        (knots.lower(), knots.upper())
    } else {
        (
            first_u_knot_index(degree, mults)? + 1,
            last_u_knot_index(degree, mults)? - 1,
        )
    };
    if index < first || index > last {
        return Ok(false);
    }
    let the_index: i32 = if periodic && index == first {
        last
    } else {
        index
    };
    let removal = Removal {
        the_index,
        mult,
        degree,
        periodic,
        dimension,
        depth: array_value(mults, the_index)? - mult,
        length: degree - mult,
    };
    let (lk, pole_start) = removal.local_knots(knots, mults)?;
    let mut lp: Vec<f64> = removal.local_poles(poles, pole_start)?;
    let u: f64 = array_value(knots, the_index)?;
    let dim: i32 = dimension;
    if !anti_boor_scheme(
        u,
        degree,
        &lk,
        dim,
        &mut lp,
        removal.depth,
        removal.length,
        tolerance,
    )? {
        return Ok(false);
    }
    removal.copy_poles(poles, pole_start, &lp, new_poles)?;
    removal.copy_knots([first, last], knots, mults, new_knots, new_mults)?;
    Ok(true)
}

struct Removal {
    the_index: i32,
    mult: i32,
    degree: i32,
    periodic: bool,
    dimension: i32,
    depth: i32,
    length: i32,
}

impl Removal {
    // The knots after removal, from the knots around TheIndex-1 and TheIndex,
    // and the first pole of the scheme.
    fn local_knots(
        &self,
        knots: &Array1<'_, f64>,
        mults: &Array1<'_, i32>,
    ) -> Result<(Vec<f64>, i32), Failure> {
        let deg: usize = non_negative(REMOVE, "Degree", self.degree)?;
        let mut lk: Vec<f64> = vec![0.0; 4 * deg];
        let (lower, upper) = lk.split_at_mut(2 * deg);
        build_knots(
            self.degree,
            self.the_index - 1,
            self.periodic,
            knots,
            Some(mults),
            lower,
        )?;
        let index: i32 = pole_index(self.degree, self.the_index - 1, self.periodic, mults)?;
        build_knots(
            self.degree,
            self.the_index,
            self.periodic,
            knots,
            Some(mults),
            upper,
        )?;
        let shift: usize = non_negative(REMOVE, "Mult", self.mult)?;
        for i in 0..deg.saturating_sub(shift) {
            lk[i] = lk[i + shift];
        }
        for i in deg.saturating_sub(shift)..2 * deg {
            lk[i] = lk[2 * deg + i];
        }
        Ok((lk, index + self.mult))
    }

    // Poles at their Boor positions, read cyclically from Poles.Lower()+index*Dimension.
    fn local_poles(&self, poles: &Array1<'_, f64>, index: i32) -> Result<Vec<f64>, Failure> {
        let deg: usize = non_negative(REMOVE, "Degree", self.degree)?;
        let dim: usize = non_negative(REMOVE, "Dimension", self.dimension)?;
        let mut lp: Vec<f64> = vec![0.0; (2 * deg + 1) * dim];
        let mut p: i32 = poles.lower() + index * self.dimension;
        for i in 0..=self.length + self.depth {
            let j: i32 = self.dimension * boor_index(i, self.length, self.depth);
            for k in 0..self.dimension {
                let slot: usize = usize::try_from(j + k)
                    .ok()
                    .filter(|&slot| slot < lp.len())
                    .ok_or_else(|| {
                        Failure::out_of_range(&format!(
                            "{REMOVE}: Boor slot {} outside {} local values",
                            j + k,
                            lp.len()
                        ))
                    })?;
                lp[slot] = array_value(poles, p + k)?;
            }
            p += self.dimension;
            if p > poles.upper() {
                p = poles.lower();
            }
        }
        Ok(lp)
    }

    fn copy_poles(
        &self,
        poles: &Array1<'_, f64>,
        index: i32,
        lp: &[f64],
        new_poles: &mut Array1<'_, f64>,
    ) -> Result<(), Failure> {
        let (mut p, mut np): (i32, i32) = (poles.lower(), new_poles.lower());
        // Unmodified poles before.
        copy_poles(
            (index + 1) * self.dimension,
            &mut p,
            poles,
            &mut np,
            new_poles,
        )?;
        for i in 1..=self.length {
            get_pole(i, self.length, 0, self.dimension, lp, &mut np, new_poles)?;
        }
        p += (self.length + self.depth) * self.dimension;
        // Unmodified poles after.
        if p != poles.lower() {
            let count: i32 = poles.upper() - p + 1;
            copy_poles(count, &mut p, poles, &mut np, new_poles)?;
        }
        Ok(())
    }

    fn copy_knots(
        &self,
        [first, last]: [i32; 2],
        knots: &Array1<'_, f64>,
        mults: &Array1<'_, i32>,
        new_knots: &mut Array1<'_, f64>,
        new_mults: &mut Array1<'_, i32>,
    ) -> Result<(), Failure> {
        if self.mult > 0 {
            new_knots.assign(knots);
            new_mults.assign(mults);
            *array_cell(new_mults, self.the_index)? = self.mult;
            if self.periodic && self.the_index == first {
                *array_cell(new_mults, last)? = self.mult;
            }
            if self.periodic && self.the_index == last {
                *array_cell(new_mults, first)? = self.mult;
            }
            return Ok(());
        }
        if !self.periodic || (self.the_index != first && self.the_index != last) {
            return drop_knot(self.the_index, knots, mults, new_knots, new_mults);
        }
        // A periodic curve whose first and last knot is removed.
        for i in first..last - 1 {
            *array_cell(new_knots, i)? = array_value(knots, i + 1)?;
            *array_cell(new_mults, i)? = array_value(mults, i + 1)?;
        }
        let knot: f64 =
            array_value(new_knots, first)? + array_value(knots, last)? - array_value(knots, first)?;
        *array_cell(new_knots, last - 1)? = knot;
        *array_cell(new_mults, last - 1)? = array_value(new_mults, first)?;
        Ok(())
    }
}

fn drop_knot(
    the_index: i32,
    knots: &Array1<'_, f64>,
    mults: &Array1<'_, i32>,
    new_knots: &mut Array1<'_, f64>,
    new_mults: &mut Array1<'_, i32>,
) -> Result<(), Failure> {
    for i in knots.lower()..the_index {
        *array_cell(new_knots, i)? = array_value(knots, i)?;
        *array_cell(new_mults, i)? = array_value(mults, i)?;
    }
    for i in the_index + 1..=knots.upper() {
        *array_cell(new_knots, i - 1)? = array_value(knots, i)?;
        *array_cell(new_mults, i - 1)? = array_value(mults, i)?;
    }
    Ok(())
}
