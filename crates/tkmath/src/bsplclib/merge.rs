// Rust translation of FoundationClasses/TKMath/BSplCLib/BSplCLib_2.cxx (MergeBSplineKnots) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Knot vector of the product of two B-splines.
use super::forms::reparametrize;
use super::knots::nb_poles;
use super::views::one_based;
use crate::plib::access::{array_cell, array_value};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;

const MERGE: &str = "BSplCLib::MergeBSplineKnots";

/// The merged knot vector of two B-splines reparametrized to `[start_value, end_value]`
/// (`BSplCLib::MergeBSplineKnots`), for a product of degree Degree1+Degree2.
///
/// Returns `(NumPoles, NewKnots, NewMults)`, the arrays indexed from 1 (the native
/// handles). Knots closer than `tolerance` are merged. Multiplicities are read with
/// the 1-based knot indices, as natively;
/// `bsplclib::merge_bspline_knots(1e-9, 0.0, 1.0, 2, &k1, &m1, 3, &k2, &m2)?`.
pub fn merge_bspline_knots(
    tolerance: f64,
    start_value: f64,
    end_value: f64,
    degree1: i32,
    knots1: &Array1<'_, f64>,
    mults1: &Array1<'_, i32>,
    degree2: i32,
    knots2: &Array1<'_, f64>,
    mults2: &Array1<'_, i32>,
) -> Result<(i32, Array1<'static, f64>, Array1<'static, i32>), Failure> {
    let degree: i32 = degree1 + degree2;
    // The negated test keeps the native branch for NaN bounds.
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    if !(start_value < end_value - tolerance) {
        return two_knots(start_value, end_value, degree);
    }
    let first: Array1<'static, f64> = reparametrized(knots1, start_value, end_value)?;
    let second: Array1<'static, f64> = reparametrized(knots2, start_value, end_value)?;
    let merger = Merger {
        tolerance,
        degrees: [degree1, degree2],
        first: &first,
        second: &second,
    };
    let count: i32 = merger.count()?;
    let mut new_knots: Array1<'static, f64> = one_based(MERGE, count)?;
    let mut new_mults: Array1<'static, i32> = one_based(MERGE, count)?;
    let last: i32 = merger.fill([mults1, mults2], &mut new_knots, &mut new_mults)?;
    *array_cell(&mut new_mults, 1)? = degree + 1;
    *array_cell(&mut new_mults, last)? = degree + 1;
    let mut index: i32 = 0;
    for ii in 1..=last {
        index += array_value(&new_mults, ii)?;
    }
    Ok((index - degree - 1, new_knots, new_mults))
}

fn two_knots(
    start_value: f64,
    end_value: f64,
    degree: i32,
) -> Result<(i32, Array1<'static, f64>, Array1<'static, i32>), Failure> {
    let mut new_knots: Array1<'static, f64> = one_based(MERGE, 2)?;
    new_knots
        .as_mut_slice()
        .copy_from_slice(&[start_value, end_value]);
    let mut new_mults: Array1<'static, i32> = one_based(MERGE, 2)?;
    new_mults.init(degree + 1);
    let num_poles: i32 = nb_poles(degree, false, &new_mults)?;
    Ok((num_poles, new_knots, new_mults))
}

fn reparametrized(
    knots: &Array1<'_, f64>,
    start: f64,
    end: f64,
) -> Result<Array1<'static, f64>, Failure> {
    let mut copy: Array1<'static, f64> = one_based(MERGE, knots.len() as i32)?;
    copy.as_mut_slice().copy_from_slice(knots.as_slice());
    reparametrize(start, end, &mut copy)?;
    Ok(copy)
}

struct Merger<'r> {
    tolerance: f64,
    degrees: [i32; 2],
    first: &'r Array1<'static, f64>,
    second: &'r Array1<'static, f64>,
}

impl Merger<'_> {
    // Number of merged knots (the counting pass of the native code).
    fn count(&self) -> Result<i32, Failure> {
        let (mut num_knots, mut jj): (i32, i32) = (0, 1);
        let length2: i32 = self.second.len() as i32;
        for ii in 1..=self.first.len() as i32 {
            let knot: f64 = array_value(self.first, ii)?;
            while jj <= length2 && array_value(self.second, jj)? <= knot - self.tolerance {
                jj += 1;
                num_knots += 1;
            }
            while jj <= length2 && array_value(self.second, jj)? <= knot + self.tolerance {
                jj += 1;
            }
            num_knots += 1;
        }
        Ok(num_knots)
    }

    // Fills the merged knots; returns the index of the last one.
    fn fill(
        &self,
        [mults1, mults2]: [&Array1<'_, i32>; 2],
        new_knots: &mut Array1<'_, f64>,
        new_mults: &mut Array1<'_, i32>,
    ) -> Result<i32, Failure> {
        let [degree1, degree2] = self.degrees;
        let (mut num_knots, mut jj): (i32, i32) = (1, 1);
        let length2: i32 = self.second.len() as i32;
        for ii in 1..=self.first.len() as i32 {
            let knot: f64 = array_value(self.first, ii)?;
            while jj <= length2 && array_value(self.second, jj)? <= knot - self.tolerance {
                *array_cell(new_knots, num_knots)? = array_value(self.second, jj)?;
                *array_cell(new_mults, num_knots)? = array_value(mults2, jj)? + degree1;
                jj += 1;
                num_knots += 1;
            }
            let mut set_mults: bool = false;
            while jj <= length2 && array_value(self.second, jj)? <= knot + self.tolerance {
                let continuity: i32 =
                    (degree1 - array_value(mults1, ii)?).min(degree2 - array_value(mults2, jj)?);
                set_mults = true;
                *array_cell(new_mults, num_knots)? = degree1 + degree2 - continuity;
                jj += 1;
            }
            *array_cell(new_knots, num_knots)? = knot;
            if !set_mults {
                *array_cell(new_mults, num_knots)? = array_value(mults1, ii)? + degree2;
            }
            num_knots += 1;
        }
        Ok(num_knots - 1)
    }
}
