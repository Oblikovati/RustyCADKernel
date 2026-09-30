// Rust translation of FoundationClasses/TKMath/BSplCLib/BSplCLib.cxx (IncreaseDegreeCountKnots, IncreaseDegree) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Degree elevation by averaging knot-inserted curves (Prautzsch, CAGD 1, 1984).
use super::insert::insert_knots_flat;
use super::knots::{first_u_knot_index, last_u_knot_index};
use super::views::{one_based, prefix_view};
use crate::plib::access::{array_cell, array_value};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;

const INCREASE: &str = "BSplCLib::IncreaseDegree";

/// Number of knots after elevating the degree (`BSplCLib::IncreaseDegreeCountKnots`);
/// `bsplclib::increase_degree_count_knots(3, 4, false, &mults)?`.
pub fn increase_degree_count_knots(
    degree: i32,
    new_degree: i32,
    periodic: bool,
    mults: &Array1<'_, i32>,
) -> Result<i32, Failure> {
    if periodic {
        return Ok(mults.len() as i32);
    }
    let f: i32 = first_u_knot_index(degree, mults)?;
    let l: i32 = last_u_knot_index(degree, mults)?;
    let step: i32 = new_degree - degree;
    let mut removed: i32 = 0;
    let mut i: i32 = mults.lower();
    let mut m: i32 = degree + (f - i + 1) * step + 1;
    while m > new_degree + 1 {
        removed += 1;
        m -= array_value(mults, i)? + step;
        i += 1;
    }
    if m < new_degree + 1 {
        removed -= 1;
    }
    i = mults.upper();
    m = degree + (i - l + 1) * step + 1;
    while m > new_degree + 1 {
        removed += 1;
        m -= array_value(mults, i)? + step;
        i -= 1;
    }
    if m < new_degree + 1 {
        removed -= 1;
    }
    Ok(mults.len() as i32 - removed)
}

/// Elevates a curve with `dimension` interleaved pole values from `degree` to
/// `new_degree` (`BSplCLib::IncreaseDegree` on flat poles).
///
/// Size the new knots with [`increase_degree_count_knots`]. Quirk kept: the working
/// poles are read from `wpoles(NewPoles.Lower() + i*Dimension + k)`, so a non-1 lower
/// bound of `new_poles` shifts that read;
/// `bsplclib::increase_degree_flat(2, 3, false, 3, &p, &k, &m, &mut np, &mut nk, &mut nm)?`.
#[allow(clippy::too_many_arguments)] // Mirrors the native ten-argument signature.
pub fn increase_degree_flat(
    degree: i32,
    new_degree: i32,
    periodic: bool,
    dimension: i32,
    poles: &Array1<'_, f64>,
    knots: &Array1<'_, f64>,
    mults: &Array1<'_, i32>,
    new_poles: &mut Array1<'_, f64>,
    new_knots: &mut Array1<'_, f64>,
    new_mults: &mut Array1<'_, i32>,
) -> Result<(), Failure> {
    let mut work = WorkingCurve::new(degree, new_degree, periodic, dimension, knots, mults)?;
    work.fill_poles(poles)?;
    for cur_deg in degree..new_degree {
        work.elevate_once(cur_deg, new_poles.lower())?;
    }
    work.copy_results(new_poles, new_knots, new_mults)
}

// The non-periodic working curve with end multiplicities Degree+1 (null poles added).
struct WorkingCurve {
    degree: i32,
    new_degree: i32,
    periodic: bool,
    dimension: i32,
    f: i32,
    l: i32,
    upper_mult: i32,
    pf: i32,
    pl: i32,
    wknots: Array1<'static, f64>,
    wmults: Array1<'static, i32>,
    nbwpoles: i32,
    // Maximum number of poles after all degree elevations.
    max_poles: i32,
    wpoles: Array1<'static, f64>,
}

impl WorkingCurve {
    fn new(
        degree: i32,
        new_degree: i32,
        periodic: bool,
        dimension: i32,
        knots: &Array1<'_, f64>,
        mults: &Array1<'_, i32>,
    ) -> Result<Self, Failure> {
        let f: i32 = first_u_knot_index(degree, mults)?;
        let l: i32 = last_u_knot_index(degree, mults)?;
        let (mut nbwknots, mut pf, mut pl): (i32, i32, i32) = (knots.len() as i32, 0, 0);
        if periodic {
            // Periodic curves are transformed in non-periodic curves.
            nbwknots += f - mults.lower();
            pf = -degree - 1 + sum_mults(mults, mults.lower(), f)?;
            nbwknots += mults.upper() - l;
            pl = -degree - 1 + sum_mults(mults, l, mults.upper())?;
        }
        let mut curve = Self {
            degree,
            new_degree,
            periodic,
            dimension,
            f,
            l,
            upper_mult: mults.upper(),
            pf,
            pl,
            wknots: one_based(INCREASE, nbwknots)?,
            wmults: one_based(INCREASE, nbwknots)?,
            nbwpoles: 0,
            max_poles: 0,
            wpoles: Array1::default(),
        };
        curve.copy_knots(knots, mults)?;
        curve.clamp_ends(nbwknots)?;
        Ok(curve)
    }

    fn copy_knots(
        &mut self,
        knots: &Array1<'_, f64>,
        mults: &Array1<'_, i32>,
    ) -> Result<(), Failure> {
        if !self.periodic {
            self.wknots.assign(knots);
            self.wmults.assign(mults);
            return Ok(());
        }
        let period: f64 = array_value(knots, knots.upper())? - array_value(knots, knots.lower())?;
        let mut i: i32 = 0;
        let ranges: [(i32, i32, f64); 3] = [
            (self.l, knots.upper() - 1, -period),
            (knots.lower(), knots.upper(), 0.0),
            (knots.lower() + 1, self.f, period),
        ];
        for (from, to, shift) in ranges {
            for k in from..=to {
                i += 1;
                let knot: f64 = array_value(knots, k)?;
                // The middle copy is exact; the others add or subtract the period.
                *array_cell(&mut self.wknots, i)? = if shift == 0.0 { knot } else { knot + shift };
                *array_cell(&mut self.wmults, i)? = array_value(mults, k)?;
            }
        }
        Ok(())
    }

    // Sets the first and last multiplicities to Degree+1, counting the null poles added.
    fn clamp_ends(&mut self, nbwknots: i32) -> Result<(), Failure> {
        let deg1: i32 = self.degree + 1;
        self.pf += deg1 - array_value(&self.wmults, 1)?;
        *array_cell(&mut self.wmults, 1)? = deg1;
        self.pl += deg1 - array_value(&self.wmults, nbwknots)?;
        *array_cell(&mut self.wmults, nbwknots)? = deg1;
        self.nbwpoles = sum_mults(&self.wmults, 1, nbwknots)? - deg1;
        self.max_poles = self.nbwpoles + (nbwknots - 1) * (self.new_degree - self.degree);
        Ok(())
    }

    fn nbwknots(&self) -> i32 {
        self.wknots.len() as i32
    }

    fn fill_poles(&mut self, poles: &Array1<'_, f64>) -> Result<(), Failure> {
        let dim: i32 = self.dimension;
        self.wpoles = one_based(INCREASE, self.max_poles * dim)?;
        for i in 1..=self.pf * dim {
            *array_cell(&mut self.wpoles, i)? = 0.0;
        }
        let mut k: i32 = poles.lower();
        for i in self.pf * dim + 1..=(self.nbwpoles - self.pl) * dim {
            *array_cell(&mut self.wpoles, i)? = array_value(poles, k)?;
            k += 1;
            if k > poles.upper() {
                k = poles.lower();
            }
        }
        for i in (self.nbwpoles - self.pl) * dim + 1..=self.nbwpoles * dim {
            *array_cell(&mut self.wpoles, i)? = 0.0;
        }
        Ok(())
    }
}

fn sum_mults(mults: &Array1<'_, i32>, from: i32, to: i32) -> Result<i32, Failure> {
    let mut sum: i32 = 0;
    for i in from..=to {
        sum += array_value(mults, i)?;
    }
    Ok(sum)
}

impl WorkingCurve {
    // One degree incrementation: the average of curDeg+1 curves with raised multiplicities.
    fn elevate_once(&mut self, cur_deg: i32, new_poles_lower: i32) -> Result<(), Failure> {
        let dim: i32 = self.dimension;
        let nbp: i32 = self.nbwpoles;
        let nbwp: i32 = nbp + self.nbwknots() - 1;
        let mut nwpoles: Array1<'static, f64> = one_based(INCREASE, self.max_poles * dim)?;
        let mut tempc1: Array1<'static, f64> = one_based(INCREASE, self.max_poles * dim)?;
        let mut tempc2: Array1<'static, f64> = one_based(INCREASE, self.max_poles * dim)?;
        for step in 0..=cur_deg {
            if step != 0 {
                for i in 1..=self.nbwknots() {
                    *array_cell(&mut self.wmults, i)? -= 1;
                }
            }
            let offset: i32 =
                self.duplicate_poles(cur_deg, step, nbp, new_poles_lower, &mut tempc1)?;
            let iknots: Vec<f64> = self.raise_mults(cur_deg, step)?;
            let curve: &Array1<'_, f64> = if iknots.is_empty() {
                &tempc1
            } else {
                self.insert_raised(
                    cur_deg,
                    &iknots,
                    &tempc1,
                    offset * dim,
                    nbwp * dim,
                    &mut tempc2,
                )?;
                &tempc2
            };
            // Add to the average.
            for i in 1..=nbwp * dim {
                *array_cell(&mut nwpoles, i)? += array_value(curve, i)?;
            }
        }
        // The result is the average.
        for i in 1..=nbwp * dim {
            *array_cell(&mut self.wpoles, i)? = array_value(&nwpoles, i)? / f64::from(cur_deg + 1);
        }
        self.nbwpoles = nbwp;
        Ok(())
    }

    // The current poles, with the poles congruent to `step` duplicated.
    fn duplicate_poles(
        &self,
        cur_deg: i32,
        step: i32,
        nbp: i32,
        new_poles_lower: i32,
        tempc1: &mut Array1<'_, f64>,
    ) -> Result<i32, Failure> {
        let dim: i32 = self.dimension;
        let mut offset: i32 = 0;
        for i in 0..nbp {
            let copies: i32 = if i % (cur_deg + 1) == step { 2 } else { 1 };
            for _ in 0..copies {
                offset += 1;
                for k in 0..dim {
                    let value: f64 = array_value(&self.wpoles, new_poles_lower + i * dim + k)?;
                    *array_cell(tempc1, (offset - 1) * dim + k + 1)? = value;
                }
            }
        }
        Ok(offset)
    }

    // Raises the multiplicity of knots where the running sum reaches the step;
    // returns the other knots, which must be inserted.
    fn raise_mults(&mut self, cur_deg: i32, step: i32) -> Result<Vec<f64>, Failure> {
        let (mut stepmult, mut smult): (i32, i32) = (step + 1, 0);
        let mut iknots: Vec<f64> = Vec::new();
        for k in 1..=self.nbwknots() {
            smult += array_value(&self.wmults, k)?;
            if smult >= stepmult {
                stepmult += cur_deg + 1;
                *array_cell(&mut self.wmults, k)? += 1;
            } else {
                iknots.push(array_value(&self.wknots, k)?);
            }
        }
        Ok(iknots)
    }

    // The curve obtained by inserting the knots to raise the multiplicities. The native
    // call passes wmults as both input and output; only multiplicities change, so
    // reading a snapshot of the input is equivalent.
    fn insert_raised(
        &mut self,
        cur_deg: i32,
        iknots: &[f64],
        tempc1: &Array1<'_, f64>,
        curve_len: i32,
        ncurve_len: i32,
        tempc2: &mut Array1<'_, f64>,
    ) -> Result<(), Failure> {
        let mut aknots: Array1<'static, f64> = one_based(INCREASE, iknots.len() as i32)?;
        aknots.as_mut_slice().copy_from_slice(iknots);
        let curve = prefix_view(INCREASE, tempc1, curve_len)?;
        let mut ncurve = super::views::prefix_view_mut(INCREASE, tempc2, ncurve_len)?;
        let mut nknots: Array1<'static, f64> = one_based(INCREASE, self.nbwknots())?;
        let input_mults: Array1<'static, i32> = self.wmults.clone();
        insert_knots_flat(
            cur_deg + 1,
            false,
            self.dimension,
            &curve,
            &self.wknots,
            &input_mults,
            &aknots,
            None,
            &mut ncurve,
            &mut nknots,
            &mut self.wmults,
            0.0,
            true,
        )
    }

    fn copy_results(
        &mut self,
        new_poles: &mut Array1<'_, f64>,
        new_knots: &mut Array1<'_, f64>,
        new_mults: &mut Array1<'_, i32>,
    ) -> Result<(), Failure> {
        // Index in the new knots of the first knot of the curve.
        let firstknot: i32 = if self.periodic {
            self.upper_mult - self.l + 1
        } else {
            self.f
        };
        let (mut k, mut pf): (i32, i32) = self.first_knot_and_pole(firstknot)?;
        // On a periodic curve the knots start with firstknot.
        if self.periodic {
            k = firstknot;
        }
        for i in new_knots.lower()..=new_knots.upper() {
            *array_cell(new_knots, i)? = array_value(&self.wknots, k)?;
            *array_cell(new_mults, i)? = array_value(&self.wmults, k)?;
            k += 1;
        }
        pf *= self.dimension;
        for i in new_poles.lower()..=new_poles.upper() {
            pf += 1;
            *array_cell(new_poles, i)? = array_value(&self.wpoles, pf)?;
        }
        Ok(())
    }

    // Removes knots until the multiplicities up to the start sum to NewDegree+1.
    fn first_knot_and_pole(&mut self, firstknot: i32) -> Result<(i32, i32), Failure> {
        let target: i32 = self.new_degree + 1;
        let mut m: i32 = sum_mults(&self.wmults, 1, firstknot)?;
        let (mut k, mut pf): (i32, i32) = (1, 0);
        while m > target {
            k += 1;
            let mult: i32 = array_value(&self.wmults, k)?;
            m -= mult;
            pf += mult;
        }
        if m < target {
            k -= 1;
            *array_cell(&mut self.wmults, k)? += m - target;
            pf += m - target;
        }
        Ok((k, pf))
    }
}
